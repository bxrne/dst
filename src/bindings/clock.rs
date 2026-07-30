use mlua::{Lua, Result, Table};

use crate::bindings::LuaModule;
use crate::engine::context::BindingContext;
use crate::substrate::Substrate;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Clock;

impl<S: Substrate> LuaModule<S> for Clock {
    fn register(lua: &Lua, dstest: &Table, _ctx: &BindingContext<S>) -> Result<()> {
        let clock_fn = lua.create_function(move |lua, _: ()| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards");

            let t = lua.create_table()?;
            t.set("nanos", now.as_nanos() as u64)?;
            t.set("micros", now.as_micros() as u64)?;
            t.set("millis", now.as_millis() as u64)?;
            t.set("secs", now.as_secs())?;

            Ok(t)
        })?;

        dstest.set("clock", clock_fn)?;
        Ok(())
    }
}
