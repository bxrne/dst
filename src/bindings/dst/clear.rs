use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

pub fn register<S: Substrate>(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> Result<()> {
    let state = Arc::clone(&ctx.state);
    let substrate = Arc::clone(&ctx.substrate);

    let clear_fn = lua.create_async_function(move |_, subject_id: String| {
        let state = Arc::clone(&state);
        let substrate = Arc::clone(&substrate);

        async move {
            let subject = crate::substrate::Subject::new(subject_id.clone());
            substrate
                .clear_faults(&subject)
                .await
                .map_err(mlua::Error::RuntimeError)?;

            let mut s = state.lock().expect("poisoned engine state lock");
            if let Some(rec) = s.subjects.iter_mut().find(|r| r.id == subject_id) {
                rec.active_faults.clear();
            }

            Ok(())
        }
    })?;

    dstest.set("clear", clear_fn)?;
    Ok(())
}
