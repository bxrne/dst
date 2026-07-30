use std::sync::{Arc, Mutex};

use mlua::Lua;

use crate::config::Config;
use crate::fault::FaultTree;
use crate::oracle::OracleRef;
use crate::substrate::docker::DockerSubjectData;
use crate::substrate::{Substrate, docker::Docker};

#[derive(Default)]
pub struct EngineState {
    pub subjects: Vec<(String, DockerSubjectData)>,
    pub subject_hosts: std::collections::HashMap<String, String>,
    pub seed: Option<u64>,
}

pub struct BindingContext {
    pub state: Arc<Mutex<EngineState>>,
    pub config: Arc<Mutex<Config>>,
    pub fault_tree: Arc<Mutex<Option<FaultTree>>>,
    pub oracle: OracleRef,
    pub substrate: Arc<dyn Substrate>,
    pub docker: Arc<Docker>,
    pub lua: Lua,
}

impl BindingContext {
    pub fn new() -> Self {
        let docker = Arc::new(Docker::new().expect("failed to connect to Docker daemon"));
        let substrate = Arc::clone(&docker) as Arc<dyn Substrate>;
        Self {
            state: Arc::new(Mutex::new(EngineState::default())),
            config: Arc::new(Mutex::new(Config::default())),
            fault_tree: Arc::new(Mutex::new(None)),
            oracle: Arc::new(Mutex::new(crate::oracle::Oracle::new())),
            substrate,
            docker,
            lua: Lua::new(),
        }
    }

    pub fn http_client_with_timeout(timeout_secs: u64) -> reqwest::blocking::Client {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .expect("failed to create HTTP client")
    }
}

impl Default for BindingContext {
    fn default() -> Self {
        Self::new()
    }
}
