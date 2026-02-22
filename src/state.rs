// src/state.rs

use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy)]
pub enum SimulationMode {
    Normal,
    Debugging,
    Reading,
}

#[derive(Debug)]
pub struct ServiceState {
    pub heartbeat_count: u64,
    pub total_events: u64,
    pub running: bool,
    pub simulation_mode: SimulationMode,
}

impl ServiceState {
    pub fn new() -> Self {
        Self {
            heartbeat_count: 0,
            total_events: 0,
            running: true,
            simulation_mode: SimulationMode::Normal,
        }
    }
}

pub type SharedState = Arc<Mutex<ServiceState>>;

pub fn create_state() -> SharedState {
    Arc::new(Mutex::new(ServiceState::new()))
}