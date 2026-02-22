// src/behavior/engine.rs
// CLEAN UTF-8 SAFE VERSION

use tokio::time::{Duration, Instant};

use crate::behavior::snapshot::BehaviorSnapshot;
use crate::sensors::event::SensorEvent;

pub struct BehaviorEngine {
    // (timestamp, is_backspace)
    key_events: Vec<(Instant, bool)>,

    // window change timestamps
    window_events: Vec<Instant>,

    // (timestamp, x, y)
    mouse_positions: Vec<(Instant, i32, i32)>,
}

impl BehaviorEngine {
    pub fn new() -> Self {
        Self {
            key_events: Vec::new(),
            window_events: Vec::new(),
            mouse_positions: Vec::new(),
        }
    }

    pub fn process(&mut self, event: &SensorEvent) {
        let now = Instant::now();

        match event {
            SensorEvent::KeyPressed { .. } => {
                self.key_events.push((now, false));
            }
            SensorEvent::KeyBackspace => {
                self.key_events.push((now, true));
            }
            SensorEvent::WindowChanged { .. } => {
                self.window_events.push(now);
            }
            SensorEvent::MouseMoved { x, y } => {
                self.mouse_positions.push((now, *x, *y));
            }
            _ => {}
        }
    }

    pub fn compute_snapshot(&mut self) -> BehaviorSnapshot {
        let now = Instant::now();

        self.prune(now);

        BehaviorSnapshot {
            typing_speed_cps: self.typing_speed(now),
            backspace_ratio: self.backspace_ratio(now),
            window_change_rate: self.window_change_rate(now),
            mouse_velocity_variance: self.mouse_velocity_variance(now),
        }
    }

    fn prune(&mut self, now: Instant) {
        self.key_events
            .retain(|(t, _)| now.duration_since(*t) <= Duration::from_secs(5));

        self.window_events
            .retain(|t| now.duration_since(*t) <= Duration::from_secs(30));

        self.mouse_positions
            .retain(|(t, _, _)| now.duration_since(*t) <= Duration::from_secs(5));
    }

    fn typing_speed(&self, now: Instant) -> f32 {
        let count = self
            .key_events
            .iter()
            .filter(|(t, backspace)| {
                !*backspace
                    && now.duration_since(*t) <= Duration::from_secs(3)
            })
            .count();

        count as f32 / 3.0
    }

    fn backspace_ratio(&self, now: Instant) -> f32 {
        let mut total = 0usize;
        let mut backspaces = 0usize;

        for (t, is_backspace) in &self.key_events {
            if now.duration_since(*t) <= Duration::from_secs(5) {
                total += 1;
                if *is_backspace {
                    backspaces += 1;
                }
            }
        }

        if total == 0 {
            0.0
        } else {
            backspaces as f32 / total as f32
        }
    }

    fn window_change_rate(&self, now: Instant) -> f32 {
        let count = self
            .window_events
            .iter()
            .filter(|t| now.duration_since(**t) <= Duration::from_secs(30))
            .count();

        (count as f32 / 30.0) * 60.0
    }

    fn mouse_velocity_variance(&self, now: Instant) -> f32 {
        let points: Vec<_> = self
            .mouse_positions
            .iter()
            .filter(|(t, _, _)| {
                now.duration_since(*t) <= Duration::from_secs(5)
            })
            .collect();

        if points.len() < 2 {
            return 0.0;
        }

        let mut velocities = Vec::with_capacity(points.len() - 1);

        for pair in points.windows(2) {
            let (t1, x1, y1) = *pair[0];
            let (t2, x2, y2) = *pair[1];

            let dt = t2.duration_since(t1).as_secs_f32();
            if dt <= 0.0 {
                continue;
            }

            let dx = (x2 - x1) as f32;
            let dy = (y2 - y1) as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            velocities.push(dist / dt);
        }

        if velocities.len() < 2 {
            return 0.0;
        }

        let mean =
            velocities.iter().sum::<f32>() / velocities.len() as f32;

        let mut var_sum = 0.0;

        for v in &velocities {
            let d = *v - mean;
            var_sum += d * d;
        }

        var_sum / velocities.len() as f32
    }
}