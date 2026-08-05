//! Shared fault-step execution for `dstest.dst.step` and
//! `dstest.dst.run_steps`.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use mlua::{Lua, Table};

use crate::config::AccumulationMode;
use crate::engine::state::EngineState;
use crate::fault::FaultTree;
use crate::oracle::OracleRef;
use crate::substrate::{Subject, Substrate};

/// Resolve which config a fault step applies to: an explicit handle, or the
/// only registered config when there is exactly one.
pub fn resolve_handle<S: Substrate>(
    state: &EngineState<S>,
    arg: Option<String>,
) -> Result<String, mlua::Error> {
    match arg {
        Some(h) => {
            if state.configs.contains_key(&h) {
                Ok(h)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "unknown config '{}' — pass the handle returned by dstest.config()",
                    h
                )))
            }
        }
        None => match state.configs.len() {
            0 => Err(mlua::Error::RuntimeError(
                "no configs registered: call dstest.config({...}) first".into(),
            )),
            1 => Ok(state.configs.keys().next().unwrap().clone()),
            _ => Err(mlua::Error::RuntimeError(
                "multiple configs registered; pass a config handle, e.g. dstest.dst.step(cfg)"
                    .into(),
            )),
        },
    }
}

/// Execute a single fault step against the given config's subjects: pick the
/// next fault from the config's fault tree, apply it, record it, and run the
/// oracle if enabled. Returns the Lua result table, or `None` when the
/// config's fault schedule is exhausted.
#[allow(clippy::await_holding_lock)]
pub async fn execute_step<S: Substrate>(
    lua: &Lua,
    state: &Arc<Mutex<EngineState<S>>>,
    fault_trees: &Arc<Mutex<BTreeMap<String, FaultTree>>>,
    substrate: &Arc<S>,
    oracle: &OracleRef,
    cfg_arg: Option<String>,
) -> mlua::Result<Option<Table>> {
    let (cfg, handle) = {
        let s = state.lock().expect("poisoned engine state lock");
        let h = resolve_handle(&s, cfg_arg)?;
        let cfg = s
            .configs
            .get(&h)
            .expect("resolved config must exist")
            .clone();
        (cfg, h)
    };

    if cfg.require_seed && cfg.seed.is_none() {
        return Err(mlua::Error::RuntimeError(format!(
            "config '{}' has no seed: set seed = n in dstest.config()",
            handle
        )));
    }

    // Lazily create this config's fault tree from its current subjects.
    // Subjects set up after the first step for a config are not faulted.
    {
        let mut trees = fault_trees.lock().expect("poisoned fault tree lock");
        if !trees.contains_key(&handle) {
            let subject_ids: Vec<String> = {
                let s = state.lock().expect("poisoned engine state lock");
                s.subjects
                    .iter()
                    .filter(|r| r.config == handle)
                    .map(|r| r.id.clone())
                    .collect()
            };
            if subject_ids.is_empty() {
                return Err(mlua::Error::RuntimeError(format!(
                    "no subjects for config '{}' — call dstest.setup({}, {{...}}) first",
                    handle, handle
                )));
            }
            let seed = cfg.seed.ok_or_else(|| {
                mlua::Error::RuntimeError(format!("config '{}' has no seed", handle))
            })?;
            trees.insert(handle.clone(), FaultTree::new(seed, subject_ids, &cfg));
        }
    }

    let step_result = {
        let mut trees = fault_trees.lock().expect("poisoned fault tree lock");
        trees.get_mut(&handle).and_then(|t| t.step())
    };

    let Some(step_result) = step_result else {
        return Ok(None);
    };

    let subject = Subject::new(step_result.subject_id.clone());

    match cfg.accumulation_mode {
        AccumulationMode::Single => {
            substrate
                .clear_faults(&subject)
                .await
                .map_err(mlua::Error::RuntimeError)?;
            {
                let mut s = state.lock().expect("poisoned engine state lock");
                if let Some(rec) = s
                    .subjects
                    .iter_mut()
                    .find(|r| r.id == step_result.subject_id)
                {
                    rec.active_faults.clear();
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(cfg.step_delay_ms)).await;
        }
        AccumulationMode::Accumulate => {}
    }

    substrate
        .affect(&subject, &step_result.fault)
        .await
        .map_err(mlua::Error::RuntimeError)?;

    {
        let mut s = state.lock().expect("poisoned engine state lock");
        if let Some(rec) = s
            .subjects
            .iter_mut()
            .find(|r| r.id == step_result.subject_id)
        {
            rec.active_faults.push(step_result.fault);
        }
    }

    let t = lua.create_table()?;
    t.set("fault", step_result.fault.to_string())?;
    t.set("subject", step_result.subject_id.clone())?;
    t.set("config", handle.clone())?;
    t.set("round", step_result.round)?;
    t.set("total_rounds", step_result.total_rounds)?;
    t.set("remaining", step_result.remaining)?;
    t.set("more", step_result.more)?;

    {
        let mut o = oracle.lock().expect("poisoned oracle lock");
        if o.enabled {
            let report = o
                .check_all(
                    lua,
                    &step_result.subject_id,
                    &step_result.fault.to_string(),
                    step_result.round,
                )
                .await;
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

    Ok(Some(t))
}
