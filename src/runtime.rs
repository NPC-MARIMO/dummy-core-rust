// src/runtime.rs
// FULL REPLACEMENT

use anyhow::Result;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

use crate::behavior::engine::BehaviorEngine;
use crate::behavior::snapshot::BehaviorSnapshot;
use crate::intent::engine as intent_engine;
use crate::intent::state::IntentState;
use crate::patterns::engine::PatternEngine;
use crate::patterns::event::PatternEvent;
use crate::scheduler;
use crate::state::create_state;
use crate::sensors::{self, event::SensorEvent};

pub async fn run() -> Result<()> {
    info!("Astra Core starting up");

    let state = create_state();

    let (sensor_tx, mut sensor_rx) = mpsc::channel::<SensorEvent>(512);

    let (pattern_tx, pattern_rx_for_intent) =
        mpsc::channel::<PatternEvent>(256);
    let (pattern_log_tx, mut pattern_log_rx) =
        mpsc::channel::<PatternEvent>(256);

    let (behavior_tx, behavior_rx_for_intent) =
        mpsc::channel::<BehaviorSnapshot>(128);
    let (behavior_log_tx, mut behavior_log_rx) =
        mpsc::channel::<BehaviorSnapshot>(128);

    let (intent_tx, mut intent_rx) =
        mpsc::channel::<IntentState>(64);

    let mut tasks: Vec<JoinHandle<()>> = Vec::new();

    tasks.push(tokio::spawn(scheduler::heartbeat_loop(state.clone())));
    tasks.push(tokio::spawn(
        sensors::simulation::mode_rotation_loop(state.clone()),
    ));

    tasks.push(tokio::spawn(sensors::window::run(
        state.clone(),
        sensor_tx.clone(),
    )));
    tasks.push(tokio::spawn(sensors::keyboard::run(
        state.clone(),
        sensor_tx.clone(),
    )));
    tasks.push(tokio::spawn(sensors::mouse::run(
        state.clone(),
        sensor_tx.clone(),
    )));

    drop(sensor_tx);

    // collector
    let collector_state = state.clone();
    let pattern_sender = pattern_tx.clone();
    let pattern_log_sender = pattern_log_tx.clone();
    let behavior_sender = behavior_tx.clone();
    let behavior_log_sender = behavior_log_tx.clone();

    let collector = tokio::spawn(async move {
        let mut pattern_engine = PatternEngine::new();
        let mut behavior_engine = BehaviorEngine::new();
        let mut tick = interval(Duration::from_secs(3));

        loop {
            tokio::select! {
                _ = tick.tick() => {
                    let snap = behavior_engine.compute_snapshot();

                    let _ = behavior_sender.send(snap).await;
                    let _ = behavior_log_sender.send(snap).await;
                }

                maybe_event = sensor_rx.recv() => {
                    match maybe_event {
                        Some(event) => {
                            {
                                let mut s = collector_state.lock().unwrap();
                                s.total_events += 1;
                            }

                            tracing::info!(?event, "sensor_event");

                            let patterns = pattern_engine.process(&event);

                          for p in patterns {
                            let _ = pattern_sender.send(p.clone()).await;
                            let _ = pattern_log_sender.send(p).await;
}

                            behavior_engine.process(&event);
                        }
                        None => break,
                    }
                }
            }
        }

        info!("collector loop ended (sensor channel closed)");
    });

    drop(pattern_tx);
    drop(pattern_log_tx);
    drop(behavior_tx);
    drop(behavior_log_tx);

    // intent engine task
    let intent_task = tokio::spawn(intent_engine::run(
        pattern_rx_for_intent,
        behavior_rx_for_intent,
        intent_tx,
    ));

    // loggers
    let pattern_logger = tokio::spawn(async move {
        while let Some(p) = pattern_log_rx.recv().await {
            tracing::info!(?p, "pattern_event");
        }
        info!("pattern logger ended");
    });

    let behavior_logger = tokio::spawn(async move {
        while let Some(b) = behavior_log_rx.recv().await {
            tracing::info!(?b, "behavior_snapshot");
        }
        info!("behavior logger ended");
    });

    let intent_logger = tokio::spawn(async move {
        while let Some(i) = intent_rx.recv().await {
            tracing::info!(
                intent = ?i.current,
                confidence = i.confidence,
                "intent_update"
            );
        }
        info!("intent logger ended");
    });

    tokio::signal::ctrl_c().await?;
    warn!("Ctrl+C received, initiating graceful shutdown");

    {
        let mut s = state.lock().unwrap();
        s.running = false;
    }

    for t in tasks {
        let _ = t.await;
    }

    let _ = collector.await;
    let _ = intent_task.await;
    let _ = pattern_logger.await;
    let _ = behavior_logger.await;
    let _ = intent_logger.await;

    {
        let s = state.lock().unwrap();
        info!(
            heartbeat_count = s.heartbeat_count,
            total_events = s.total_events,
            mode = ?s.simulation_mode,
            "shutdown summary"
        );
    }

    info!("Astra Core shutdown complete");

    std::process::exit(0);
}