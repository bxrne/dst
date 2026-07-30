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
        clear::register(lua, dstest, ctx)?;
        oracle::register(lua, dstest, ctx)?;
        step::register(lua, dstest, ctx)?;
        run_steps::register(lua, dstest, ctx)?;
        Ok(())
    }
}
