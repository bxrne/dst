use tracing::{debug, warn};

use crate::bindings::{BindingContext, register_all};

pub struct Engine {
    ctx: BindingContext,
}

impl Engine {
    pub fn new() -> Self {
        let ctx = BindingContext::new();
        let globals = ctx.lua.globals();
        let dstest = ctx
            .lua
            .create_table()
            .expect("failed to create dstest table");

        let _ = globals.set(
            "print",
            ctx.lua
                .create_function(|_, msg: String| {
                    tracing::info!("lua: {}", msg);
                    Ok(())
                })
                .expect("failed to create print function"),
        );

        register_all(&ctx.lua, &dstest, &ctx).expect("failed to register bindings");
        globals
            .set("dstest", dstest)
            .expect("failed to set global dstest");

        Engine { ctx }
    }

    pub fn execute(&self, script: &str) -> mlua::Result<()> {
        self.ctx.lua.load(script).exec()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let subjects = {
            let mut state = self.ctx.state.lock().expect("poisoned engine state lock");
            std::mem::take(&mut state.subjects)
        };

        for (id, _) in subjects {
            let subject = crate::substrate::Subject::new(id.clone());
            if let Err(e) = self.ctx.substrate.teardown(subject) {
                warn!("RAII teardown failed for subject {}: {}", id, e);
            } else {
                debug!("teardown complete for subject {}", id);
            }
        }
    }
}
