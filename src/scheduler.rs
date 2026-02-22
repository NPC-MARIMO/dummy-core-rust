// src/scheduler.rs

use tokio::time::{sleep, Duration};
use tracing::info;

use crate::state::SharedState;

// Layer 0 heartbeat loop.
// Runs until running=false.
pub async fn heartbeat_loop(state: SharedState) {
    loop {
        {
            let mut s = state.lock().unwrap();
            if !s.running {
                break;
            }
            s.heartbeat_count += 1;
            info!(heartbeat = s.heartbeat_count, "heartbeat");
        }

        sleep(Duration::from_secs(2)).await;
    }

    info!("heartbeat loop stopped");
}