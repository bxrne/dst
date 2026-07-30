use crate::bindings::LuaModule;

mod exec;
mod inspect;
mod logs;

pub struct Subs;

impl LuaModule for Subs {
    fn namespace() -> &'static str {
        "subs"
    }

    fn register(
        lua: &mlua::Lua,
        dstest: &mlua::Table,
        ctx: &crate::bindings::context::BindingContext,
    ) -> mlua::Result<()> {
        let subs_table = lua.create_table()?;
        exec::register(lua, &subs_table, ctx)?;
        inspect::register(lua, &subs_table, ctx)?;
        logs::register(lua, &subs_table, ctx)?;
        dstest.set(Self::namespace(), subs_table)?;
        Ok(())
    }
}
