use std::sync::{Arc, Mutex};

use mlua::Lua;

use crate::config::Config;
use crate::fault::FaultTree;
use crate::oracle::OracleRef;
use crate::substrate::Substrate;

use super::state::EngineState;

pub struct BindingContext<S: Substrate> {
    pub state: Arc<Mutex<EngineState<S>>>,
    pub config: Arc<Mutex<Config>>,
    pub fault_tree: Arc<Mutex<Option<FaultTree>>>,
    pub oracle: OracleRef,
    pub substrate: Arc<S>,
    pub lua: Lua,
}

impl<S: Substrate> BindingContext<S> {
    pub fn new(substrate: S) -> Self {
        let substrate = Arc::new(substrate);
        Self {
            state: Arc::new(Mutex::new(EngineState::default())),
            config: Arc::new(Mutex::new(Config::default())),
            fault_tree: Arc::new(Mutex::new(None)),
            oracle: Arc::new(Mutex::new(crate::oracle::Oracle::new())),
            substrate,
            lua: Lua::new(),
        }
    }
}
