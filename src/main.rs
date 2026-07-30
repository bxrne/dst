use std::io::{Read, stdin};
use tracing::{debug, error};
mod bindings;
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
        std::process::exit(1);
    });

    let engine = engine::Engine::new(docker);

    debug!("Reading scripts from stdin");
    let mut script = String::new();
    if stdin().read_to_string(&mut script).is_err() {
        error!("Failed to read stdin");
        std::process::exit(1);
    }

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let result = rt.block_on(async {
        match engine.execute(&script).await {
            Ok(_) => {
                debug!("Experiment complete");
                0
            }
            Err(e) => {
                error!("Failed to execute script error=\"{e}\"");
                1
            }
        }
    });

    drop(rt);
    drop(engine);
    debug!("Exiting dstest");
    std::process::exit(result);
}
