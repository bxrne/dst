use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::bindings::context::BindingContext;
use crate::substrate::Substrate;

pub fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> Result<()> {
    let docker = Arc::clone(&ctx.docker);

    let clear_fn = lua.create_function(move |_, subject_id: String| {
        let subject = crate::substrate::Subject::new(subject_id.clone());
        docker
            .clear_faults(&subject)
            .map_err(mlua::Error::RuntimeError)?;

        Ok(())
    })?;

    dstest.set("clear", clear_fn)?;
    Ok(())
}
