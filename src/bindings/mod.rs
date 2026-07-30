mod clear;
mod clock;
mod context;
mod exec;
mod http;
mod inspect;
mod log;
mod logs;
mod opts;
mod oracle;
mod pg;
mod run_steps;
mod setup;
mod step;
mod tcp;

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
    logs::register(lua, dstest, ctx)?;
    inspect::register(lua, dstest, ctx)?;
    exec::register(lua, dstest, ctx)?;
    pg::register(lua, dstest, ctx)?;
    clock::register(lua, dstest, ctx)?;
    tcp::register(lua, dstest, ctx)?;
    Ok(())
}
