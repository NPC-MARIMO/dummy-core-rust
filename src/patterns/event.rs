#[derive(Debug, Clone)]
pub enum PatternEvent {
    RepeatedWindowTitle { title: String, count: u32 },
    HighBackspaceRate { ratio: f32 },
    TypingBurst { chars_per_second: f32 },
    WindowInstability { changes_per_minute: f32 },
}