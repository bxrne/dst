use crate::bindings::LuaModule;
use crate::bindings::context::BindingContext;
use mlua::{Lua, Table};

mod clear;
mod oracle;
mod run_steps;
mod step;

pub struct Dst;

impl LuaModule for Dst {
    fn namespace() -> &'static str {
        "dst"
    }

    fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> mlua::Result<()> {
        let dst_table = lua.create_table()?;
        clear::register(lua, &dst_table, ctx)?;
        oracle::register(lua, &dst_table, ctx)?;
        step::register(lua, &dst_table, ctx)?;
        run_steps::register(lua, &dst_table, ctx)?;
        dstest.set(Self::namespace(), dst_table)?;
        Ok(())
    }
}
