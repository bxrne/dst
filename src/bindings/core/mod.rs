mod opts;
mod setup;

use mlua::Lua;

use crate::bindings::LuaModule;
use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

pub struct Core;

impl<S: Substrate> LuaModule<S> for Core {
    fn register(lua: &Lua, dstest: &mlua::Table, ctx: &BindingContext<S>) -> mlua::Result<()> {
        opts::register(lua, dstest, ctx)?;
        setup::register(lua, dstest, ctx)?;
        Ok(())
    }
}
