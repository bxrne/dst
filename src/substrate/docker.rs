use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;

use bollard::Docker as BollardDocker;
use bollard::container::LogOutput;
use bollard::errors::Error as BollardError;
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::{
    ContainerCreateBody, ContainerStateStatusEnum, ContainerUpdateBody, HostConfig,
    NetworkConnectRequest, NetworkDisconnectRequest, PortBinding, ThrottleDevice,
};
use bollard::query_parameters::CreateImageOptions;
use bollard::query_parameters::{
    CreateContainerOptions, InspectContainerOptions, LogsOptionsBuilder, RemoveContainerOptions,
    StartContainerOptions,
};
use futures_util::TryStreamExt;
use tracing::{debug, info, warn};

use crate::substrate::{
    ContainerState, ExecResult, Fault, HostedSubject, InspectResult, LogEntry, LogOptions, Stream,
    Subject, Substrate,
};

pub fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"))
}

pub struct Docker {
    connection: BollardDocker,
}

impl Docker {
    pub fn new() -> Result<Self, String> {
        let connection = BollardDocker::connect_with_local_defaults()
            .map_err(|e| format!("Failed to connect to Docker: {}", e))?;
        Ok(Self { connection })
    }

    fn block_on<F, Fut, T>(&self, f: F) -> T
    where
        F: FnOnce(BollardDocker) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.connection.clone();
        let rt = runtime();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = rt.block_on(f(conn));
            let _ = tx.send(result);
        });
        rx.recv().expect("docker operation thread panicked")
    }
}

#[derive(Clone, Debug)]
pub struct DockerSubjectData {
    pub image: String,
    pub cmd: Option<Vec<String>>,
    pub ports: Option<Vec<u16>>,
    pub volumes: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
}

impl Substrate for Docker {
    const NAME: &'static str = "docker";

    type SubjectData = DockerSubjectData;

    fn parse_subject(&self, table: &mlua::Table) -> Result<Self::SubjectData, String> {
        let image: String = table
            .get("image")
            .map_err(|_| "setup requires `image` field".to_string())?;
        let ports: Option<Vec<u16>> = table.get("ports").ok();
        let cmd: Option<Vec<String>> = table.get("cmd").ok();
        let volumes: Option<Vec<String>> = table.get("volumes").ok();
        let env: Option<HashMap<String, String>> = table.get("env").ok();
        let env = env.map(|e| e.into_iter().map(|(k, v)| format!("{}={}", k, v)).collect());

        Ok(DockerSubjectData {
            image,
            cmd,
            ports,
            volumes,
            env,
        })
    }

    fn host(&self, data: &Self::SubjectData) -> Result<HostedSubject, String> {
        let image = data.image.clone();
        self.block_on(|conn| async move {
            conn.create_image(
                Some(CreateImageOptions {
                    from_image: Some(image),
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await
        })
        .map_err(|e| format!("Failed to pull image: {}", e))?;

        let container_config = ContainerCreateBody {
            image: Some(data.image.clone()),
            cmd: data.cmd.clone(),
            exposed_ports: data
                .ports
                .as_ref()
                .map(|ports| ports.iter().map(|p| format!("{}/tcp", p)).collect()),
            host_config: Some(HostConfig {
                port_bindings: data.ports.as_ref().map(|ports| {
                    let mut map: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
                    for p in ports {
                        map.insert(
                            format!("{}/tcp", p),
                            Some(vec![PortBinding {
                                host_ip: None,
                                host_port: Some(p.to_string()),
                            }]),
                        );
                    }
                    map
                }),
                binds: data.volumes.clone(),
                ..Default::default()
            }),
            env: data.env.clone(),
            ..Default::default()
        };

        let container_config_clone = container_config.clone();
        let container = self
            .block_on(|conn| async move {
                conn.create_container(None::<CreateContainerOptions>, container_config_clone)
                    .await
            })
            .map_err(|e| format!("Failed to create container: {}", e))?;

        let container_id = container.id.clone();
        let container_id_clone = container_id.clone();
        self.block_on(|conn| async move {
            conn.start_container(&container_id_clone, None::<StartContainerOptions>)
                .await
        })
        .map_err(|e| format!("Failed to start container: {}", e))?;

        let addr = data
            .ports
            .as_ref()
            .and_then(|ports| ports.first())
            .map(|p| format!("localhost:{}", p));

        info!("Started container id={}", container.id);
        Ok(HostedSubject {
            id: container.id,
            addr,
        })
    }

    fn affect(&self, subject: &Subject, fault: &Fault) -> Result<(), String> {
        let id = Self::container_id(subject).to_string();

        match fault {
            Fault::Pause => {
                let id_for_call = id.clone();
                match self.block_on(|conn| async move { conn.pause_container(&id_for_call).await })
                {
                    Ok(_) => info!("Paused container id={}", id),
                    Err(BollardError::DockerResponseServerError {
                        status_code: 409, ..
                    }) => debug!("Container id={} already paused", id),
                    Err(e) => return Err(format!("Failed to pause container {}: {}", id, e)),
                }
            }
            Fault::Kill => {
                let id_for_call = id.clone();
                match self
                    .block_on(|conn| async move { conn.kill_container(&id_for_call, None).await })
                {
                    Ok(_) => info!("Killed container id={}", id),
                    Err(BollardError::DockerResponseServerError {
                        status_code: 409, ..
                    }) => debug!("Container id={} not running", id),
                    Err(e) => return Err(format!("Failed to kill container {}: {}", id, e)),
                }
            }
            Fault::Deprive(tier) => {
                info!("Depriving container id={} tier={}", id, tier);
                self.deprive_resource(subject, tier)?;
            }
        }
        Ok(())
    }

    fn clear_faults(&self, subject: &Subject) -> Result<(), String> {
        let id = Self::container_id(subject).to_string();
        info!("Clearing faults id={}", id);

        let id_for_call = id.clone();
        match self.block_on(|conn| async move { conn.unpause_container(&id_for_call).await }) {
            Ok(_) => debug!("Unpaused container id={}", id),
            Err(BollardError::DockerResponseServerError {
                status_code: 409, ..
            }) => {}
            Err(BollardError::DockerResponseServerError {
                status_code: 404, ..
            }) => {}
            Err(e) => debug!("Failed to unpause container id={} error=\"{}\"", id, e),
        }

        self.restart_if_killed(subject)?;
        self.reconnect_network(subject)?;
        self.clear_resource_limits(subject)?;

        Ok(())
    }

    fn teardown(&self, subject: Subject) -> Result<(), String> {
        let id = Self::container_id(&subject).to_string();
        info!("Tearing down container id={}", id);

        let id_for_call = id.clone();
        self.block_on(|conn| async move { conn.stop_container(&id_for_call, None).await })
            .map_err(|e| format!("Failed to stop container: {}", e))?;

        let options = RemoveContainerOptions {
            v: true,
            force: true,
            link: false,
        };
        self.block_on(|conn| async move { conn.remove_container(&id, Some(options)).await })
            .map_err(|e| format!("Failed to remove container: {}", e))?;

        Ok(())
    }

    fn logs(&self, subject: &Subject, opts: LogOptions) -> Result<Vec<LogEntry>, String> {
        let id = Self::container_id(subject).to_string();

        let mut builder = LogsOptionsBuilder::new()
            .stdout(opts.stdout)
            .stderr(opts.stderr)
            .timestamps(opts.timestamps);

        if let Some(tail) = opts.tail {
            builder = builder.tail(&tail);
        }
        if let Some(since) = opts.since {
            builder = builder.since(since);
        }

        let options = builder.build();

        let stream = self
            .block_on(
                |conn| async move { conn.logs(&id, Some(options)).try_collect::<Vec<_>>().await },
            )
            .map_err(|e| format!("Failed to get logs: {}", e))?;

        stream
            .into_iter()
            .filter_map(|entry| match entry {
                LogOutput::StdOut { message } => Some(LogEntry {
                    stream: Stream::StdOut,
                    message: String::from_utf8_lossy(&message).to_string(),
                }),
                LogOutput::StdErr { message } => Some(LogEntry {
                    stream: Stream::StdErr,
                    message: String::from_utf8_lossy(&message).to_string(),
                }),
                _ => None,
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(Ok)
            .collect()
    }

    fn inspect(&self, subject: &Subject) -> Result<InspectResult, String> {
        let id = Self::container_id(subject).to_string();

        let info = self
            .block_on(|conn| async move {
                conn.inspect_container(&id, None::<InspectContainerOptions>)
                    .await
            })
            .map_err(|e| format!("Inspect failed: {}", e))?;

        let state = match info.state.as_ref().and_then(|s| s.status) {
            Some(ContainerStateStatusEnum::RUNNING) => ContainerState::Running,
            Some(ContainerStateStatusEnum::PAUSED) => ContainerState::Paused,
            Some(ContainerStateStatusEnum::EXITED) => ContainerState::Exited,
            Some(ContainerStateStatusEnum::DEAD) => ContainerState::Dead,
            _ => ContainerState::Dead,
        };

        Ok(InspectResult {
            state,
            pid: info.state.as_ref().and_then(|s| s.pid.map(|p| p as u32)),
            ip: info
                .network_settings
                .and_then(|n| n.networks)
                .and_then(|networks| {
                    networks
                        .values()
                        .next()
                        .and_then(|endpoint| endpoint.ip_address.clone())
                }),
            memory_limit: info
                .host_config
                .as_ref()
                .and_then(|h| h.memory.map(|m| m as u64)),
            cpu_quota: info.host_config.as_ref().and_then(|h| {
                h.cpu_quota
                    .zip(h.cpu_period)
                    .map(|(q, p)| q as f64 / p as f64)
            }),
        })
    }

    fn exec(&self, subject: &Subject, cmd: &[String]) -> Result<ExecResult, String> {
        let id = Self::container_id(subject).to_string();
        let cmd: Vec<String> = cmd.to_vec();

        let exec = self
            .block_on(move |conn| {
                let id = id;
                let cmd = cmd;
                async move {
                    let config = CreateExecOptions {
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        cmd: Some(cmd.iter().map(|s| s.as_str()).collect()),
                        ..Default::default()
                    };
                    conn.create_exec(&id, config).await
                }
            })
            .map_err(|e| format!("Create exec failed: {}", e))?;

        let exec_id = exec.id.clone();
        let result = self
            .block_on(|conn| async move {
                conn.start_exec(&exec_id, Some(StartExecOptions::default()))
                    .await
            })
            .map_err(|e| format!("Start exec failed: {}", e))?;

        let (stdout, stderr) = match result {
            StartExecResults::Attached { output, .. } => {
                let entries = self
                    .block_on(|_conn| async move { output.try_collect::<Vec<_>>().await })
                    .map_err(|e| format!("Exec output failed: {}", e))?;

                let mut stdout = String::new();
                let mut stderr = String::new();

                for entry in entries {
                    match entry {
                        LogOutput::StdOut { message } => {
                            stdout.push_str(&String::from_utf8_lossy(&message));
                        }
                        LogOutput::StdErr { message } => {
                            stderr.push_str(&String::from_utf8_lossy(&message));
                        }
                        _ => {}
                    }
                }
                (stdout, stderr)
            }
            StartExecResults::Detached => (String::new(), String::new()),
        };

        let exec_id = exec.id.clone();
        let inspect = self
            .block_on(|conn| async move { conn.inspect_exec(&exec_id).await })
            .map_err(|e| format!("Inspect exec failed: {}", e))?;

        Ok(ExecResult {
            exit_code: inspect.exit_code.unwrap_or(-1) as i32,
            stdout,
            stderr,
        })
    }
}

impl Docker {
    fn container_id(subject: &Subject) -> &str {
        subject.id.strip_prefix("docker/").unwrap_or(&subject.id)
    }

    /// Resolve the host's root filesystem to the physical whole block device
    /// backing it, walking through dm-crypt/LVM/btrfs layers and partitions.
    fn root_block_device() -> Option<String> {
        let source = Self::root_mount_source()?;
        let canonical = fs::canonicalize(&source).ok()?;
        let name = canonical.file_name()?.to_string_lossy().to_string();
        Self::resolve_to_physical(&name)
    }

    /// Extract the source device of the root mount from `/proc/self/mountinfo`.
    /// The mount source sits two fields past the optional-fields separator `-`.
    fn root_mount_source() -> Option<String> {
        let mountinfo = fs::read_to_string("/proc/self/mountinfo").ok()?;
        for line in mountinfo.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() > 4 && fields[4] == "/" {
                let dash = fields.iter().position(|f| *f == "-")?;
                return fields.get(dash + 2).map(|s| s.to_string());
            }
        }
        None
    }

    /// Walk a block device name through slaves (dm-crypt/LVM/RAID) and
    /// partition parents to the underlying physical whole device, returning
    /// its `/dev/<name>` path. Verifies the result is throttleable on cgroup
    /// v2 via `io.stat`, since `io.max` only accepts whole-device major:minor
    /// pairs that appear there.
    fn resolve_to_physical(name: &str) -> Option<String> {
        let mut current = name.to_string();
        loop {
            // If this device is backed by slaves (e.g. dm-0 backed by
            // nvme0n1p2), recurse into the first slave.
            let slaves_dir = format!("/sys/class/block/{}/slaves", current);
            if let Ok(slaves) = fs::read_dir(&slaves_dir)
                && let Some(slave) = slaves.flatten().next()
            {
                current = slave.file_name().to_string_lossy().to_string();
                continue;
            }
            // If this device is a partition, walk to its parent whole device.
            let partition_file = format!("/sys/class/block/{}/partition", current);
            if fs::metadata(&partition_file).is_ok() {
                let parent = fs::canonicalize(format!("/sys/class/block/{}", current))
                    .ok()?
                    .parent()?
                    .file_name()?
                    .to_string_lossy()
                    .to_string();
                current = parent;
                continue;
            }
            break;
        }
        let dev_file = format!("/sys/class/block/{}/dev", current);
        let majmin = fs::read_to_string(dev_file).ok()?.trim().to_string();
        if !Self::in_io_stat(&majmin) {
            debug!(
                "resolved device /dev/{} ({} not in io.stat; not throttleable on cgroup v2)",
                current, majmin
            );
            return None;
        }
        Some(format!("/dev/{}", current))
    }

    /// Check whether a `major:minor` pair appears in `/sys/fs/cgroup/io.stat`.
    fn in_io_stat(majmin: &str) -> bool {
        let Ok(stat) = fs::read_to_string("/sys/fs/cgroup/io.stat") else {
            return false;
        };
        stat.lines()
            .filter_map(|l| l.split_whitespace().next())
            .any(|m| m == majmin)
    }

    /// Fallback: scan `/dev` for the first whole block device matching common
    /// NVMe/SCSI/virtio prefixes, skipping char devices (e.g. `nvme0`) and
    /// partitions. Prefers whole devices, falls back to partitions.
    fn fallback_block_device() -> Option<String> {
        let entries = fs::read_dir("/dev").ok()?;
        let mut whole: Option<String> = None;
        let mut partition: Option<String> = None;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !(name.starts_with("nvme") || name.starts_with("sd") || name.starts_with("vd")) {
                continue;
            }
            // Only real block devices have a sysfs entry under /sys/class/block.
            if fs::metadata(format!("/sys/class/block/{}", name)).is_err() {
                continue;
            }
            if fs::metadata(format!("/sys/class/block/{}/partition", name)).is_ok() {
                partition.get_or_insert(format!("/dev/{}", name));
            } else {
                whole.get_or_insert(format!("/dev/{}", name));
            }
        }
        whole.or(partition)
    }

    fn deprive_resource(&self, subject: &Subject, tier: &crate::fault::Tier) -> Result<(), String> {
        let id = Self::container_id(subject).to_string();

        match tier {
            crate::fault::Tier::Disk => {
                let device = Self::root_block_device()
                    .or_else(Self::fallback_block_device)
                    .ok_or_else(|| {
                        "could not resolve a throttleable block device for disk fault".to_string()
                    })?;

                info!("Throttling disk I/O for container id={} on {}", id, device);
                let update_config = ContainerUpdateBody {
                    blkio_weight: Some(50),
                    blkio_device_read_bps: Some(vec![ThrottleDevice {
                        path: Some(device.clone()),
                        rate: Some(1024 * 1024),
                    }]),
                    blkio_device_write_bps: Some(vec![ThrottleDevice {
                        path: Some(device),
                        rate: Some(1024 * 1024),
                    }]),
                    ..Default::default()
                };
                self.block_on(
                    |conn| async move { conn.update_container(&id, update_config).await },
                )
                .map_err(|e| format!("Failed to throttle disk: {}", e))?;
            }
            crate::fault::Tier::Network => {
                info!("Disconnecting network for container id={}", id);
                let disconnect = NetworkDisconnectRequest {
                    container: id.clone(),
                    force: Some(true),
                };
                match self.block_on(|conn| async move {
                    conn.disconnect_network("bridge", disconnect).await
                }) {
                    Ok(_) => info!("Container disconnected from bridge network"),
                    Err(e) => {
                        warn!(
                            "Failed to disconnect network (may already be disconnected): {}",
                            e
                        );
                    }
                }
            }
            crate::fault::Tier::Memory => {
                let id_for_inspect = id.clone();
                let container_info = self
                    .block_on(|conn| async move {
                        conn.inspect_container(
                            &id_for_inspect,
                            None::<bollard::query_parameters::InspectContainerOptions>,
                        )
                        .await
                    })
                    .map_err(|e| format!("Failed to inspect container: {}", e))?;

                let current_limit = container_info
                    .host_config
                    .and_then(|hc| hc.memory)
                    .unwrap_or(0);

                let new_limit = if current_limit > 0 {
                    (current_limit / 2).max(64 * 1024 * 1024)
                } else {
                    64 * 1024 * 1024
                };

                info!(
                    "Limiting memory for container id={} to {}MB (was {}MB)",
                    id,
                    new_limit / (1024 * 1024),
                    current_limit / (1024 * 1024)
                );

                let update_config = ContainerUpdateBody {
                    memory: Some(new_limit),
                    memory_swap: Some(new_limit),
                    ..Default::default()
                };
                let id_for_update = id.clone();
                self.block_on(|conn| async move {
                    conn.update_container(&id_for_update, update_config).await
                })
                .map_err(|e| format!("Failed to limit memory: {}", e))?;
            }
            crate::fault::Tier::Cpu => {
                info!("Throttling CPU for container id={}", id);
                let update_config = ContainerUpdateBody {
                    cpu_period: Some(100000),
                    cpu_quota: Some(20000),
                    ..Default::default()
                };
                self.block_on(
                    |conn| async move { conn.update_container(&id, update_config).await },
                )
                .map_err(|e| format!("Failed to throttle CPU: {}", e))?;
            }
        }

        Ok(())
    }

    fn restart_if_killed(&self, subject: &Subject) -> Result<(), String> {
        let id = Self::container_id(subject).to_string();
        let id_for_inspect = id.clone();

        match self.block_on(|conn| async move {
            conn.inspect_container(
                &id_for_inspect,
                None::<bollard::query_parameters::InspectContainerOptions>,
            )
            .await
        }) {
            Ok(container) => {
                if let Some(state) = container.state
                    && state.status == Some(ContainerStateStatusEnum::EXITED)
                {
                    info!("Restarting killed container id={}", id);
                    let id_for_restart = id.clone();
                    self.block_on(|conn| async move {
                        conn.restart_container(
                            &id_for_restart,
                            None::<bollard::query_parameters::RestartContainerOptions>,
                        )
                        .await
                    })
                    .map_err(|e| format!("Failed to restart container: {}", e))?;
                }
            }
            Err(e) => {
                debug!("Could not inspect container: {}", e);
            }
        }

        Ok(())
    }

    fn reconnect_network(&self, subject: &Subject) -> Result<(), String> {
        let id = Self::container_id(subject).to_string();

        let connect = NetworkConnectRequest {
            container: id.clone(),
            endpoint_config: None,
        };

        match self.block_on(|conn| async move { conn.connect_network("bridge", connect).await }) {
            Ok(_) => info!("Reconnected container to bridge network"),
            Err(e) => {
                debug!(
                    "Network reconnect skipped (may already be connected): {}",
                    e
                );
            }
        }

        Ok(())
    }

    fn clear_resource_limits(&self, subject: &Subject) -> Result<(), String> {
        let id = Self::container_id(subject).to_string();

        let update_config = ContainerUpdateBody {
            blkio_weight: None,
            memory: None,
            memory_swap: None,
            blkio_device_read_bps: None,
            blkio_device_write_bps: None,
            cpu_period: None,
            cpu_quota: None,
            ..Default::default()
        };

        let id_for_update = id.clone();
        match self.block_on(|conn| async move {
            conn.update_container(&id_for_update, update_config).await
        }) {
            Ok(_) => debug!("Cleared resource limits for container id={}", id),
            Err(BollardError::DockerResponseServerError {
                status_code: 404, ..
            }) => {}
            Err(e) => {
                debug!(
                    "Failed to clear resource limits for container id={} error=\"{}\"",
                    id, e
                )
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_new() {
        assert!(Docker::new().is_ok());
    }

    /// The root block device must resolve to a real, throttleable whole
    /// device: it must exist, be a block device, and its `major:minor` must
    /// appear in `/sys/fs/cgroup/io.stat` (the cgroup v2 throttle list).
    #[test]
    fn test_root_block_device_resolves_to_throttleable_whole_device() {
        let Some(device) = Docker::root_block_device() else {
            return; // host without a resolvable root device (e.g. CI)
        };
        let meta = fs::metadata(&device)
            .unwrap_or_else(|e| panic!("resolved device {device} missing: {e}"));
        use std::os::unix::fs::FileTypeExt;
        assert!(
            meta.file_type().is_block_device(),
            "{device} is not a block device",
        );
        let name = device.strip_prefix("/dev/").unwrap_or(&device);
        assert!(
            fs::metadata(format!("/sys/class/block/{name}/partition")).is_err(),
            "{device} is a partition, not a whole device",
        );
        let majmin = fs::read_to_string(format!("/sys/class/block/{name}/dev"))
            .expect("sysfs dev file")
            .trim()
            .to_string();
        assert!(
            Docker::in_io_stat(&majmin),
            "{device} ({majmin}) not in io.stat — not throttleable on cgroup v2",
        );
    }

    /// `root_mount_source` must find a non-empty source path for the root mount.
    #[test]
    fn test_root_mount_source_present() {
        let source = Docker::root_mount_source();
        assert!(source.is_some(), "no root mount source found in mountinfo");
        let source = source.unwrap();
        assert!(!source.is_empty());
    }
}
