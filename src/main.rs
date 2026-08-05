use std::io::{Read, stdin};
use tracing::{debug, error};
mod bindings;
mod components;
mod config;
mod engine;
mod fault;
mod oracle;
mod substrate;

fn main() {
    tracing_subscriber::fmt::init();

    debug!("Starting dstest");

    let docker = substrate::docker::Docker::new().unwrap_or_else(|e| {
        error!("Failed to connect to Docker daemon: {e}");
        std::process::exit(3);
    });

    let engine = engine::Engine::new(docker);

    debug!("Reading scripts from stdin");
    let mut script = String::new();
    if stdin().read_to_string(&mut script).is_err() {
        error!("Failed to read stdin");
        std::process::exit(1);
    }

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let code = rt.block_on(async {
        let result = engine.execute(&script).await;
        engine.shutdown().await;

        match result {
            Ok(()) => {
                let report = engine.oracle_report();
                if report.total_checks > 0 && !report.passed {
                    error!(
                        "oracle failures detected: {} of {} checks failed",
                        report.failed_checks, report.total_checks
                    );
                    2
                } else {
                    debug!("Experiment complete");
                    0
                }
            }
            Err(e) => {
                error!("Failed to execute script error=\"{e}\"");
                1
            }
        }
    });

    drop(engine);
    drop(rt);
    debug!("Exiting dstest");
    std::process::exit(code);
}
