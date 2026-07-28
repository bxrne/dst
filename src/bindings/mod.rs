mod clear;
mod context;
mod http;
mod log;
mod opts;
mod oracle;
mod run_steps;
mod setup;
mod step;

pub use context::BindingContext;

use mlua::{Lua, Table};

pub fn register_all(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> mlua::Result<()> {
    log::register(lua, dstest, ctx)?;
    opts::register(lua, dstest, ctx)?;
    setup::register(lua, dstest, ctx)?;
    http::register(lua, dstest, ctx)?;
    oracle::register(lua, dstest, ctx)?;
    step::register(lua, dstest, ctx)?;
    run_steps::register(lua, dstest, ctx)?;
    clear::register(lua, dstest, ctx)?;
    Ok(())
}
