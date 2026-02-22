#[derive(Debug, Clone, Copy)]
pub struct BehaviorSnapshot {
    pub typing_speed_cps: f32,
    pub backspace_ratio: f32,
    pub window_change_rate: f32,
    pub mouse_velocity_variance: f32,
}