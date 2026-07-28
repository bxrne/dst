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

pub trait Substrate {
    fn affect(&self, subject: &Subject, fault: &Fault) -> Result<(), String>;
    fn clear_faults(&self, subject: &Subject) -> Result<(), String>;
    fn teardown(&self, subject: Subject) -> Result<(), String>;
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
