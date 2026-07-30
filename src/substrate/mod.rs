use crate::fault::Fault;
use std::fmt::{self, Display};

pub mod docker;

pub struct Subject {
    pub id: String,
}

impl Subject {
    pub fn new(id: String) -> Self {
        Subject { id }
    }
}

impl Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Subject {{ id: {} }}", self.id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    StdOut,
    StdErr,
}

impl Display for Stream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stream::StdOut => write!(f, "stdout"),
            Stream::StdErr => write!(f, "stderr"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub stream: Stream,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerState {
    Running,
    Paused,
    Exited,
    Dead,
}

impl Display for ContainerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerState::Running => write!(f, "running"),
            ContainerState::Paused => write!(f, "paused"),
            ContainerState::Exited => write!(f, "exited"),
            ContainerState::Dead => write!(f, "dead"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InspectResult {
    pub state: ContainerState,
    pub pid: Option<u32>,
    pub ip: Option<String>,
    pub memory_limit: Option<u64>,
    pub cpu_quota: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// The result of hosting a subject: its instance id and an optional
/// reachable address (e.g. `"localhost:8080"`). Each substrate decides
/// what address format makes sense — Docker returns a host port mapping,
/// a future k8s substrate could return a service DNS name.
pub struct HostedSubject {
    pub id: String,
    pub addr: Option<String>,
}

pub trait Substrate: Send + Sync + 'static {
    /// Human-readable substrate name (e.g. `"docker"`), matched against the
    /// `substrate` field of `dstest.config()`.
    const NAME: &'static str;

    /// Substrate-specific data describing how to host a subject.
    type SubjectData: Clone + Send + Sync + 'static;

    /// Parse the Lua table from `dstest.setup()` into this substrate's
    /// `SubjectData`. Each substrate owns its own config schema — Docker
    /// reads `image`/`ports`/`volumes`/`env`/`cmd`, other substrates can
    /// read whatever fields they need.
    fn parse_subject(&self, table: &mlua::Table) -> Result<Self::SubjectData, String>;

    /// Pull/create/start a subject and return its instance id plus an
    /// optional reachable address.
    fn host(&self, data: &Self::SubjectData) -> Result<HostedSubject, String>;

    fn affect(&self, subject: &Subject, fault: &Fault) -> Result<(), String>;
    fn clear_faults(&self, subject: &Subject) -> Result<(), String>;
    fn teardown(&self, subject: Subject) -> Result<(), String>;

    fn logs(&self, subject: &Subject, opts: LogOptions) -> Result<Vec<LogEntry>, String>;
    fn inspect(&self, subject: &Subject) -> Result<InspectResult, String>;
    fn exec(&self, subject: &Subject, cmd: &[String]) -> Result<ExecResult, String>;
}

#[derive(Clone, Debug)]
pub struct LogOptions {
    pub stdout: bool,
    pub stderr: bool,
    pub tail: Option<String>,
    pub since: Option<i32>,
    pub timestamps: bool,
}

impl Default for LogOptions {
    fn default() -> Self {
        Self {
            stdout: true,
            stderr: true,
            tail: None,
            since: None,
            timestamps: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_new() {
        let subject = Subject::new("docker/abc123".to_string());
        assert_eq!(subject.id, "docker/abc123");
    }

    #[test]
    fn test_subject_display() {
        let subject = Subject::new("docker/abc123".to_string());
        assert_eq!(format!("{subject}"), "Subject { id: docker/abc123 }");
    }
}
