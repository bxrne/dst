use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::engine::context::BindingContext;
use crate::substrate::{Subject, Substrate, ToLua};

pub fn register<S: Substrate>(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> Result<()> {
    let substrate = Arc::clone(&ctx.substrate);

    let inspect_fn = lua.create_function(move |lua, id: String| {
        let subject = Subject::new(id);

        let info = substrate
            .inspect(&subject)
            .map_err(mlua::Error::RuntimeError)?;

        info.to_lua(lua)
    })?;

    dstest.set("inspect", inspect_fn)?;
    Ok(())
}
