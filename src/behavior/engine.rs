use std::time::{Duration, Instant};

use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::interval;

use crate::behavior::event_adapter::EventAdapter; // FIXED IMPORT
use crate::behavior::metrics::Metrics;
use crate::behavior::smoothing::ExpSmoother;
use crate::behavior::snapshot::BehaviorSnapshot;
use crate::behavior::window::SlidingWindow;
use crate::sensors::event::SensorEvent;

pub struct BehaviorEngine {
    rx: Receiver<SensorEvent>,
    snapshot_tx: Sender<BehaviorSnapshot>,

    window_5s: SlidingWindow,
    window_10s: SlidingWindow,

    typing_smoother: ExpSmoother,
    backspace_smoother: ExpSmoother,
    mouse_smoother: ExpSmoother,
    scroll_smoother: ExpSmoother,
    window_switch_smoother: ExpSmoother,
}

impl BehaviorEngine {
    pub fn new(
        rx: Receiver<SensorEvent>,
        snapshot_tx: Sender<BehaviorSnapshot>,
    ) -> Self {
        Self {
            rx,
            snapshot_tx,

            window_5s: SlidingWindow::new(Duration::from_secs(5)),
            window_10s: SlidingWindow::new(Duration::from_secs(10)),

            typing_smoother: ExpSmoother::new(0.3),
            backspace_smoother: ExpSmoother::new(0.3),
            mouse_smoother: ExpSmoother::new(0.3),
            scroll_smoother: ExpSmoother::new(0.3),
            window_switch_smoother: ExpSmoother::new(0.3),
        }
    }

    pub async fn run(mut self) {
        let mut ticker = interval(Duration::from_millis(200));

        // ✅ MUST be outside loop (stateful)
        let mut adapter = EventAdapter::new();

        loop {
            tokio::select! {
                Some(event) = self.rx.recv() => {

                    // ✅ Correct conversion
                    if let Some(behavior_event) = adapter.convert(event) {
                        self.window_5s.push(behavior_event.clone());
                        self.window_10s.push(behavior_event);
                    }
                }

                _ = ticker.tick() => {
                    self.process_tick();
                }
            }
        }
    }

    fn process_tick(&mut self) {
        let metrics = Metrics::from_window(&self.window_5s);

        let snapshot = BehaviorSnapshot {
            typing_speed: self.typing_smoother.update(metrics.typing_speed),
            backspace_rate: self.backspace_smoother.update(metrics.backspace_rate),
            mouse_speed: self.mouse_smoother.update(metrics.mouse_speed),
            scroll_rate: self.scroll_smoother.update(metrics.scroll_rate),
            window_switch_rate: self.window_switch_smoother.update(metrics.window_switch_rate),
            timestamp: Instant::now(),
        };

        let _ = self.snapshot_tx.try_send(snapshot);
    }
}