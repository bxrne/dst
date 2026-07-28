use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::bindings::context::BindingContext;

pub fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> Result<()> {
    let state = Arc::clone(&ctx.state);
    let config = Arc::clone(&ctx.config);

    let http_fn =
        lua.create_function(move |lua, (id, method, path): (String, String, String)| {
            let (timeout, retries, delay) = {
                let cfg = config.lock().expect("poisoned config lock");
                (
                    cfg.http_timeout_secs,
                    cfg.http_retries,
                    cfg.http_retry_delay_ms,
                )
            };

            let host =
                {
                    let state = state.lock().expect("poisoned engine state lock");
                    state.subject_hosts.get(&id).cloned().ok_or_else(|| {
                        mlua::Error::RuntimeError(format!("unknown subject {}", id))
                    })?
                };

            let url = format!("http://{host}{path}");
            let client = BindingContext::http_client_with_timeout(timeout);

            let req_method: reqwest::Method = method
                .parse()
                .map_err(|e| mlua::Error::RuntimeError(format!("invalid method: {}", e)))?;

            let mut last_err = None;

            for attempt in 0..retries {
                match client.request(req_method.clone(), &url).send() {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        let body = resp.text().map_err(|e| {
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
                            std::thread::sleep(std::time::Duration::from_millis(delay));
                        }
                    }
                }
            }

            let e = last_err.unwrap();
            Err(mlua::Error::RuntimeError(format!(
                "HTTP failed after {} retries: {}",
                retries, e
            )))
        })?;

    dstest.set("http", http_fn)?;
    Ok(())
}
