use mlua::Lua;

use crate::bindings::LuaModule;
use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

mod exec;
mod inspect;
mod logs;

pub struct Subs;

impl<S: Substrate> LuaModule<S> for Subs {
    fn register(lua: &Lua, dstest: &mlua::Table, ctx: &BindingContext<S>) -> mlua::Result<()> {
        exec::register(lua, dstest, ctx)?;
        inspect::register(lua, dstest, ctx)?;
        logs::register(lua, dstest, ctx)?;
        Ok(())
    }
}
