use crate::bindings::register_all;
use crate::substrate::{Subject, Substrate};
use tracing::{debug, warn};

pub mod context;
pub mod state;

pub use context::BindingContext;

pub struct Engine<S: Substrate> {
    ctx: BindingContext<S>,
}

impl<S: Substrate> Engine<S> {
    pub fn new(substrate: S) -> Self {
        let ctx = BindingContext::new(substrate);
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

    pub async fn execute(&self, script: &str) -> mlua::Result<()> {
        self.ctx.lua.load(script).call_async::<()>(()).await
    }

    /// Tear down every live subject, awaiting each. Call this while a tokio
    /// runtime is still alive; `Drop` remains as a last-resort fallback.
    pub async fn shutdown(&self) {
        let records: Vec<(String, String)> = {
            let mut state = self.ctx.state.lock().expect("poisoned engine state lock");
            state.subjects.drain(..).map(|r| (r.id, r.name)).collect()
        };

        for (id, name) in records {
            if let Err(e) = self.ctx.substrate.teardown(Subject::new(id.clone())).await {
                warn!("teardown failed for subject {} ({}): {}", name, id, e);
            } else {
                debug!("teardown complete for subject {} ({})", name, id);
            }
        }
    }

    /// Final oracle report for the run (used for the process exit code).
    pub fn oracle_report(&self) -> crate::oracle::OracleReport {
        self.ctx
            .oracle
            .lock()
            .expect("poisoned oracle lock")
            .report
            .clone()
    }
}

impl<S: Substrate> Drop for Engine<S> {
    fn drop(&mut self) {
        let records: Vec<(String, String)> = {
            let mut state = self.ctx.state.lock().expect("poisoned engine state lock");
            state.subjects.drain(..).map(|r| (r.id, r.name)).collect()
        };

        if records.is_empty() {
            return;
        }

        // Last-resort teardown (engine dropped without `shutdown`). We may be
        // on a thread without a tokio runtime, so run the async teardown on a
        // dedicated thread with its own current-thread runtime.
        let substrate = std::sync::Arc::clone(&self.ctx.substrate);
        let handle = std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    warn!("could not build teardown runtime: {}", e);
                    return;
                }
            };
            rt.block_on(async move {
                for (id, name) in records {
                    if let Err(e) = substrate.teardown(Subject::new(id.clone())).await {
                        warn!("RAII teardown failed for subject {} ({}): {}", name, id, e);
                    } else {
                        debug!("teardown complete for subject {} ({})", name, id);
                    }
                }
            });
        });
        let _ = handle.join();
    }
}
