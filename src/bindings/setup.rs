use std::collections::HashMap;
use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::bindings::context::BindingContext;
use crate::substrate::docker::DockerSubjectData;

pub fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> Result<()> {
    let state = Arc::clone(&ctx.state);
    let docker = Arc::clone(&ctx.docker);
    let config = Arc::clone(&ctx.config);

    let setup =
        lua.create_function(
            move |_, (substrate_type, config_tbl): (Option<String>, Table)| {
                let cfg = config.lock().expect("poisoned config lock");
                let substrate = substrate_type
                    .or_else(|| cfg.substrate.clone())
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(
                            "no substrate configured: call dstest.config({ substrate = \"docker\" }) first".to_string(),
                        )
                    })?;

                match substrate.as_str() {
                    "docker" => {
                        let image: String = config_tbl.get("image")?;
                        let ports: Option<Vec<u16>> = config_tbl.get("ports").ok();
                        let cmd: Option<Vec<String>> = config_tbl.get("cmd").ok();
                        let volumes: Option<Vec<String>> = config_tbl.get("volumes").ok();
                        let env: Option<HashMap<String, String>> = config_tbl.get("env").ok();
                        let env = env
                            .map(|e| e.into_iter().map(|(k, v)| format!("{}={}", k, v)).collect());

                        let data = DockerSubjectData {
                            image,
                            cmd,
                            ports,
                            volumes,
                            env,
                        };

                        let container_id =
                            docker.host(&data).map_err(mlua::Error::RuntimeError)?;

                        let mut state = state.lock().expect("poisoned engine state lock");

                        if let Some(ref p) = data.ports.as_ref().and_then(|ports| ports.first()) {
                            state.subject_hosts.insert(
                                format!("docker/{}", container_id),
                                format!("localhost:{}", p),
                            );
                        }

                        state
                            .subjects
                            .push((format!("docker/{}", container_id), data));

                        Ok(format!("docker/{}", container_id))
                    }
                    _ => Err(mlua::Error::RuntimeError(format!(
                        "unknown substrate type: {}",
                        substrate
                    ))),
                }
            },
        )?;

    dstest.set("setup", setup)?;
    Ok(())
}
