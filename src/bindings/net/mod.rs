use mlua::{Lua, Table};

use crate::bindings::LuaModule;
use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

mod http;
mod tcp;

pub struct Net;

impl<S: Substrate> LuaModule<S> for Net {
    fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> mlua::Result<()> {
        http::register(lua, dstest, ctx)?;
        tcp::register(lua, dstest, ctx)?;
        Ok(())
    }
}
