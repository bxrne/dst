use std::sync::Arc;

use mlua::{Function, Lua, Result, Table, Value};

use crate::engine::context::BindingContext;
use crate::substrate::Substrate;

pub fn register<S: Substrate>(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> Result<()> {
    let oracle = ctx.oracle.clone();
    let oracle_table = lua.create_table()?;

    let oracle_clone = Arc::clone(&oracle);
    let predicate_fn =
        lua.create_async_function(move |lua, (name, func): (String, Function)| {
            let oracle = Arc::clone(&oracle_clone);
            async move {
                let func_ref = lua.create_registry_value(func)?;
                let func_ref = Arc::new(func_ref);

                let predicate: crate::oracle::PredicateFn = Box::new(
                    move |lua: &Lua, subject: String, fault: String, round: usize| {
                        let func_ref = Arc::clone(&func_ref);
                        Box::pin(async move {
                            let func: Function = lua
                                .registry_value(&func_ref)
                                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                            let result: Value = func.call_async((subject, fault, round)).await?;

                            match result {
                                Value::Boolean(b) => Ok((b, None)),
                                Value::Table(t) => {
                                    let passed: bool = t.get(1)?;
                                    let msg: Option<String> = t.get(2).ok();
                                    Ok((passed, msg))
                                }
                                other => Err(mlua::Error::RuntimeError(format!(
                                    "predicate must return boolean or {{passed, message?}}, got {:?}",
                                    other.type_name()
                                ))),
                            }
                        })
                    },
                );

                oracle.lock().unwrap().add_predicate(name, predicate);
                Ok(())
            }
        })?;
    oracle_table.set("predicate", predicate_fn)?;

    let oracle_clone = Arc::clone(&oracle);
    let invariant_fn =
        lua.create_async_function(move |lua, (name, func): (String, Function)| {
            let oracle = Arc::clone(&oracle_clone);
            async move {
                let func_ref = lua.create_registry_value(func)?;
                let func_ref = Arc::new(func_ref);

                let invariant: crate::oracle::InvariantFn = Box::new(move |lua: &Lua| {
                    let func_ref = Arc::clone(&func_ref);
                    Box::pin(async move {
                        let func: Function = lua
                            .registry_value(&func_ref)
                            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                        let result: Value = func.call_async(()).await?;

                        match result {
                            Value::Boolean(b) => Ok((b, None)),
                            Value::Table(t) => {
                                let passed: bool = t.get(1)?;
                                let msg: Option<String> = t.get(2).ok();
                                Ok((passed, msg))
                            }
                            other => Err(mlua::Error::RuntimeError(format!(
                                "invariant must return boolean or {{passed, message?}}, got {:?}",
                                other.type_name()
                            ))),
                        }
                    })
                });

                oracle.lock().unwrap().add_invariant(name, invariant);
                Ok(())
            }
        })?;
    oracle_table.set("invariant", invariant_fn)?;

    let oracle_clone = Arc::clone(&oracle);
    let run_fn = lua.create_async_function(move |lua, func: Function| {
        let oracle = Arc::clone(&oracle_clone);
        async move {
            {
                let mut o = oracle.lock().unwrap();
                o.enabled = true;
                o.reset();
            }

            let _: Value = func.call_async(()).await?;

            let report = {
                let mut o = oracle.lock().unwrap();
                o.enabled = false;
                o.report.clone()
            };

            let t = lua.create_table()?;
            t.set("passed", report.passed)?;
            t.set("total_checks", report.total_checks)?;
            t.set("passed_checks", report.passed_checks)?;
            t.set("failed_checks", report.failed_checks)?;

            let failures = lua.create_table()?;
            for (i, f) in report.failures.into_iter().enumerate() {
                let ft = lua.create_table()?;
                ft.set("type", f.check_type)?;
                ft.set("name", f.name)?;
                if let Some(r) = f.round {
                    ft.set("round", r)?;
                }
                if let Some(fault) = f.fault {
                    ft.set("fault", fault)?;
                }
                if let Some(s) = f.subject {
                    ft.set("subject", s)?;
                }
                ft.set("error", f.error)?;
                failures.set(i + 1, ft)?;
            }
            t.set("failures", failures)?;

            Ok(t)
        }
    })?;
    oracle_table.set("run", run_fn)?;

    let oracle_clone = Arc::clone(&oracle);
    let enable_fn = lua.create_function(move |_lua, _: ()| {
        let mut o = oracle_clone.lock().unwrap();
        o.enabled = true;
        o.reset();
        Ok(())
    })?;
    oracle_table.set("enable", enable_fn)?;

    let oracle_clone = Arc::clone(&oracle);
    let disable_fn = lua.create_function(move |_lua, _: ()| {
        let mut o = oracle_clone.lock().unwrap();
        o.enabled = false;
        Ok(())
    })?;
    oracle_table.set("disable", disable_fn)?;

    let oracle_clone = Arc::clone(&oracle);
    let report_fn = lua.create_function(move |lua, ()| {
        let o = oracle_clone.lock().unwrap();
        let report = &o.report;

        let t = lua.create_table()?;
        t.set("passed", report.passed)?;
        t.set("total_checks", report.total_checks)?;
        t.set("passed_checks", report.passed_checks)?;
        t.set("failed_checks", report.failed_checks)?;

        let failures = lua.create_table()?;
        for (i, f) in report.failures.iter().enumerate() {
            let ft = lua.create_table()?;
            ft.set("type", f.check_type.clone())?;
            ft.set("name", f.name.clone())?;
            if let Some(r) = f.round {
                ft.set("round", r)?;
            }
            if let Some(ref fault) = f.fault {
                ft.set("fault", fault.clone())?;
            }
            if let Some(ref s) = f.subject {
                ft.set("subject", s.clone())?;
            }
            ft.set("error", f.error.clone())?;
            failures.set(i + 1, ft)?;
        }
        t.set("failures", failures)?;

        Ok(t)
    })?;
    oracle_table.set("report", report_fn)?;

    let oracle_clone = Arc::clone(&oracle);
    let reset_fn = lua.create_function(move |_lua, ()| {
        oracle_clone.lock().unwrap().reset();
        Ok(())
    })?;
    oracle_table.set("reset", reset_fn)?;

    dstest.set("oracle", oracle_table)?;
    Ok(())
}
