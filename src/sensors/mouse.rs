// src/sensors/mouse.rs

use rand::Rng;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

use crate::state::{SharedState, SimulationMode};
use crate::sensors::event::SensorEvent;

pub async fn run(state: SharedState, tx: Sender<SensorEvent>) {
    let mut x = 400;
    let mut y = 300;

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
                let (dx, dy) = {
                    let mut rng = rand::thread_rng();
                    (rng.gen_range(-15..=15), rng.gen_range(-15..=15))
                };

                x += dx;
                y += dy;

                let _ = tx.send(SensorEvent::MouseMoved { x, y }).await;
                sleep(Duration::from_millis(120)).await;
            }

            SimulationMode::Debugging => {
                let move_now = {
                    let mut rng = rand::thread_rng();
                    rng.gen_bool(0.2)
                };

                if move_now {
                    let (dx, dy) = {
                        let mut rng = rand::thread_rng();
                        (rng.gen_range(-3..=3), rng.gen_range(-3..=3))
                    };

                    x += dx;
                    y += dy;

                    let _ = tx.send(SensorEvent::MouseMoved { x, y }).await;
                }

                sleep(Duration::from_millis(500)).await;
            }

            SimulationMode::Reading => {
                let scroll_now = {
                    let mut rng = rand::thread_rng();
                    rng.gen_bool(0.6)
                };

                if scroll_now {
                    let delta = {
                        let mut rng = rand::thread_rng();
                        rng.gen_range(1..5)
                    };

                    let _ = tx.send(SensorEvent::Scroll { delta }).await;
                }

                let (dx, dy) = {
                    let mut rng = rand::thread_rng();
                    (rng.gen_range(-5..=5), rng.gen_range(-5..=5))
                };

                x += dx;
                y += dy;

                let _ = tx.send(SensorEvent::MouseMoved { x, y }).await;
                sleep(Duration::from_millis(700)).await;
            }
        }
    }
}   