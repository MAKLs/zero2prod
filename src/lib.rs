#![forbid(unsafe_code)]
pub mod configuration;
mod routes;
mod startup;
pub mod telemetry;

pub use startup::{run, run_with_listener};
