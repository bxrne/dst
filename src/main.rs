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

    let engine = engine::Engine::new();

    debug!("Reading scripts from stdin");
    let mut script = String::new();
    if stdin().read_to_string(&mut script).is_err() {
        error!("Failed to read stdin");
        std::process::exit(1);
    }

    match engine.execute(&script) {
        Ok(_) => debug!("Experiment complete"),
        Err(e) => error!("Failed to execute script error=\"{e}\""),
    }

    debug!("Exiting dstest");
}
