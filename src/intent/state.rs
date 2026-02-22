// src/intent/state.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Coding,
    Debugging,
    Reading,
    Idle,
}

#[derive(Debug, Clone, Copy)]
pub struct IntentState {
    pub current: Intent,
    pub confidence: f32,
}