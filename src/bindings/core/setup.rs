use std::sync::Arc;
use std::time::Duration;

use mlua::{Lua, Result, Table};
use tracing::debug;

use crate::engine::context::BindingContext;
use crate::engine::state::SubjectRecord;
use crate::substrate::Substrate;

/// Default time to wait for a dependency to become ready (60 × 500ms = 30s).
const DEP_WAIT_ATTEMPTS: usize = 120;
const DEP_WAIT_INTERVAL: Duration = Duration::from_millis(500);

/// Map a config handle to a valid resource name fragment (Docker container
/// names accept [a-zA-Z0-9_.-]; other substrates have similar rules).
fn sanitize_name(handle: &str) -> String {
    handle
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

pub fn register<S: Substrate>(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> Result<()> {
    let state = Arc::clone(&ctx.state);
    let substrate = Arc::clone(&ctx.substrate);

    let setup =
        lua.create_async_function(move |_, (handle, config_tbl): (String, Table)| {
            let state = Arc::clone(&state);
            let substrate = Arc::clone(&substrate);

            async move {
                let name = {
                    let mut state = state.lock().expect("poisoned engine state lock");

                    let cfg = state.configs.get(&handle).ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "unknown config '{}' — pass the handle returned by dstest.config()",
                            handle
                        ))
                    })?;

                    if cfg.substrate.as_deref() != Some(S::NAME) {
                        return Err(mlua::Error::RuntimeError(format!(
                            "substrate mismatch: config '{}' wants \"{}\" but the engine was built for \"{}\"",
                            handle,
                            cfg.substrate.as_deref().unwrap_or("<none>"),
                            S::NAME
                        )));
                    }

                    state.name_counter += 1;
                    format!(
                        "dstest-{}-{}",
                        sanitize_name(&handle),
                        state.name_counter
                    )
                };

                // Wait for dependencies to be ready before hosting.
                let depends: Vec<String> = config_tbl.get("depends").unwrap_or_default();
                for dep_id in &depends {
                    let dep_addr = {
                        let state = state.lock().expect("poisoned engine state lock");
                        state.subject_hosts.get(dep_id).cloned()
                    };
                    if let Some(addr) = dep_addr {
                        let mut ready = false;
                        for _attempt in 0..DEP_WAIT_ATTEMPTS {
                            if tokio::net::TcpStream::connect(&addr).await.is_ok() {
                                ready = true;
                                break;
                            }
                            tokio::time::sleep(DEP_WAIT_INTERVAL).await;
                        }
                        if !ready {
                            return Err(mlua::Error::RuntimeError(format!(
                                "dependency {} not ready after {}s (polling {})",
                                dep_id,
                                DEP_WAIT_ATTEMPTS * DEP_WAIT_INTERVAL.as_millis() as usize / 1000,
                                addr
                            )));
                        }
                        debug!("dependency {} ready at {}", dep_id, addr);
                    } else {
                        debug!(
                            "dependency {} has no address; skipping readiness wait",
                            dep_id
                        );
                    }
                }

                let data = substrate
                    .parse_subject(&config_tbl)
                    .map_err(mlua::Error::RuntimeError)?;

                let hosted = substrate
                    .host(&name, &data)
                    .await
                    .map_err(mlua::Error::RuntimeError)?;

                let subject_id = format!("{}/{}", S::NAME, hosted.id);

                let mut state = state.lock().expect("poisoned engine state lock");

                if let Some(addr) = hosted.addr {
                    state.subject_hosts.insert(subject_id.clone(), addr);
                }

                state.subjects.push(SubjectRecord {
                    id: subject_id.clone(),
                    name,
                    config: handle,
                    data,
                    active_faults: Vec::new(),
                });

                Ok(subject_id)
            }
        })?;

    dstest.set("setup", setup)?;
    Ok(())
}
