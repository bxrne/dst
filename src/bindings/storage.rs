//! `dstest.storage` — fault-injectable virtual disk control.
//!
//! Dispatches through the substrate's `StorageControl` implementation.
//! Substrates without virtual storage return "not supported" errors.

use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::bindings::LuaModule;
use crate::components::{StorageControl, StorageOpts};
use crate::engine::context::BindingContext;
use crate::substrate::{Subject, Substrate};

pub struct Storage;

impl<S: Substrate> LuaModule<S> for Storage {
    fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> Result<()> {
        let storage_table = lua.create_table()?;

        let substrate = Arc::clone(&ctx.substrate);
        let attach_fn = lua.create_async_function(move |_, (id, opts): (String, Table)| {
            let substrate = Arc::clone(&substrate);
            async move {
                let size_mb: u64 = opts.get("size_mb").unwrap_or(512);
                let mount: String = opts.get("mount").map_err(|_| {
                    mlua::Error::RuntimeError("storage.attach requires a `mount` field".to_string())
                })?;
                let opts = StorageOpts { size_mb, mount };
                opts.validate().map_err(mlua::Error::RuntimeError)?;
                substrate
                    .storage()
                    .attach(&Subject::new(id), opts)
                    .await
                    .map_err(mlua::Error::RuntimeError)
            }
        })?;
        storage_table.set("attach", attach_fn)?;

        let substrate = Arc::clone(&ctx.substrate);
        let error_fn = lua.create_async_function(move |_, (id, on): (String, bool)| {
            let substrate = Arc::clone(&substrate);
            async move {
                substrate
                    .storage()
                    .error(&Subject::new(id), on)
                    .await
                    .map_err(mlua::Error::RuntimeError)
            }
        })?;
        storage_table.set("error", error_fn)?;

        let substrate = Arc::clone(&ctx.substrate);
        let drop_fn = lua.create_async_function(move |_, (id, on): (String, bool)| {
            let substrate = Arc::clone(&substrate);
            async move {
                substrate
                    .storage()
                    .drop_writes(&Subject::new(id), on)
                    .await
                    .map_err(mlua::Error::RuntimeError)
            }
        })?;
        storage_table.set("drop_writes", drop_fn)?;

        let substrate = Arc::clone(&ctx.substrate);
        let slow_fn = lua.create_async_function(move |_, (id, delay_ms): (String, u64)| {
            let substrate = Arc::clone(&substrate);
            async move {
                substrate
                    .storage()
                    .slow(&Subject::new(id), delay_ms)
                    .await
                    .map_err(mlua::Error::RuntimeError)
            }
        })?;
        storage_table.set("slow", slow_fn)?;

        let substrate = Arc::clone(&ctx.substrate);
        let corrupt_fn = lua.create_async_function(move |_, (id, n): (String, u64)| {
            let substrate = Arc::clone(&substrate);
            async move {
                substrate
                    .storage()
                    .corrupt(&Subject::new(id), n)
                    .await
                    .map_err(mlua::Error::RuntimeError)
            }
        })?;
        storage_table.set("corrupt", corrupt_fn)?;

        let substrate = Arc::clone(&ctx.substrate);
        let snapshot_fn = lua.create_async_function(move |_, id: String| {
            let substrate = Arc::clone(&substrate);
            async move {
                substrate
                    .storage()
                    .snapshot(&Subject::new(id))
                    .await
                    .map_err(mlua::Error::RuntimeError)
            }
        })?;
        storage_table.set("snapshot", snapshot_fn)?;

        let substrate = Arc::clone(&ctx.substrate);
        let restore_fn = lua.create_async_function(move |_, (id, snap): (String, String)| {
            let substrate = Arc::clone(&substrate);
            async move {
                substrate
                    .storage()
                    .restore(&Subject::new(id), &snap)
                    .await
                    .map_err(mlua::Error::RuntimeError)
            }
        })?;
        storage_table.set("restore", restore_fn)?;

        dstest.set("storage", storage_table)?;
        Ok(())
    }
}
