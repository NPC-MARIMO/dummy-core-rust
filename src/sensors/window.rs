// src/sensors/window.rs

use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};
use rand::Rng;

use crate::state::{SharedState, SimulationMode};
use crate::sensors::event::SensorEvent;

pub async fn run(state: SharedState, tx: Sender<SensorEvent>) {
    let mut tick: u64 = 0;

    loop {
        let (running, mode) = {
            let s = state.lock().unwrap();
            (s.running, s.simulation_mode)
        };

        if !running {
            break;
        }

        match mode {
            SimulationMode::Normal => {
                let send_window = {
                    let mut rng = rand::thread_rng();
                    rng.gen_bool(0.35)
                };

                if send_window {
                    let _ = tx.send(SensorEvent::WindowChanged {
                        title: "Browser - Documentation".to_string(),
                    }).await;
                }

                sleep(Duration::from_millis(1200)).await;
            }

            SimulationMode::Debugging => {
                tick += 1;

                let title = if tick % 3 == 0 {
                    "error[E0382]: use of moved value"
                } else {
                    "main.rs - Visual Studio Code"
                };

                let _ = tx.send(SensorEvent::WindowChanged {
                    title: title.to_string(),
                }).await;

                sleep(Duration::from_millis(900)).await;
            }

            SimulationMode::Reading => {
                if tick % 4 == 0 {
                    let _ = tx.send(SensorEvent::WindowChanged {
                        title: "Rust Ownership - Mozilla Docs".to_string(),
                    }).await;
                }

                tick += 1;
                sleep(Duration::from_millis(2500)).await;
            }
        }
    }
}