// src/sensors/keyboard.rs

use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};
use rand::Rng;

use crate::state::{SharedState, SimulationMode};
use crate::sensors::event::SensorEvent;

const LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

pub async fn run(state: SharedState, tx: Sender<SensorEvent>) {
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
                // RNG MUST NOT live across await points.
                let c = {
                    let mut rng = rand::thread_rng();
                    LETTERS[rng.gen_range(0..LETTERS.len())] as char
                };

                let _ = tx.send(SensorEvent::KeyPressed { key: c }).await;
                sleep(Duration::from_millis(180)).await;
            }

            SimulationMode::Debugging => {
                let burst_len = {
                    let mut rng = rand::thread_rng();
                    rng.gen_range(3..8)
                };

                for _ in 0..burst_len {
                    let c = {
                        let mut rng = rand::thread_rng();
                        LETTERS[rng.gen_range(0..LETTERS.len())] as char
                    };

                    let _ = tx.send(SensorEvent::KeyPressed { key: c }).await;
                    sleep(Duration::from_millis(70)).await;
                }

                let backspaces = {
                    let mut rng = rand::thread_rng();
                    rng.gen_range(2..6)
                };

                for _ in 0..backspaces {
                    let _ = tx.send(SensorEvent::KeyBackspace).await;
                    sleep(Duration::from_millis(60)).await;
                }

                sleep(Duration::from_millis(300)).await;
            }

            SimulationMode::Reading => {
                let should_type = {
                    let mut rng = rand::thread_rng();
                    rng.gen_bool(0.15)
                };

                if should_type {
                    let c = {
                        let mut rng = rand::thread_rng();
                        LETTERS[rng.gen_range(0..LETTERS.len())] as char
                    };

                    let _ = tx.send(SensorEvent::KeyPressed { key: c }).await;
                }

                sleep(Duration::from_millis(1200)).await;
            }
        }
    }
}