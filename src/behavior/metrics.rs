use crate::behavior::event_adapter::BehaviorEventKind;
use crate::behavior::window::SlidingWindow;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub typing_speed: f32,
    pub backspace_rate: f32,
    pub mouse_speed: f32,
    pub scroll_rate: f32,
    pub window_switch_rate: f32,
}

impl Metrics {
    pub fn from_window(window: &SlidingWindow) -> Self {
        let mut key_count = 0;
        let mut backspace_count = 0;
        let mut mouse_distance = 0.0;
        let mut scroll_total = 0.0;
        let mut window_switch_count = 0;

        for event in window.iter() {
            match &event.kind {
                BehaviorEventKind::KeyPress => key_count += 1,
                BehaviorEventKind::Backspace => backspace_count += 1,
                BehaviorEventKind::MouseMoved { dx, dy } => {
                    mouse_distance += (dx * dx + dy * dy).sqrt();
                }
                BehaviorEventKind::Scroll { delta } => {
                    scroll_total += delta.abs();
                }
                BehaviorEventKind::WindowSwitch => window_switch_count += 1,
            }
        }

        let secs = window.duration_secs();

        Self {
            typing_speed: key_count as f32 / secs,
            backspace_rate: backspace_count as f32 / secs,
            mouse_speed: mouse_distance / secs,
            scroll_rate: scroll_total / secs,
            window_switch_rate: window_switch_count as f32 / secs,
        }
    }
}