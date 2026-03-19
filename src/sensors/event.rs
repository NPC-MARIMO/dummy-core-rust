#[derive(Debug)]
pub enum SensorEvent {
    WindowChanged { title: String },
    KeyPressed { key: String },
    KeyBackspace,
    MouseMoved { x: i32, y: i32 },
    Scroll { delta: i32 },
}