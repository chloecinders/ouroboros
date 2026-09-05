use std::time::{Duration, Instant};

use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Step {
    pub name: &'static str,
    pub nanos: u64,
}

#[derive(Clone, Debug)]
pub struct Trace {
    started: Instant,
    previous: Instant,
    points: Vec<Step>,
}

impl Default for Trace {
    fn default() -> Self {
        Self::new()
    }
}

impl Trace {
    pub fn new() -> Self {
        let now = Instant::now();

        Self {
            started: now,
            previous: now,
            points: Vec::new(),
        }
    }

    pub fn point(&mut self, name: &'static str) {
        let now = Instant::now();

        self.points.push(Step {
            name,
            nanos: now.duration_since(self.previous).as_nanos() as u64,
        });
        self.previous = now;
    }

    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    pub fn as_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.points).unwrap_or_else(|_| serde_json::Value::Array(Vec::new()))
    }
}
