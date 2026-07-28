use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::bindings::context::BindingContext;

pub fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> Result<()> {
    let config = Arc::clone(&ctx.config);
    let state = Arc::clone(&ctx.state);

    let config_fn = lua.create_function(move |lua, tbl: Table| {
        let mut cfg = config.lock().expect("poisoned config lock");

        if let Ok(substrate) = tbl.get::<String>("substrate") {
            cfg.substrate = Some(substrate);
        }

        if let Ok(seed) = tbl.get::<u64>("seed") {
            cfg.seed = Some(seed);
            let mut s = state.lock().expect("poisoned engine state lock");
            s.seed = Some(seed);
            let globals = lua.globals();
            if let Ok(math) = globals.get::<Table>("math") {
                let _ = math.set("randomseed", seed);
            }
        }

        if let Ok(weights) = tbl.get::<Table>("weights") {
            let mut fault_weights = std::collections::HashMap::new();
            for (k, v) in weights.pairs::<String, f32>().flatten() {
                fault_weights.insert(k, v);
            }
            cfg.fault_weights = fault_weights;
        }

        if let Ok(mode) = tbl.get::<String>("accumulation") {
            cfg.accumulation_mode = mode
                .parse()
                .map_err(|e: String| mlua::Error::RuntimeError(e))?;
        }

        if let Ok(timeout) = tbl.get::<u64>("http_timeout") {
            cfg.http_timeout_secs = timeout;
        }

        if let Ok(retries) = tbl.get::<u32>("http_retries") {
            cfg.http_retries = retries;
        }

        if let Ok(delay) = tbl.get::<u64>("http_retry_delay") {
            cfg.http_retry_delay_ms = delay;
        }

        if let Ok(delay) = tbl.get::<u64>("step_delay") {
            cfg.step_delay_ms = delay;
        }

        if let Ok(require) = tbl.get::<bool>("require_seed") {
            cfg.require_seed = require;
        }

        cfg.normalize_weights();

        cfg.validate()
            .map_err(|e| mlua::Error::RuntimeError(format!("invalid configuration: {}", e)))?;

        Ok(())
    })?;

    dstest.set("config", config_fn)?;
    Ok(())
}
