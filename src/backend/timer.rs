use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Serialize, Deserialize)]
pub enum TimerType {
    Focus,
    Rest,
    Transitioning(Box<TimerType>),
}

#[derive(Clone, Copy)]
pub struct Timer {
    pub current_time: Duration,
    pub initial_time: Duration,
}

impl Timer {
    pub fn new(initial_time: Duration) -> Self {
        Self {
            current_time: initial_time,
            initial_time,
        }
    }
}
