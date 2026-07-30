use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::bindings::context::BindingContext;
use crate::substrate::{LogOptions, Stream, Subject};

pub fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> Result<()> {
    let substrate = Arc::clone(&ctx.substrate);

    let logs_fn = lua.create_function(move |lua, (id, opts): (String, Option<Table>)| {
        let subject = Subject::new(id);

        let log_opts = if let Some(opts) = opts {
            LogOptions {
                stdout: opts.get("stdout").unwrap_or(true),
                stderr: opts.get("stderr").unwrap_or(true),
                tail: opts.get("tail").ok(),
                since: opts.get("since").ok(),
                timestamps: opts.get("timestamps").unwrap_or(false),
            }
        } else {
            LogOptions::default()
        };

        let entries = substrate
            .logs(&subject, log_opts)
            .map_err(mlua::Error::RuntimeError)?;

        let result = lua.create_table()?;
        for (i, entry) in entries.into_iter().enumerate() {
            let t = lua.create_table()?;
            t.set(
                "stream",
                match entry.stream {
                    Stream::StdOut => "stdout",
                    Stream::StdErr => "stderr",
                },
            )?;
            t.set("message", entry.message)?;
            result.set(i + 1, t)?;
        }

        Ok(result)
    })?;

    dstest.set("logs", logs_fn)?;
    Ok(())
}
