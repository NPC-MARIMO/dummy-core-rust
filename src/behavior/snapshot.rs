use std::time::Instant;

#[derive(Debug)]
pub struct BehaviorSnapshot {
    pub typing_speed: f32,
    pub backspace_rate: f32,
    pub mouse_speed: f32,
    pub scroll_rate: f32,
    pub window_switch_rate: f32,
    pub timestamp: Instant,
}