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

/// Render a substrate-specific value onto a Lua value. Each substrate owns
/// the shape of its `inspect` result (and any other associated type it
/// surfaces to Lua) and implements this trait so the bindings stay generic
/// over `S: Substrate`.
pub trait ToLua {
    fn to_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value>;
}

pub trait Substrate: Send + Sync + 'static {
    /// Human-readable substrate name (e.g. `"docker"`), matched against the
    /// `substrate` field of `dstest.config()`.
    const NAME: &'static str;

    /// Substrate-specific data describing how to host a subject.
    type SubjectData: Clone + Send + Sync + 'static;

    /// Substrate-specific inspect result, surfaced to Lua via [`ToLua`].
    type Inspect: ToLua + Send + 'static;

    /// Substrate-specific log-query options, parsed from the optional Lua
    /// table passed to `dstest.logs`.
    type LogOpts: Default + Send + Sync + 'static;

    /// Parse the Lua table from `dstest.setup()` into this substrate's
    /// `SubjectData`. Each substrate owns its own config schema — Docker
    /// reads `image`/`ports`/`volumes`/`env`/`cmd`, other substrates can
    /// read whatever fields they need.
    fn parse_subject(&self, table: &mlua::Table) -> Result<Self::SubjectData, String>;

    /// Parse the optional Lua table from `dstest.logs` into this
    /// substrate's `LogOpts`. `None` means no options table was passed;
    /// the substrate should fall back to [`Default`].
    fn parse_log_opts(&self, table: Option<&mlua::Table>) -> Result<Self::LogOpts, String>;

    /// Pull/create/start a subject and return its instance id plus an
    /// optional reachable address.
    fn host(&self, data: &Self::SubjectData) -> Result<HostedSubject, String>;

    fn affect(&self, subject: &Subject, fault: &Fault) -> Result<(), String>;
    fn clear_faults(&self, subject: &Subject) -> Result<(), String>;
    fn teardown(&self, subject: Subject) -> Result<(), String>;

    fn logs(&self, subject: &Subject, opts: Self::LogOpts) -> Result<Vec<LogEntry>, String>;
    fn inspect(&self, subject: &Subject) -> Result<Self::Inspect, String>;
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
