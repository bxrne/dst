use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

pub fn register<S: Substrate>(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> Result<()> {
    let state = Arc::clone(&ctx.state);
    let substrate = Arc::clone(&ctx.substrate);
    let config = Arc::clone(&ctx.config);

    let setup = lua.create_function(move |lua, args: mlua::MultiValue| {
        let mut args_iter = args.into_iter();
        let first = args_iter.next();
        let second = args_iter.next();

        let (substrate_type, config_tbl): (Option<String>, Table) = match (first, second) {
            (Some(mlua::Value::String(s)), Some(mlua::Value::Table(t))) => {
                (Some(s.to_str()?.to_owned()), t)
            }
            (Some(mlua::Value::Table(t)), None) => (None, t),
            (Some(mlua::Value::String(s)), None) => {
                let s = s.to_str()?.to_owned();
                let t = lua.create_table()?;
                (Some(s), t)
            }
            _ => {
                return Err(mlua::Error::RuntimeError(
                    "dstest.setup expects (table) or (string, table)".to_string(),
                ));
            }
        };

        let cfg = config.lock().expect("poisoned config lock");
        let substrate_name = substrate_type
            .or_else(|| cfg.substrate.clone())
            .ok_or_else(|| {
                mlua::Error::RuntimeError(
                    "no substrate configured: call dstest.config({ substrate = \"docker\" }) first"
                        .to_string(),
                )
            })?;
        drop(cfg);

        if substrate_name != S::NAME {
            return Err(mlua::Error::RuntimeError(format!(
                "substrate mismatch: config wants \"{}\" but engine was built for \"{}\"",
                substrate_name,
                S::NAME
            )));
        }

        let data = substrate
            .parse_subject(&config_tbl)
            .map_err(mlua::Error::RuntimeError)?;

        let hosted = substrate.host(&data).map_err(mlua::Error::RuntimeError)?;

        let subject_id = format!("{}/{}", S::NAME, hosted.id);

        let mut state = state.lock().expect("poisoned engine state lock");

        if let Some(addr) = hosted.addr {
            state.subject_hosts.insert(subject_id.clone(), addr);
        }

        state.subjects.push((subject_id.clone(), data));

        Ok(subject_id)
    })?;

    dstest.set("setup", setup)?;
    Ok(())
}
