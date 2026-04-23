use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::behavior::event_adapter::BehaviorEvent;

pub struct SlidingWindow {
    buffer: VecDeque<BehaviorEvent>,
    window_size: Duration,
}

impl SlidingWindow {
    pub fn new(window_size: Duration) -> Self {
        Self {
            buffer: VecDeque::new(),
            window_size,
        }
    }

    pub fn push(&mut self, event: BehaviorEvent) {
        self.buffer.push_back(event);
        self.prune();
    }

    fn prune(&mut self) {
        let now = Instant::now();

        while let Some(front) = self.buffer.front() {
            if now.duration_since(front.timestamp) > self.window_size {
                self.buffer.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &BehaviorEvent> {
        self.buffer.iter()
    }

    pub fn duration_secs(&self) -> f32 {
        self.window_size.as_secs_f32()
    }
}