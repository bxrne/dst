mod clock;
mod context;
mod core;
mod dst;
mod log;
mod net;
mod pg;
mod subs;

pub use context::BindingContext;

use mlua::{Lua, Table};

pub trait LuaModule {
    /// The namespace key used in Lua (e.g., "http", "pg", "clock")
    fn namespace() -> &'static str;

    /// Register functions, types, or state into the Lua environment
    fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> mlua::Result<()>;
}

pub fn register_all(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> mlua::Result<()> {
    // Modules
    net::Net::register(lua, dstest, ctx)?;
    dst::Dst::register(lua, dstest, ctx)?;
    subs::Subs::register(lua, dstest, ctx)?;
    log::Log::register(lua, dstest, ctx)?;
    clock::Clock::register(lua, dstest, ctx)?;
    core::Core::register(lua, dstest, ctx)?;
    pg::Pg::register(lua, dstest, ctx)?;

    Ok(())
}
