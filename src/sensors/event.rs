use std::time::Instant;

#[derive(Debug, Clone)]
pub enum SensorEvent {
    WindowChanged { title: String, ts: Instant },
    KeyPressed { key: String },
    KeyBackspace,
    MouseMoved { x: i32, y: i32 },
    Scroll { delta: i32 },
}