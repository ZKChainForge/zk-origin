//! Pipeline validation

use crate::error::Result;

/// Pipeline validator
pub struct PipelineValidator;

impl PipelineValidator {
    /// Validate witness
    pub fn validate_witness(_data: &[u8]) -> Result<()> {
        Ok(())
    }
    
    /// Validate proof
    pub fn validate_proof(_data: &[u8]) -> Result<()> {
        Ok(())
    }
    
    /// Validate state
    pub fn validate_state(_data: &[u8]) -> Result<()> {
        Ok(())
    }
}