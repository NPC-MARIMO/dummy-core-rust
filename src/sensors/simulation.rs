// src/sensors/simulation.rs

use tokio::time::{sleep, Duration};
use tracing::info;

use crate::state::{SharedState, SimulationMode};

// Phase 2.5 deterministic mode rotation:
// Normal -> Debugging -> Reading -> repeat
pub async fn mode_rotation_loop(state: SharedState) {
    loop {
        {
            let s = state.lock().unwrap();
            if !s.running {
                break;
            }
        }

        for _ in 0..20 {
            sleep(Duration::from_secs(1)).await;

            let running = {
                let s = state.lock().unwrap();
                s.running
            };

            if !running {
                break;
            }
         }
        let mut s = state.lock().unwrap();
        if !s.running {
            break;
        }

        s.simulation_mode = match s.simulation_mode {
            SimulationMode::Normal => SimulationMode::Debugging,
            SimulationMode::Debugging => SimulationMode::Reading,
            SimulationMode::Reading => SimulationMode::Normal,
        };

        info!(mode = ?s.simulation_mode, "simulation mode switched");
    }

    info!("mode rotation loop stopped");
}