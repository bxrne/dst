use crate::bindings::LuaModule;
use crate::bindings::context::BindingContext;
use mlua::{Lua, Table};

mod http;
mod tcp;

pub struct Net;

impl LuaModule for Net {
    fn namespace() -> &'static str {
        "net"
    }

    fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> mlua::Result<()> {
        let net_table = lua.create_table()?;
        http::register(lua, &net_table, ctx)?;
        tcp::register(lua, &net_table, ctx)?;
        dstest.set(Self::namespace(), net_table)?;
        Ok(())
    }
}
