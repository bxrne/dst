use mlua::{Lua, Table};

use crate::bindings::LuaModule;
use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

mod http;
mod tcp;

pub struct Net;

impl<S: Substrate> LuaModule<S> for Net {
    fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> mlua::Result<()> {
        let net_table = lua.create_table()?;
        http::register(lua, &net_table, ctx)?;
        tcp::register(lua, &net_table, ctx)?;
        dstest.set("net", net_table)?;
        Ok(())
    }
}
