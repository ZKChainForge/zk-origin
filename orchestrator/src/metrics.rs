//! Metrics collection

use std::time::Duration;

/// Metrics collector
#[derive(Clone, Debug)]
pub struct Metrics {
    /// Steps executed
    pub steps: usize,
    
    /// Total time
    pub total_time: Duration,
}

impl Metrics {
    /// Create new metrics
    pub fn new() -> Self {
        Metrics {
            steps: 0,
            total_time: Duration::from_secs(0),
        }
    }
    
    /// Record step
    pub fn record_step(&self, _step: usize, elapsed: Duration) {
        println!("  Step time: {:.2}s", elapsed.as_secs_f64());
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}