use std::sync::Arc;

use mlua::{Lua, Result, Table};

use crate::bindings::context::BindingContext;
use crate::config::AccumulationMode;
use crate::fault::FaultTree;
use crate::substrate::Substrate;

pub fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext) -> Result<()> {
    let state = Arc::clone(&ctx.state);
    let config = Arc::clone(&ctx.config);
    let fault_tree = Arc::clone(&ctx.fault_tree);
    let docker = Arc::clone(&ctx.docker);
    let oracle = Arc::clone(&ctx.oracle);

    let run_steps_fn = lua.create_function(move |lua, n: usize| {
        let (require_seed, accumulation_mode, step_delay) = {
            let cfg = config.lock().expect("poisoned config lock");
            (cfg.require_seed, cfg.accumulation_mode, cfg.step_delay_ms)
        };

        if require_seed {
            let s = state.lock().expect("poisoned engine state lock");
            if s.seed.is_none() {
                return Err(mlua::Error::RuntimeError(
                    "dstest.config({ seed = n }) must be called before dstest.run_steps()".into(),
                ));
            }
        }

        let mut tree_guard = fault_tree.lock().expect("poisoned fault tree lock");

        if tree_guard.is_none() {
            let s = state.lock().expect("poisoned engine state lock");
            if let Some(seed) = s.seed {
                let subject_ids: Vec<String> =
                    s.subjects.iter().map(|(id, _)| id.clone()).collect();
                if subject_ids.is_empty() {
                    return Err(mlua::Error::RuntimeError(
                        "no subjects available for fault injection".into(),
                    ));
                }
                let cfg = config.lock().expect("poisoned config lock");
                *tree_guard = Some(FaultTree::new(seed, subject_ids, &cfg));
            } else {
                return Err(mlua::Error::RuntimeError(
                    "dstest.config({ seed = n }) must be called before dstest.run_steps()".into(),
                ));
            }
        }

        let tree = tree_guard
            .as_mut()
            .ok_or_else(|| mlua::Error::RuntimeError("fault tree not initialized".into()))?;

        let mut results = Vec::new();
        for _ in 0..n {
            let Some(step_result) = tree.step() else {
                break;
            };

            let subject = crate::substrate::Subject::new(step_result.subject_id.clone());

            match accumulation_mode {
                AccumulationMode::Single => {
                    docker
                        .clear_faults(&subject)
                        .map_err(mlua::Error::RuntimeError)?;
                    std::thread::sleep(std::time::Duration::from_millis(step_delay));
                }
                AccumulationMode::Accumulate => {}
            }

            docker
                .affect(&subject, &step_result.fault)
                .map_err(mlua::Error::RuntimeError)?;

            let t = lua.create_table()?;
            t.set("fault", step_result.fault.to_string())?;
            t.set("subject", step_result.subject_id.clone())?;
            t.set("round", step_result.round)?;
            t.set("total_rounds", step_result.total_rounds)?;
            t.set("remaining", step_result.remaining)?;
            t.set("more", step_result.more)?;

            {
                let mut o = oracle.lock().expect("poisoned oracle lock");
                if o.enabled {
                    let report = o.check_all(
                        lua,
                        &step_result.subject_id,
                        &step_result.fault.to_string(),
                        step_result.round,
                    );
                    o.report.merge(&report);
                    t.set("oracle", {
                        let ot = lua.create_table()?;
                        ot.set("passed", report.passed)?;
                        ot.set("total_checks", report.total_checks)?;
                        ot.set("passed_checks", report.passed_checks)?;
                        ot.set("failed_checks", report.failed_checks)?;
                        ot
                    })?;
                }
            }

            results.push(t);
        }

        let result_table = lua.create_table()?;
        for (i, t) in results.into_iter().enumerate() {
            result_table.set(i + 1, t)?;
        }

        Ok(result_table)
    })?;

    dstest.set("run_steps", run_steps_fn)?;
    Ok(())
}
