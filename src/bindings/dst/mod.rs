use mlua::{Lua, Table};

use crate::bindings::LuaModule;
use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

mod clear;
mod oracle;
mod run_steps;
mod step;

pub struct Dst;

impl<S: Substrate> LuaModule<S> for Dst {
    fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> mlua::Result<()> {
        let dst_table = lua.create_table()?;
        clear::register(lua, &dst_table, ctx)?;
        oracle::register(lua, &dst_table, ctx)?;
        step::register(lua, &dst_table, ctx)?;
        run_steps::register(lua, &dst_table, ctx)?;
        dstest.set("dst", dst_table)?;
        Ok(())
    }
}
