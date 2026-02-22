// src/sensors/event.rs

#[derive(Debug)]
pub enum SensorEvent {
    WindowChanged { title: String },
    KeyPressed { key: char },
    KeyBackspace,
    MouseMoved { x: i32, y: i32 },
    Scroll { delta: i32 },
}