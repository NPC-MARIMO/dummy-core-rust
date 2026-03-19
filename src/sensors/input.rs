use device_query::{DeviceQuery, DeviceState};
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc::Sender;

use crate::sensors::event::SensorEvent;
use crate::state::SharedState;

pub async fn run_input(
    state: SharedState,
    tx: Sender<SensorEvent>,
) {
    let device = DeviceState::new();

    let mut last_keys = Vec::new();
    let mut last_mouse = (0, 0);

    loop {
        // ? shutdown check
        let running = {
            let s = state.lock().unwrap();
            s.running
        };

        if !running {
            break;
        }

        let keys = device.get_keys();
        let mouse = device.get_mouse();

        // Key press detection
        for key in &keys {
            if !last_keys.contains(key) {
                let key_str = format!("{:?}", key);

                if key_str == "Backspace" {
                    tx.send(SensorEvent::KeyBackspace).await.unwrap();
                } else {
                    tx.send(SensorEvent::KeyPressed {
                        key: key_str,
                    })
                    .await
                    .unwrap();
                }
            }
        }

        // Mouse movement
        if mouse.coords != last_mouse {
            let (x, y) = mouse.coords;

            tx.send(SensorEvent::MouseMoved { x, y })
                .await
                .unwrap();

            last_mouse = mouse.coords;
        }

        last_keys = keys;

        sleep(Duration::from_millis(50)).await;
    }
}