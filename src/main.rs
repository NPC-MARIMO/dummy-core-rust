// src/main.rs

mod runtime;
mod scheduler;
mod state;
mod sensors;
mod patterns;
mod behavior;
mod intent;

use anyhow::Result;
use runtime::run;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    run().await
}