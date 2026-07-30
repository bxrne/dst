use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::engine::context::BindingContext;
use crate::substrate::{Subject, Substrate};

pub fn register<S: Substrate>(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> Result<()> {
    let substrate = Arc::clone(&ctx.substrate);

    let exec_fn = lua.create_function(move |lua, (id, cmd): (String, Vec<String>)| {
        let subject = Subject::new(id);

        let result = substrate
            .exec(&subject, &cmd)
            .map_err(mlua::Error::RuntimeError)?;

        let t = lua.create_table()?;
        t.set("exit_code", result.exit_code)?;
        t.set("stdout", result.stdout)?;
        t.set("stderr", result.stderr)?;

        Ok(t)
    })?;

    dstest.set("exec", exec_fn)?;
    Ok(())
}
