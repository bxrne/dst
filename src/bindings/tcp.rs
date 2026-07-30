use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use mlua::{Lua, Result, Table};

use crate::bindings::context::BindingContext;

pub fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> Result<()> {
    let state = Arc::clone(&ctx.state);
    let config = Arc::clone(&ctx.config);

    let tcp_fn = lua.create_function(move |lua, (id, port): (String, u16)| {
        let host = {
            let state = state.lock().expect("poisoned lock");
            state
                .subject_hosts
                .get(&id)
                .cloned()
                .ok_or_else(|| mlua::Error::RuntimeError(format!("unknown subject {}", id)))?
        };

        let host_ip = host.split(':').next().unwrap_or(&host);
        let addr = format!("{}:{}", host_ip, port);

        let timeout_secs = {
            let cfg = config.lock().expect("poisoned lock");
            cfg.http_timeout_secs
        };

        match TcpStream::connect_timeout(
            &addr
                .parse()
                .map_err(|e| mlua::Error::RuntimeError(format!("invalid address: {}", e)))?,
            Duration::from_secs(timeout_secs),
        ) {
            Ok(_stream) => {
                let t = lua.create_table()?;
                t.set("connected", true)?;
                t.set("addr", addr)?;
                Ok(t)
            }
            Err(e) => {
                let t = lua.create_table()?;
                t.set("connected", false)?;
                t.set("error", e.to_string())?;
                t.set("addr", addr)?;
                Ok(t)
            }
        }
    })?;

    dstest.set("tcp", tcp_fn)?;
    Ok(())
}
