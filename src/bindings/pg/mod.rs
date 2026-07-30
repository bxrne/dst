use crate::bindings::LuaModule;

mod command;
mod pool;

pub struct Pg;

impl LuaModule for Pg {
    fn namespace() -> &'static str {
        "pg"
    }

    fn register(
        lua: &mlua::Lua,
        dstest: &mlua::Table,
        ctx: &crate::bindings::context::BindingContext,
    ) -> mlua::Result<()> {
        let pg_table = lua.create_table()?;
        command::register(lua, &pg_table, ctx)?;
        dstest.set(Self::namespace(), pg_table)?;
        Ok(())
    }
}
