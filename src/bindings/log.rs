use mlua::{Lua, Result, Table};

use crate::bindings::context::BindingContext;

pub fn register(lua: &Lua, dstest: &Table, _ctx: &BindingContext) -> Result<()> {
    let debug_fn = lua.create_function(|_, msg: String| {
        tracing::debug!("{}", msg);
        Ok(())
    })?;

    let info_fn = lua.create_function(|_, msg: String| {
        tracing::info!("{}", msg);
        Ok(())
    })?;

    let warn_fn = lua.create_function(|_, msg: String| {
        tracing::warn!("{}", msg);
        Ok(())
    })?;

    let error_fn = lua.create_function(|_, msg: String| {
        tracing::error!("{}", msg);
        Ok(())
    })?;

    dstest.set("debug", debug_fn)?;
    dstest.set("info", info_fn)?;
    dstest.set("warn", warn_fn)?;
    dstest.set("error", error_fn)?;
    Ok(())
}
