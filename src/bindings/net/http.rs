use std::sync::Arc;
use std::time::Duration;

use mlua::{Lua, Result, Table};

use crate::engine::context::BindingContext;
use crate::engine::state::EngineState;
use crate::substrate::Substrate;

/// Resolve a subject id to its reachable address plus the HTTP settings of
/// the config it was created under.
pub fn resolve_subject_http<S: Substrate>(
    state: &EngineState<S>,
    id: &str,
) -> mlua::Result<(String, u64, u32, u64)> {
    let rec = state
        .subjects
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| mlua::Error::RuntimeError(format!("unknown subject {}", id)))?;
    let cfg = state.configs.get(&rec.config).ok_or_else(|| {
        mlua::Error::RuntimeError(format!(
            "subject {} has unknown config '{}'",
            id, rec.config
        ))
    })?;
    let host = state
        .subject_hosts
        .get(id)
        .cloned()
        .ok_or_else(|| mlua::Error::RuntimeError(format!("subject {} has no address", id)))?;
    Ok((
        host,
        cfg.http_timeout_secs,
        cfg.http_retries,
        cfg.http_retry_delay_ms,
    ))
}

pub fn register<S: Substrate>(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> Result<()> {
    let state = Arc::clone(&ctx.state);

    let http_fn =
        lua.create_async_function(move |lua, (id, method, path): (String, String, String)| {
            let state = Arc::clone(&state);

            async move {
                let (host, timeout, retries, delay) = {
                    let state = state.lock().expect("poisoned engine state lock");
                    resolve_subject_http(&state, &id)?
                };

                let url = format!("http://{host}{path}");
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(timeout))
                    .build()
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!("failed to create HTTP client: {}", e))
                    })?;

                let req_method: reqwest::Method = method
                    .parse()
                    .map_err(|e| mlua::Error::RuntimeError(format!("invalid method: {}", e)))?;

                let mut last_err = None;

                for attempt in 0..retries {
                    match client.request(req_method.clone(), &url).send().await {
                        Ok(resp) => {
                            let status = resp.status().as_u16();
                            let body = resp.text().await.map_err(|e| {
                                mlua::Error::RuntimeError(format!("body read failed: {}", e))
                            })?;
                            let t = lua.create_table()?;
                            t.set("status", status)?;
                            t.set("body", body)?;
                            return Ok(t);
                        }
                        Err(e) => {
                            last_err = Some(e);
                            if attempt < retries - 1 {
                                tokio::time::sleep(Duration::from_millis(delay)).await;
                            }
                        }
                    }
                }

                let e = last_err.unwrap();
                Err(mlua::Error::RuntimeError(format!(
                    "HTTP failed after {} retries: {}",
                    retries, e
                )))
            }
        })?;

    dstest.set("http", http_fn)?;
    Ok(())
}
