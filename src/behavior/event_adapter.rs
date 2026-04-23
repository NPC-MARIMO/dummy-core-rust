use std::time::Instant;

use crate::sensors::event::SensorEvent;

#[derive(Clone, Debug)]
pub struct BehaviorEvent {
    pub timestamp: Instant,
    pub kind: BehaviorEventKind,
}

#[derive(Clone, Debug)]
pub enum BehaviorEventKind {
    KeyPress,
    Backspace,
    MouseMoved { dx: f32, dy: f32 },
    Scroll { delta: f32 },
    WindowSwitch,
}

pub struct EventAdapter {
    last_mouse_pos: Option<(i32, i32)>,
}

impl EventAdapter {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: None,
        }
    }

    pub fn convert(&mut self, event: SensorEvent) -> Option<BehaviorEvent> {
        let now = Instant::now();

        match event {
            SensorEvent::KeyPressed { .. } => Some(BehaviorEvent {
                timestamp: now,
                kind: BehaviorEventKind::KeyPress,
            }),

            SensorEvent::KeyBackspace => Some(BehaviorEvent {
                timestamp: now,
                kind: BehaviorEventKind::Backspace,
            }),

            SensorEvent::MouseMoved { x, y } => {
                let (dx, dy) = if let Some((lx, ly)) = self.last_mouse_pos {
                    ((x - lx) as f32, (y - ly) as f32)
                } else {
                    (0.0, 0.0)
                };

                self.last_mouse_pos = Some((x, y));

                Some(BehaviorEvent {
                    timestamp: now,
                    kind: BehaviorEventKind::MouseMoved { dx, dy },
                })
            }

            SensorEvent::Scroll { delta } => Some(BehaviorEvent {
                timestamp: now,
                kind: BehaviorEventKind::Scroll {
                    delta: delta as f32,
                },
            }),

            SensorEvent::WindowChanged { .. } => Some(BehaviorEvent {
                timestamp: now,
                kind: BehaviorEventKind::WindowSwitch,
            }),
        }
    }
}