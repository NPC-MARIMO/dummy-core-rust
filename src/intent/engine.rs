// src/intent/engine.rs
//
// FULL FIXED VERSION
// - UTF-8 safe
// - stable intent switching
// - score decay
// - correct activity tracking
// - absolute activation threshold (no EMA self-comparison bug)
// - smoothing + dwell time
//
// DROP-IN REPLACEMENT

use std::collections::VecDeque;

use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

use crate::behavior::snapshot::BehaviorSnapshot;
use crate::intent::state::{Intent, IntentState};
use crate::patterns::event::PatternEvent;

pub struct IntentEngine {
    // EMA scores
    coding_score: f32,
    debugging_score: f32,
    reading_score: f32,
    idle_score: f32,

    // intent state
    current_intent: Intent,
    last_switch: Instant,

    // signal memory
    recent_pattern_events: VecDeque<(Instant, PatternEvent)>,
    last_behavior: Option<BehaviorSnapshot>,

    // true activity time (NOT behavior ticks)
    last_activity_time: Instant,
}

impl IntentEngine {
    pub fn new() -> Self {
        let now = Instant::now();

        Self {
            coding_score: 0.0,
            debugging_score: 0.0,
            reading_score: 0.0,
            idle_score: 0.2, // slight startup bias
            current_intent: Intent::Idle,
            last_switch: now,
            recent_pattern_events: VecDeque::new(),
            last_behavior: None,
            last_activity_time: now,
        }
    }

    // pattern events represent real activity
    pub fn ingest_pattern(&mut self, p: PatternEvent) {
        let now = Instant::now();

        self.recent_pattern_events.push_back((now, p));
        self.last_activity_time = now;

        // prune history (10s)
        while let Some((t, _)) = self.recent_pattern_events.front() {
            if now.duration_since(*t) > Duration::from_secs(10) {
                self.recent_pattern_events.pop_front();
            } else {
                break;
            }
        }
    }

    // behavior snapshots are analysis ONLY
    pub fn ingest_behavior(&mut self, b: BehaviorSnapshot) {
        self.last_behavior = Some(b);
    }

    pub fn evaluate(&mut self) -> Option<IntentState> {
        let now = Instant::now();

        let Some(ref b) = self.last_behavior else {
            return None;
        };

        // -------- score decay (critical for switching) --------
        self.coding_score *= 0.90;
        self.debugging_score *= 0.90;
        self.reading_score *= 0.90;
        self.idle_score *= 0.90;

        let mut coding = 0.0;
        let mut debugging = 0.0;
        let mut reading = 0.0;
        let mut idle = 0.0;

        // ---------- behavior signals ----------

        // typing speed
        if b.typing_speed_cps > 6.0 {
            coding += 1.2;
        } else if b.typing_speed_cps > 2.0 {
            coding += 0.6;
        } else if b.typing_speed_cps < 0.5 {
            reading += 0.8;
            idle += 0.4;
        }

        // backspace ratio
        if b.backspace_ratio > 0.3 {
            debugging += 1.4;
        } else if b.backspace_ratio < 0.1 {
            coding += 0.8;
        }

        // window change rate
        if b.window_change_rate > 10.0 {
            debugging += 1.0;
        } else if b.window_change_rate < 4.0 {
            coding += 0.6;
            reading += 0.5;
        }

        // mouse variance
        if b.mouse_velocity_variance > 800.0 {
            debugging += 0.5;
        } else if b.mouse_velocity_variance < 150.0 {
            reading += 0.6;
            idle += 0.5;
        }

        // inactivity ? idle
        if now.duration_since(self.last_activity_time)
            > Duration::from_secs(3)
        {
            idle += 2.0;
        }

        // ---------- pattern contributions ----------

        for (_, p) in &self.recent_pattern_events {
            match p {
                PatternEvent::HighBackspaceRate { .. } => {
                    debugging += 1.3;
                }

                PatternEvent::TypingBurst { .. } => {
                    coding += 1.0;
                }

                PatternEvent::WindowInstability { .. } => {
                    debugging += 0.8;
                }

                PatternEvent::RepeatedWindowTitle { title, .. } => {
                    if title.contains("error") {
                        debugging += 1.5;
                    } else {
                        coding += 0.5;
                    }
                }
            }
        }

        // ---------- EMA smoothing (alpha = 0.35) ----------

        self.coding_score =
            self.coding_score * 0.65 + coding * 0.35;

        self.debugging_score =
            self.debugging_score * 0.65 + debugging * 0.35;

        self.reading_score =
            self.reading_score * 0.65 + reading * 0.35;

        self.idle_score =
            self.idle_score * 0.65 + idle * 0.35;

        let scores = [
            (Intent::Coding, self.coding_score),
            (Intent::Debugging, self.debugging_score),
            (Intent::Reading, self.reading_score),
            (Intent::Idle, self.idle_score),
        ];

        let (best_intent, best_score) = scores
            .iter()
            .copied()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();

        // -------- temporal smoothing --------

        let dwell_ok =
            now.duration_since(self.last_switch)
                > Duration::from_secs(3);

        // ABSOLUTE activation threshold
        if best_intent != self.current_intent
            && dwell_ok
            && best_score > 0.35
        {
            self.current_intent = best_intent;
            self.last_switch = now;

            let sum = self.coding_score
                + self.debugging_score
                + self.reading_score
                + self.idle_score;

            let confidence = if sum <= 0.0001 {
                0.0
            } else {
                best_score / sum
            };

            return Some(IntentState {
                current: best_intent,
                confidence,
            });
        }

        None
    }
}

pub async fn run(
    mut pattern_rx: mpsc::Receiver<PatternEvent>,
    mut behavior_rx: mpsc::Receiver<BehaviorSnapshot>,
    intent_tx: mpsc::Sender<IntentState>,
) {
    let mut engine = IntentEngine::new();
    let mut tick =
        tokio::time::interval(Duration::from_millis(800));

    loop {
        tokio::select! {
            maybe_p = pattern_rx.recv() => {
                match maybe_p {
                    Some(p) => engine.ingest_pattern(p),
                    None => break,
                }
            }

            maybe_b = behavior_rx.recv() => {
                match maybe_b {
                    Some(b) => engine.ingest_behavior(b),
                    None => break,
                }
            }

            _ = tick.tick() => {
                if let Some(state) = engine.evaluate() {
                    if intent_tx.send(state).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}