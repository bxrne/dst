pub mod http;
pub mod pg;

use mlua::{Lua, Table};

use crate::bindings::LuaModule;
use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

pub struct Workload;

impl<S: Substrate> LuaModule<S> for Workload {
    fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> mlua::Result<()> {
        let workload_table = lua.create_table()?;
        http::register(lua, &workload_table, ctx)?;
        pg::register(lua, &workload_table, ctx)?;
        dstest.set("workload", workload_table)?;
        Ok(())
    }
}
