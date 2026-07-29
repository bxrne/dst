use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;

use bollard::Docker as BollardDocker;
use bollard::errors::Error as BollardError;
use bollard::models::{
    ContainerCreateBody, ContainerStateStatusEnum, ContainerUpdateBody, HostConfig,
    NetworkConnectRequest, NetworkDisconnectRequest, PortBinding, ThrottleDevice,
};
use bollard::query_parameters::CreateImageOptions;
use bollard::query_parameters::{
    CreateContainerOptions, RemoveContainerOptions, StartContainerOptions,
};
use futures_util::TryStreamExt;
use tracing::{debug, info, warn};

use crate::substrate::{Fault, Subject, Substrate};

fn runtime() -> &'static tokio::runtime::Runtime {
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

    pub fn host(&self, data: &DockerSubjectData) -> Result<String, String> {
        let rt = runtime();

        rt.block_on(
            self.connection
                .create_image(
                    Some(CreateImageOptions {
                        from_image: Some(data.image.clone()),
                        ..Default::default()
                    }),
                    None,
                    None,
                )
                .try_collect::<Vec<_>>(),
        )
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

        let container = rt
            .block_on(
                self.connection
                    .create_container(None::<CreateContainerOptions>, container_config),
            )
            .map_err(|e| format!("Failed to create container: {}", e))?;

        rt.block_on(
            self.connection
                .start_container(&container.id, None::<StartContainerOptions>),
        )
        .map_err(|e| format!("Failed to start container: {}", e))?;

        info!("Started container id={}", container.id);
        Ok(container.id.clone())
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
    fn affect(&self, subject: &Subject, fault: &Fault) -> Result<(), String> {
        let rt = runtime();
        let id = Self::container_id(subject);

        match fault {
            Fault::Pause => match rt.block_on(self.connection.pause_container(id)) {
                Ok(_) => info!("Paused container id={}", id),
                Err(BollardError::DockerResponseServerError {
                    status_code: 409, ..
                }) => {
                    debug!("Container id={} already paused", id);
                }
                Err(e) => return Err(format!("Failed to pause container {}: {}", id, e)),
            },
            Fault::Kill => match rt.block_on(self.connection.kill_container(id, None)) {
                Ok(_) => info!("Killed container id={}", id),
                Err(BollardError::DockerResponseServerError {
                    status_code: 409, ..
                }) => {
                    debug!("Container id={} not running", id);
                }
                Err(e) => return Err(format!("Failed to kill container {}: {}", id, e)),
            },
            Fault::Deprive(tier) => {
                info!("Depriving container id={} tier={}", id, tier);
                self.deprive_resource(subject, tier)?;
            }
        }
        Ok(())
    }

    fn clear_faults(&self, subject: &Subject) -> Result<(), String> {
        let rt = runtime();
        let id = Self::container_id(subject);
        info!("Clearing faults id={}", id);

        match rt.block_on(self.connection.unpause_container(id)) {
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
        let id = Self::container_id(&subject);
        info!("Tearing down container id={}", id);

        let rt = runtime();
        rt.block_on(self.connection.stop_container(id, None))
            .map_err(|e| format!("Failed to stop container: {}", e))?;

        let options = RemoveContainerOptions {
            v: true,
            force: true,
            link: false,
        };
        rt.block_on(self.connection.remove_container(id, Some(options)))
            .map_err(|e| format!("Failed to remove container: {}", e))?;

        Ok(())
    }
}

impl Docker {
    fn container_id(subject: &Subject) -> &str {
        subject.id.strip_prefix("docker/").unwrap_or(&subject.id)
    }

    fn root_block_device() -> Option<String> {
        let mountinfo = fs::read_to_string("/proc/self/mountinfo").ok()?;
        for line in mountinfo.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() > 4 && fields[4] == "/" {
                let dev = fields.get(2)?;
                let (major, _minor) = dev.split_once(':')?;
                let major: i32 = major.parse().ok()?;
                let partitions = fs::read_to_string("/proc/partitions").ok()?;
                for pline in partitions.lines().skip(2) {
                    let pfields: Vec<&str> = pline.split_whitespace().collect();
                    if pfields.len() >= 4 {
                        let pmajor: i32 = pfields[0].parse().ok()?;
                        if pmajor == major {
                            let name = pfields[3];
                            return Some(format!("/dev/{}", name));
                        }
                    }
                }
            }
        }
        None
    }

    fn deprive_resource(&self, subject: &Subject, tier: &crate::fault::Tier) -> Result<(), String> {
        let rt = runtime();
        let id = Self::container_id(subject);

        match tier {
            crate::fault::Tier::Disk => {
                let device = Self::root_block_device()
                    .or_else(|| {
                        fs::read_dir("/dev").ok().and_then(|entries| {
                            for entry in entries.flatten() {
                                let name = entry.file_name().to_string_lossy().to_string();
                                if name.starts_with("nvme")
                                    || name.starts_with("sd")
                                    || name.starts_with("vd")
                                {
                                    return Some(format!("/dev/{}", name));
                                }
                            }
                            None
                        })
                    })
                    .unwrap_or_else(|| "/dev/sda".to_string());

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
                rt.block_on(self.connection.update_container(id, update_config))
                    .map_err(|e| format!("Failed to throttle disk: {}", e))?;
            }
            crate::fault::Tier::Network => {
                info!("Disconnecting network for container id={}", id);
                let disconnect = NetworkDisconnectRequest {
                    container: id.to_string(),
                    force: Some(true),
                };
                match rt.block_on(self.connection.disconnect_network("bridge", disconnect)) {
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
                let container_info = rt
                    .block_on(self.connection.inspect_container(
                        id,
                        None::<bollard::query_parameters::InspectContainerOptions>,
                    ))
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
                rt.block_on(self.connection.update_container(id, update_config))
                    .map_err(|e| format!("Failed to limit memory: {}", e))?;
            }
            crate::fault::Tier::Cpu => {
                info!("Throttling CPU for container id={}", id);
                let update_config = ContainerUpdateBody {
                    cpu_period: Some(100000),
                    cpu_quota: Some(20000),
                    ..Default::default()
                };
                rt.block_on(self.connection.update_container(id, update_config))
                    .map_err(|e| format!("Failed to throttle CPU: {}", e))?;
            }
        }

        Ok(())
    }

    fn restart_if_killed(&self, subject: &Subject) -> Result<(), String> {
        let rt = runtime();
        let id = Self::container_id(subject);

        match rt.block_on(self.connection.inspect_container(
            id,
            None::<bollard::query_parameters::InspectContainerOptions>,
        )) {
            Ok(container) => {
                if let Some(state) = container.state
                    && state.status == Some(ContainerStateStatusEnum::EXITED)
                {
                    info!("Restarting killed container id={}", id);
                    rt.block_on(self.connection.restart_container(
                        id,
                        None::<bollard::query_parameters::RestartContainerOptions>,
                    ))
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
        let rt = runtime();
        let id = Self::container_id(subject);

        let connect = NetworkConnectRequest {
            container: id.to_string(),
            endpoint_config: None,
        };

        match rt.block_on(self.connection.connect_network("bridge", connect)) {
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
        let rt = runtime();
        let id = Self::container_id(subject);

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

        match rt.block_on(self.connection.update_container(id, update_config)) {
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
}
