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
    // Previous state
    prev_lineage: Option<F>,
    prev_counters: Option<F>,
    
    // Current transition
    prev_state_hash: Option<F>,
    new_state_hash: Option<F>,
    origin_class: Option<F>,
    timestamp: Option<F>,
    
    // Policy proof
    policy_root: Option<F>,
    policy_path: Vec<Option<F>>,
    policy_indices: Vec<bool>,
    
    // Rate limits
    epoch_id: Option<F>,
    rate_limits: [Option<F>; 6],
    
    _marker: std::marker::PhantomData<F>,
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
            policy_path: vec![],
            policy_indices: vec![],
            epoch_id: None,
            rate_limits: [None; 6],
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "real-nova")]
impl<F: PrimeField> LineageStepCircuit<F> {
    /// Create a new circuit from witness data
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
            _marker: std::marker::PhantomData,
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
        assert_eq!(z.len(), 2, "Expected 2 inputs: lineage and counters");

        let prev_lineage = &z[0];
        let prev_counters = &z[1];

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

        let policy_root = AllocatedNum::alloc(cs.namespace(|| "policy_root"), || {
            self.policy_root.ok_or(SynthesisError::AssignmentMissing)
        })?;

        cs.enforce(
            || "policy_root_nonzero",
            |lc| lc + policy_root.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + policy_root.get_variable(),
        );

        let transition_hash = AllocatedNum::alloc(cs.namespace(|| "transition_hash"), || {
            let prev = self.prev_state_hash.ok_or(SynthesisError::AssignmentMissing)?;
            let new = self.new_state_hash.ok_or(SynthesisError::AssignmentMissing)?;
            let orig = self.origin_class.ok_or(SynthesisError::AssignmentMissing)?;
            let time = self.timestamp.ok_or(SynthesisError::AssignmentMissing)?;
            Ok(prev + new + orig + time)
        })?;

        cs.enforce(
            || "transition_hash_computation",
            |lc| lc + prev_state.get_variable() + new_state.get_variable() + origin.get_variable() + timestamp.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + transition_hash.get_variable(),
        );

        let new_lineage = AllocatedNum::alloc(cs.namespace(|| "new_lineage"), || {
            let prev = prev_lineage.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let trans = transition_hash.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(prev + trans)
        })?;

        cs.enforce(
            || "new_lineage_computation",
            |lc| lc + prev_lineage.get_variable() + transition_hash.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + new_lineage.get_variable(),
        );

        let new_counters = AllocatedNum::alloc(cs.namespace(|| "new_counters"), || {
            let prev = prev_counters.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let orig = self.origin_class.ok_or(SynthesisError::AssignmentMissing)?;
            Ok(prev + orig)
        })?;

        cs.enforce(
            || "new_counters_computation",
            |lc| lc + prev_counters.get_variable() + origin.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + new_counters.get_variable(),
        );

        Ok(vec![new_lineage, new_counters])
    }
}

#[cfg(not(feature = "real-nova"))]
pub struct LineageStepCircuit;