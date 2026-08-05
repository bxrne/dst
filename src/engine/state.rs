use std::collections::BTreeMap;

use crate::config::Config;
use crate::fault::Fault;
use crate::substrate::Substrate;

/// A hosted subject and everything the engine needs to remember about it.
pub struct SubjectRecord<D> {
    /// Fully-qualified subject id, e.g. `"docker/<container_id>"`.
    pub id: String,
    /// Engine-assigned name passed to the substrate (e.g. container name).
    pub name: String,
    /// Handle of the `dstest.config` this subject was created under.
    pub config: String,
    /// Substrate-specific setup data, retained for future re-host semantics
    /// (e.g. clear-by-recreate when a fault cannot be undone in place).
    #[allow(dead_code)]
    pub data: D,
    /// Faults currently applied to this subject (for observability and
    /// future fault TTLs).
    pub active_faults: Vec<Fault>,
}

pub struct EngineState<S: Substrate> {
    pub subjects: Vec<SubjectRecord<S::SubjectData>>,
    pub subject_hosts: BTreeMap<String, String>,
    /// Named experiment configs, keyed by the handle returned from
    /// `dstest.config`. BTreeMap keeps iteration order deterministic.
    pub configs: BTreeMap<String, Config>,
    /// Monotonic counter for generating unique subject names.
    pub name_counter: usize,
}

impl<S: Substrate> Default for EngineState<S> {
    fn default() -> Self {
        Self {
            subjects: Vec::new(),
            subject_hosts: BTreeMap::new(),
            configs: BTreeMap::new(),
            name_counter: 0,
        }
    }
}
