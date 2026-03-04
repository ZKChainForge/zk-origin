//! src/prover/nova_circuit.rs
//! Nova circuit for lineage step verification

#[cfg(feature = "real-nova")]
use bellpepper_core::{
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};

#[cfg(feature = "real-nova")]
use ff::PrimeField;

#[cfg(feature = "real-nova")]
use nova_snark::traits::circuit::StepCircuit;

/// The main circuit that verifies one lineage step
#[cfg(feature = "real-nova")]
#[derive(Clone, Debug)]
pub struct LineageStepCircuit<F: PrimeField> {
    // Previous state commitments
    prev_lineage: Option<F>,
    prev_counters: Option<F>,
    
    // Current transition data
    prev_state_hash: Option<F>,
    new_state_hash: Option<F>,
    origin_class: Option<F>,
    timestamp: Option<F>,
    
    // Policy verification
    policy_root: Option<F>,
    policy_path: Vec<Option<F>>,
    policy_indices: Vec<bool>,
    
    // Rate limiting
    epoch_id: Option<F>,
    rate_limits: [Option<F>; 6],
}

#[cfg(feature = "real-nova")]
impl<F: PrimeField> Default for LineageStepCircuit<F> {
    fn default() -> Self {
        Self {
            prev_lineage: None,
            prev_counters: None,
            prev_state_hash: None,
            new_state_hash: None,
            origin_class: None,
            timestamp: None,
            policy_root: None,
            policy_path: vec![None; 8], // Default depth
            policy_indices: vec![false; 8],
            epoch_id: None,
            rate_limits: [None; 6],
        }
    }
}

#[cfg(feature = "real-nova")]
impl<F: PrimeField> LineageStepCircuit<F> {
    /// Create a new circuit with witness values
    pub fn new(
        prev_lineage: F,
        prev_counters: F,
        prev_state_hash: F,
        new_state_hash: F,
        origin_class: F,
        timestamp: F,
        policy_root: F,
        policy_path: Vec<F>,
        policy_indices: Vec<bool>,
        epoch_id: F,
        rate_limits: [F; 6],
    ) -> Self {
        Self {
            prev_lineage: Some(prev_lineage),
            prev_counters: Some(prev_counters),
            prev_state_hash: Some(prev_state_hash),
            new_state_hash: Some(new_state_hash),
            origin_class: Some(origin_class),
            timestamp: Some(timestamp),
            policy_root: Some(policy_root),
            policy_path: policy_path.into_iter().map(Some).collect(),
            policy_indices,
            epoch_id: Some(epoch_id),
            rate_limits: rate_limits.map(Some),
        }
    }
}

#[cfg(feature = "real-nova")]
impl<F: PrimeField> StepCircuit<F> for LineageStepCircuit<F> {
    fn arity(&self) -> usize {
        2 // [lineage_commitment, counter_commitment]
    }

    fn synthesize<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        z: &[AllocatedNum<F>],
    ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
        // Input validation
        assert_eq!(z.len(), 2, "Expected 2 inputs");
        
        let prev_lineage = &z[0];
        let prev_counters = &z[1];
        
        // === Allocate witness variables ===
        let prev_state = AllocatedNum::alloc(cs.namespace(|| "prev_state"), || {
            self.prev_state_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let new_state = AllocatedNum::alloc(cs.namespace(|| "new_state"), || {
            self.new_state_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let origin = AllocatedNum::alloc(cs.namespace(|| "origin_class"), || {
            self.origin_class.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let timestamp = AllocatedNum::alloc(cs.namespace(|| "timestamp"), || {
            self.timestamp.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let _policy_root = AllocatedNum::alloc(cs.namespace(|| "policy_root"), || {
            self.policy_root.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        let epoch_id = AllocatedNum::alloc(cs.namespace(|| "epoch_id"), || {
            self.epoch_id.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        // === Constraint 1: Compute transition hash ===
        // transition_hash = hash(prev_state, new_state, origin, timestamp)
        // Simplified: (prev_state * new_state) + origin + timestamp
        
        let state_product = AllocatedNum::alloc(cs.namespace(|| "state_product"), || {
            let p = prev_state.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let n = new_state.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(p * n)
        })?;
        
        cs.enforce(
            || "state_product_constraint",
            |lc| lc + prev_state.get_variable(),
            |lc| lc + new_state.get_variable(),
            |lc| lc + state_product.get_variable(),
        );
        
        let transition_hash = AllocatedNum::alloc(cs.namespace(|| "transition_hash"), || {
            let sp = state_product.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let o = origin.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let t = timestamp.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(sp + o + t)
        })?;
        
        cs.enforce(
            || "transition_hash_constraint",
            |lc| lc + state_product.get_variable() + origin.get_variable() + timestamp.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + transition_hash.get_variable(),
        );
        
        // === Constraint 2: Compute new lineage commitment ===
        // new_lineage = hash(prev_lineage, transition_hash)
        // Simplified: prev_lineage * transition_hash + prev_lineage + transition_hash
        
        let lineage_product = AllocatedNum::alloc(cs.namespace(|| "lineage_product"), || {
            let pl = prev_lineage.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let th = transition_hash.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(pl * th)
        })?;
        
        cs.enforce(
            || "lineage_product_constraint",
            |lc| lc + prev_lineage.get_variable(),
            |lc| lc + transition_hash.get_variable(),
            |lc| lc + lineage_product.get_variable(),
        );
        
        let new_lineage = AllocatedNum::alloc(cs.namespace(|| "new_lineage"), || {
            let lp = lineage_product.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let pl = prev_lineage.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let th = transition_hash.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(lp + pl + th)
        })?;
        
        cs.enforce(
            || "new_lineage_constraint",
            |lc| lc + lineage_product.get_variable() + prev_lineage.get_variable() + transition_hash.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + new_lineage.get_variable(),
        );
        
        // === Constraint 3: Update counter commitment ===
        // new_counters = hash(prev_counters, origin, epoch)
        // Simplified: prev_counters + origin + epoch_id
        
        let new_counters = AllocatedNum::alloc(cs.namespace(|| "new_counters"), || {
            let pc = prev_counters.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let o = origin.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let e = epoch_id.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(pc + o + e)
        })?;
        
        cs.enforce(
            || "new_counters_constraint",
            |lc| lc + prev_counters.get_variable() + origin.get_variable() + epoch_id.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + new_counters.get_variable(),
        );
        
        // Return new state: [new_lineage, new_counters]
        Ok(vec![new_lineage, new_counters])
    }
}

#[cfg(not(feature = "real-nova"))]
#[derive(Clone, Debug, Default)]
pub struct LineageStepCircuit;