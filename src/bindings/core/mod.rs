mod opts;
mod setup;

use crate::bindings::LuaModule;

pub struct Core;

impl LuaModule for Core {
    fn namespace() -> &'static str {
        "core"
    }

    fn register(
        lua: &mlua::Lua,
        dstest: &mlua::Table,
        ctx: &crate::bindings::context::BindingContext,
    ) -> mlua::Result<()> {
        let core_table = lua.create_table()?;
        opts::register(lua, &core_table, ctx)?;
        setup::register(lua, &core_table, ctx)?;
        dstest.set(Self::namespace(), core_table)?;
        Ok(())
    }
}
