use std::sync::Arc;

use mlua::{Lua, MultiValue, Result, Table, Value};

use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

use super::common::execute_step;

pub fn register<S: Substrate>(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> Result<()> {
    let state = Arc::clone(&ctx.state);
    let fault_trees = Arc::clone(&ctx.fault_trees);
    let substrate = Arc::clone(&ctx.substrate);
    let oracle = Arc::clone(&ctx.oracle);

    let run_steps_fn = lua.create_async_function(move |lua, args: MultiValue| {
        let state = Arc::clone(&state);
        let fault_trees = Arc::clone(&fault_trees);
        let substrate = Arc::clone(&substrate);
        let oracle = Arc::clone(&oracle);

        async move {
            // run_steps(n) or run_steps(cfg_handle, n)
            let mut args = args.into_iter();
            let (cfg, n) = match (args.next(), args.next()) {
                (Some(Value::Integer(n)), None) => (None, n as usize),
                (Some(Value::String(s)), Some(Value::Integer(n))) => {
                    (Some(s.to_str()?.to_owned()), n as usize)
                }
                _ => {
                    return Err(mlua::Error::RuntimeError(
                        "dstest.dst.run_steps expects (n) or (config_handle, n)".to_string(),
                    ));
                }
            };

            let mut results = Vec::new();
            for _ in 0..n {
                let Some(t) =
                    execute_step(&lua, &state, &fault_trees, &substrate, &oracle, cfg.clone())
                        .await?
                else {
                    break; // fault schedule exhausted
                };

                results.push(t);
            }

            let result_table = lua.create_table()?;
            for (i, t) in results.into_iter().enumerate() {
                result_table.set(i + 1, t)?;
            }

            Ok(result_table)
        }
    })?;

    dstest.set("run_steps", run_steps_fn)?;
    Ok(())
}
