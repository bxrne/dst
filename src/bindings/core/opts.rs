use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::config::Config;
use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

pub fn register<S: Substrate>(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> Result<()> {
    let state = Arc::clone(&ctx.state);

    let config_fn = lua.create_function(move |lua, tbl: Table| {
        let mut cfg = Config::default();

        let name: Option<String> = tbl.get("name").ok();

        if let Ok(substrate) = tbl.get::<String>("substrate") {
            cfg.substrate = Some(substrate);
        }

        if let Ok(seed) = tbl.get::<u64>("seed") {
            cfg.seed = Some(seed);
            // Seed Lua's own PRNG too, so math.random() is reproducible.
            let globals = lua.globals();
            let math: Table = globals.get("math")?;
            let randomseed: mlua::Function = math.get("randomseed")?;
            randomseed.call::<()>(seed)?;
        }

        if let Ok(weights) = tbl.get::<Table>("weights") {
            let mut fault_weights = std::collections::BTreeMap::new();
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

        if let Ok(steps) = tbl.get::<usize>("steps") {
            cfg.steps = steps;
        }

        if let Ok(require) = tbl.get::<bool>("require_seed") {
            cfg.require_seed = require;
        }

        if cfg.substrate.is_none() {
            return Err(mlua::Error::RuntimeError(
                "dstest.config requires a `substrate` field".to_string(),
            ));
        }

        cfg.validate()
            .map_err(|e| mlua::Error::RuntimeError(format!("invalid configuration: {}", e)))?;
        cfg.normalize_weights();

        let mut state = state.lock().expect("poisoned engine state lock");

        // Auto-generate a unique handle when none was given.
        let handle = match name {
            Some(n) => n,
            None => {
                let mut n = state.configs.len() + 1;
                loop {
                    let candidate = format!("config_{}", n);
                    if !state.configs.contains_key(&candidate) {
                        break candidate;
                    }
                    n += 1;
                }
            }
        };

        if state.configs.contains_key(&handle) {
            return Err(mlua::Error::RuntimeError(format!(
                "config '{}' already exists",
                handle
            )));
        }

        state.configs.insert(handle.clone(), cfg);

        Ok(handle)
    })?;

    dstest.set("config", config_fn)?;
    Ok(())
}
