use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use mlua::Lua;

use crate::fault::FaultTree;
use crate::oracle::OracleRef;
use crate::substrate::Substrate;

use super::state::EngineState;

pub struct BindingContext<S: Substrate> {
    pub state: Arc<Mutex<EngineState<S>>>,
    pub fault_trees: Arc<Mutex<BTreeMap<String, FaultTree>>>,
    pub oracle: OracleRef,
    pub substrate: Arc<S>,
    /// Seeded workload RNG for `dstest.random.*` (separate stream from the
    /// fault tree's RNG, so workload draws don't affect the fault schedule).
    pub workload_rng: Arc<Mutex<Option<rand::rngs::StdRng>>>,
    pub lua: Lua,
}

impl<S: Substrate> BindingContext<S> {
    pub fn new(substrate: S) -> Self {
        let substrate = Arc::new(substrate);
        Self {
            state: Arc::new(Mutex::new(EngineState::default())),
            fault_trees: Arc::new(Mutex::new(BTreeMap::new())),
            oracle: Arc::new(Mutex::new(crate::oracle::Oracle::new())),
            substrate,
            workload_rng: Arc::new(Mutex::new(None)),
            lua: Lua::new(),
        }
    }
}
