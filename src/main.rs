use std::io::{Read, stdin};
use tracing::{debug, error};
mod bindings;
mod config;
mod engine;
mod fault;
mod oracle;
mod substrate;

#[tokio::main] // Added: Initializes the ambient Tokio reactor pool
async fn main() {
    tracing_subscriber::fmt::init();

    debug!("Starting dstest");

    let engine = engine::Engine::new();

    debug!("Reading scripts from stdin");
    let mut script = String::new();
    if stdin().read_to_string(&mut script).is_err() {
        error!("Failed to read stdin");
        std::process::exit(1);
    }

    // Added .await since engine.execute is now an async call
    match engine.execute(&script).await {
        Ok(_) => debug!("Experiment complete"),
        Err(e) => error!("Failed to execute script error=\"{e}\""),
    }

    debug!("Exiting dstest");
}
