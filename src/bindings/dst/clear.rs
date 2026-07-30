use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::bindings::context::BindingContext;

pub fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> Result<()> {
    let substrate = Arc::clone(&ctx.substrate);

    let clear_fn = lua.create_function(move |_, subject_id: String| {
        let subject = crate::substrate::Subject::new(subject_id.clone());
        substrate
            .clear_faults(&subject)
            .map_err(mlua::Error::RuntimeError)?;

        Ok(())
    })?;

    dstest.set("clear", clear_fn)?;
    Ok(())
}
