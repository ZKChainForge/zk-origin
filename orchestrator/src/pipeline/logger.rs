//! Pipeline logging

use log::info;

/// Pipeline logger
pub struct PipelineLogger;

impl PipelineLogger {
    /// Log step
    pub fn step(message: &str) {
        info!("  {}", message);
    }
    
    /// Log success
    pub fn success(message: &str) {
        info!("  ✅ {}", message);
    }
    
    /// Log error
    pub fn error(message: &str) {
        info!("  ❌ {}", message);
    }
}