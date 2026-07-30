mod clock;
mod core;
mod dst;
mod log;
mod net;
mod pg;
mod subs;

use mlua::{Lua, Table};

use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

pub trait LuaModule<S: Substrate> {
    fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> mlua::Result<()>;
}

pub fn register_all<S: Substrate>(
    lua: &Lua,
    dstest: &Table,
    ctx: &BindingContext<S>,
) -> mlua::Result<()> {
    net::Net::register(lua, dstest, ctx)?;
    dst::Dst::register(lua, dstest, ctx)?;
    subs::Subs::register(lua, dstest, ctx)?;
    log::Log::register(lua, dstest, ctx)?;
    clock::Clock::register(lua, dstest, ctx)?;
    core::Core::register(lua, dstest, ctx)?;
    pg::Pg::register(lua, dstest, ctx)?;

    Ok(())
}
