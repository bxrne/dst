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

pub trait Substrate {
    fn affect(&self, subject: &Subject, fault: &Fault) -> Result<(), String>;
    fn clear_faults(&self, subject: &Subject) -> Result<(), String>;
    fn teardown(&self, subject: Subject) -> Result<(), String>;

    fn logs(&self, subject: &Subject, opts: LogOptions) -> Result<Vec<LogEntry>, String>;
    fn inspect(&self, subject: &Subject) -> Result<InspectResult, String>;
    fn exec(&self, subject: &Subject, cmd: &[String]) -> Result<ExecResult, String>;
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
