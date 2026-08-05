use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

use super::common::execute_step;

pub fn register<S: Substrate>(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> Result<()> {
    let state = Arc::clone(&ctx.state);
    let fault_trees = Arc::clone(&ctx.fault_trees);
    let substrate = Arc::clone(&ctx.substrate);
    let oracle = Arc::clone(&ctx.oracle);

    let step_fn = lua.create_async_function(move |lua, cfg: Option<String>| {
        let state = Arc::clone(&state);
        let fault_trees = Arc::clone(&fault_trees);
        let substrate = Arc::clone(&substrate);
        let oracle = Arc::clone(&oracle);

        async move {
            match execute_step(&lua, &state, &fault_trees, &substrate, &oracle, cfg).await? {
                Some(t) => Ok(t),
                None => {
                    let t = lua.create_table()?;
                    t.set("more", false)?;
                    Ok(t)
                }
            }
        }
    })?;

    dstest.set("step", step_fn)?;
    Ok(())
}
