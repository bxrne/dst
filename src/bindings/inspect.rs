use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::bindings::context::BindingContext;
use crate::substrate::{ContainerState, Subject};

pub fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> Result<()> {
    let substrate = Arc::clone(&ctx.substrate);

    let inspect_fn = lua.create_function(move |lua, id: String| {
        let subject = Subject::new(id);

        let info = substrate
            .inspect(&subject)
            .map_err(mlua::Error::RuntimeError)?;

        let t = lua.create_table()?;
        t.set(
            "state",
            match info.state {
                ContainerState::Running => "running",
                ContainerState::Paused => "paused",
                ContainerState::Exited => "exited",
                ContainerState::Dead => "dead",
            },
        )?;
        t.set("pid", info.pid)?;
        t.set("ip", info.ip)?;
        t.set("memory_limit", info.memory_limit)?;
        t.set("cpu_quota", info.cpu_quota)?;

        Ok(t)
    })?;

    dstest.set("inspect", inspect_fn)?;
    Ok(())
}
