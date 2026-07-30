use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use mlua::Lua;

#[derive(Clone, Debug)]
pub struct OracleFailure {
    pub check_type: String,
    pub name: String,
    pub round: Option<usize>,
    pub fault: Option<String>,
    pub subject: Option<String>,
    pub error: String,
}

#[derive(Clone, Debug)]
pub struct OracleReport {
    pub passed: bool,
    pub total_checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub failures: Vec<OracleFailure>,
}

impl Default for OracleReport {
    fn default() -> Self {
        Self {
            passed: true,
            total_checks: 0,
            passed_checks: 0,
            failed_checks: 0,
            failures: Vec::new(),
        }
    }
}

impl OracleReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_failure(&mut self, failure: OracleFailure) {
        self.failures.push(failure);
        self.failed_checks += 1;
        self.total_checks += 1;
        self.passed = false;
    }

    pub fn add_pass(&mut self) {
        self.passed_checks += 1;
        self.total_checks += 1;
    }

    pub fn merge(&mut self, other: &OracleReport) {
        self.total_checks += other.total_checks;
        self.passed_checks += other.passed_checks;
        self.failed_checks += other.failed_checks;
        self.failures.extend(other.failures.clone());
        if !other.passed {
            self.passed = false;
        }
    }
}

type PredicateFut<'a> =
    Pin<Box<dyn Future<Output = Result<(bool, Option<String>), mlua::Error>> + 'a>>;
type InvariantFut<'a> =
    Pin<Box<dyn Future<Output = Result<(bool, Option<String>), mlua::Error>> + 'a>>;
pub type PredicateFn =
    Box<dyn for<'a> Fn(&'a Lua, String, String, usize) -> PredicateFut<'a> + Send + Sync>;
pub type InvariantFn = Box<dyn for<'a> Fn(&'a Lua) -> InvariantFut<'a> + Send + Sync>;

pub struct Oracle {
    pub predicates: HashMap<String, PredicateFn>,
    pub invariants: HashMap<String, InvariantFn>,
    pub enabled: bool,
    pub report: OracleReport,
}

impl Oracle {
    pub fn new() -> Self {
        Self {
            predicates: HashMap::new(),
            invariants: HashMap::new(),
            enabled: false,
            report: OracleReport::new(),
        }
    }

    pub fn add_predicate(&mut self, name: String, func: PredicateFn) {
        self.predicates.insert(name, func);
    }

    pub fn add_invariant(&mut self, name: String, func: InvariantFn) {
        self.invariants.insert(name, func);
    }

    pub async fn check_predicates(
        &mut self,
        lua: &Lua,
        subject_id: &str,
        fault: &str,
        round: usize,
    ) -> OracleReport {
        let mut report = OracleReport::new();
        report.passed = true;

        for (name, predicate) in &self.predicates {
            match predicate(lua, subject_id.to_string(), fault.to_string(), round).await {
                Ok((true, _)) => {
                    report.add_pass();
                }
                Ok((false, Some(msg))) => {
                    report.add_failure(OracleFailure {
                        check_type: "predicate".to_string(),
                        name: name.clone(),
                        round: Some(round),
                        fault: Some(fault.to_string()),
                        subject: Some(subject_id.to_string()),
                        error: msg,
                    });
                }
                Ok((false, None)) => {
                    report.add_failure(OracleFailure {
                        check_type: "predicate".to_string(),
                        name: name.clone(),
                        round: Some(round),
                        fault: Some(fault.to_string()),
                        subject: Some(subject_id.to_string()),
                        error: "predicate returned false".to_string(),
                    });
                }
                Err(e) => {
                    report.add_failure(OracleFailure {
                        check_type: "predicate".to_string(),
                        name: name.clone(),
                        round: Some(round),
                        fault: Some(fault.to_string()),
                        subject: Some(subject_id.to_string()),
                        error: format!("predicate error: {}", e),
                    });
                }
            }
        }

        report
    }

    pub async fn check_invariants(&mut self, lua: &Lua) -> OracleReport {
        let mut report = OracleReport::new();
        report.passed = true;

        for (name, invariant) in &self.invariants {
            match invariant(lua).await {
                Ok((true, _)) => {
                    report.add_pass();
                }
                Ok((false, Some(msg))) => {
                    report.add_failure(OracleFailure {
                        check_type: "invariant".to_string(),
                        name: name.clone(),
                        round: None,
                        fault: None,
                        subject: None,
                        error: msg,
                    });
                }
                Ok((false, None)) => {
                    report.add_failure(OracleFailure {
                        check_type: "invariant".to_string(),
                        name: name.clone(),
                        round: None,
                        fault: None,
                        subject: None,
                        error: "invariant returned false".to_string(),
                    });
                }
                Err(e) => {
                    report.add_failure(OracleFailure {
                        check_type: "invariant".to_string(),
                        name: name.clone(),
                        round: None,
                        fault: None,
                        subject: None,
                        error: format!("invariant error: {}", e),
                    });
                }
            }
        }

        report
    }

    pub async fn check_all(
        &mut self,
        lua: &Lua,
        subject_id: &str,
        fault: &str,
        round: usize,
    ) -> OracleReport {
        let mut report = OracleReport::new();
        report.passed = true;

        let pred_report = self.check_predicates(lua, subject_id, fault, round).await;
        report.merge(&pred_report);

        let inv_report = self.check_invariants(lua).await;
        report.merge(&inv_report);

        report
    }

    pub fn reset(&mut self) {
        self.report = OracleReport::new();
    }
}

impl Default for Oracle {
    fn default() -> Self {
        Self::new()
    }
}

pub type OracleRef = Arc<Mutex<Oracle>>;
