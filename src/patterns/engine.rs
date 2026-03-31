use tokio::time::{Duration, Instant};

use crate::patterns::event::PatternEvent;
use crate::sensors::event::SensorEvent;

pub struct PatternEngine {
    pub recent_window_titles: Vec<(Instant, String)>,

    pub recent_key_events: Vec<(Instant, SensorEvent)>,

    pub last_window_change: Option<Instant>,

    window_change_times: Vec<Instant>,
    high_backspace_active: bool,
    typing_burst_active: bool,
    window_instability_active: bool,
}

impl PatternEngine {
    pub fn new() -> Self {
        Self {
            recent_window_titles: Vec::new(),
            recent_key_events: Vec::new(),
            last_window_change: None,
            window_change_times: Vec::new(),
            high_backspace_active: false,
    typing_burst_active: false,
    window_instability_active: false,
        }
    }

    pub fn process(&mut self, event: &SensorEvent) -> Vec<PatternEvent> {
        let now = Instant::now();
        let mut out = Vec::new();

        match event {
            SensorEvent::WindowChanged { title, .. } => {
                self.handle_window(now, title, &mut out);
            }

            SensorEvent::KeyPressed { .. } | SensorEvent::KeyBackspace => {
                self.handle_key(now, event, &mut out);
            }

            _ => {}
        }

        out
    }

    fn handle_window(
        &mut self,
        now: Instant,
        title: &str,
        out: &mut Vec<PatternEvent>,
    ) {
        // push window title
        self.recent_window_titles
            .push((now, title.to_string()));

        // retain 10s window
        self.recent_window_titles
            .retain(|(t, _)| now.duration_since(*t) <= Duration::from_secs(10));

        // count title occurrences
        let count = self
            .recent_window_titles
            .iter()
            .filter(|(_, t)| t == title)
            .count() as u32;

        if count >= 3 {
            out.push(PatternEvent::RepeatedWindowTitle {
                title: title.to_string(),
                count,
            });
        }

        // window instability tracking (30s)
        self.window_change_times.push(now);
        self.window_change_times
            .retain(|t| now.duration_since(*t) <= Duration::from_secs(30));

        let changes = self.window_change_times.len();

        if changes > 5 {
        if !self.window_instability_active {
            let cpm = (changes as f32 / 30.0) * 60.0;
            out.push(PatternEvent::WindowInstability {
                changes_per_minute: cpm,
            });
            self.window_instability_active = true;
        }
        }
        else {
            self.window_instability_active = false;
        }

        self.last_window_change = Some(now);
    }

    fn handle_key(
        &mut self,
        now: Instant,
        event: &SensorEvent,
        out: &mut Vec<PatternEvent>,
    ) {
        // bounded key memory (max needed = 5s window)
        self.recent_key_events.push((now, clone_key(event)));

        self.recent_key_events
            .retain(|(t, _)| now.duration_since(*t) <= Duration::from_secs(5));

        // ----- Rule 2: High backspace ratio (5s) -----
        let mut total = 0usize;
        let mut backspaces = 0usize;

        for (_, ev) in &self.recent_key_events {
            match ev {
                SensorEvent::KeyPressed { .. } => {
                    total += 1;
                }
                SensorEvent::KeyBackspace => {
                    total += 1;
                    backspaces += 1;
                }
                _ => {}
            }
        }

        if total > 0 {
            let ratio = backspaces as f32 / total as f32;
            if ratio > 0.3 {
                if !self.high_backspace_active {
                    out.push(PatternEvent::HighBackspaceRate { ratio });
                    self.high_backspace_active = true;
                }
            }       
            else {
                self.high_backspace_active = false;
            }
        }

        // ----- Rule 3: Typing burst (2s) -----
        let burst_count = self
            .recent_key_events
            .iter()
            .filter(|(t, ev)| {
                now.duration_since(*t) <= Duration::from_secs(2)
                    && matches!(
                        ev,
                        SensorEvent::KeyPressed { .. } | SensorEvent::KeyBackspace
                    )
            })
            .count();

        if burst_count > 20 {
            if !self.typing_burst_active {
                out.push(PatternEvent::TypingBurst {
                    chars_per_second: burst_count as f32 / 2.0,
                });
                self.typing_burst_active = true;
            }
        } 
        else {
            self.typing_burst_active = false;
        }
    }
}

fn clone_key(event: &SensorEvent) -> SensorEvent {
    match event {
        SensorEvent::KeyPressed { key } => {
            SensorEvent::KeyPressed { key: key.clone() }
        }
        SensorEvent::KeyBackspace => SensorEvent::KeyBackspace,
        _ => unreachable!(),
    }
}