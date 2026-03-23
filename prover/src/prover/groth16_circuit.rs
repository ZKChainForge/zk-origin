//! Groth16 circuit for compact ZK proofs (<1KB)

#[cfg(feature = "compact-zk")]
use bellman::{Circuit, ConstraintSystem, SynthesisError};

#[cfg(feature = "compact-zk")]
use bls12_381::Scalar as Fr;

/// Maximum number of transitions supported in a single compact proof
#[cfg(feature = "compact-zk")]
pub const MAX_TRANSITIONS: usize = 16;

/// Compact lineage circuit for Groth16 proving
/// Proves: "I know a valid sequence of N transitions from genesis to final state"
#[cfg(feature = "compact-zk")]
#[derive(Clone)]
pub struct CompactLineageCircuit {
    /// Genesis lineage commitment
    pub genesis_lineage: Option<Fr>,
    /// Genesis counter commitment  
    pub genesis_counters: Option<Fr>,
    /// Final lineage commitment
    pub final_lineage: Option<Fr>,
    /// Final counter commitment
    pub final_counters: Option<Fr>,
    /// Number of actual transitions (rest are padding)
    pub num_transitions: usize,
    /// All transition data
    pub transitions: Vec<TransitionWitness>,
}

/// Witness for a single transition
#[cfg(feature = "compact-zk")]
#[derive(Clone, Default)]
pub struct TransitionWitness {
    /// Previous state hash
    pub prev_state: Option<Fr>,
    /// New state hash
    pub new_state: Option<Fr>,
    /// Origin class
    pub origin: Option<Fr>,
    /// Timestamp
    pub timestamp: Option<Fr>,
    /// Epoch ID
    pub epoch_id: Option<Fr>,
    /// Is this a real transition (vs padding)
    pub is_real: bool,
}

#[cfg(feature = "compact-zk")]
impl Default for CompactLineageCircuit {
    fn default() -> Self {
        Self {
            genesis_lineage: None,
            genesis_counters: None,
            final_lineage: None,
            final_counters: None,
            num_transitions: 0,
            transitions: vec![TransitionWitness::default(); MAX_TRANSITIONS],
        }
    }
}

#[cfg(feature = "compact-zk")]
impl CompactLineageCircuit {
    /// Create a new circuit with witness values
    pub fn new(
        genesis_lineage: Fr,
        genesis_counters: Fr,
        final_lineage: Fr,
        final_counters: Fr,
        transitions: Vec<TransitionWitness>,
    ) -> Self {
        let num_transitions = transitions.iter().filter(|t| t.is_real).count();
        
        // Pad to MAX_TRANSITIONS
        let mut padded = transitions;
        while padded.len() < MAX_TRANSITIONS {
            padded.push(TransitionWitness::default());
        }
        
        Self {
            genesis_lineage: Some(genesis_lineage),
            genesis_counters: Some(genesis_counters),
            final_lineage: Some(final_lineage),
            final_counters: Some(final_counters),
            num_transitions,
            transitions: padded,
        }
    }

    /// Create circuit for setup (no witness)
    pub fn empty() -> Self {
        Self::default()
    }
}

#[cfg(feature = "compact-zk")]
impl Circuit<Fr> for CompactLineageCircuit {
    fn synthesize<CS: ConstraintSystem<Fr>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        // Allocate public inputs
        let genesis_lineage_var = cs.alloc_input(
            || "genesis_lineage",
            || self.genesis_lineage.ok_or(SynthesisError::AssignmentMissing),
        )?;

        let genesis_counters_var = cs.alloc_input(
            || "genesis_counters",
            || self.genesis_counters.ok_or(SynthesisError::AssignmentMissing),
        )?;

        let final_lineage_var = cs.alloc_input(
            || "final_lineage",
            || self.final_lineage.ok_or(SynthesisError::AssignmentMissing),
        )?;

        let final_counters_var = cs.alloc_input(
            || "final_counters",
            || self.final_counters.ok_or(SynthesisError::AssignmentMissing),
        )?;

        // Track current state through transitions
        let mut current_lineage_var = genesis_lineage_var;
        let mut current_counters_var = genesis_counters_var;
        
        let mut current_lineage_val = self.genesis_lineage;
        let mut current_counters_val = self.genesis_counters;

        // Process each transition slot
        for (i, transition) in self.transitions.iter().enumerate() {
            let ns = format!("trans_{}", i);

            // Allocate transition witnesses
            let prev_state_var = cs.alloc(
                || format!("{}/prev_state", ns),
                || transition.prev_state.ok_or(SynthesisError::AssignmentMissing),
            )?;

            let new_state_var = cs.alloc(
                || format!("{}/new_state", ns),
                || transition.new_state.ok_or(SynthesisError::AssignmentMissing),
            )?;

            let origin_var = cs.alloc(
                || format!("{}/origin", ns),
                || transition.origin.ok_or(SynthesisError::AssignmentMissing),
            )?;

            let timestamp_var = cs.alloc(
                || format!("{}/timestamp", ns),
                || transition.timestamp.ok_or(SynthesisError::AssignmentMissing),
            )?;

            let epoch_id_var = cs.alloc(
                || format!("{}/epoch_id", ns),
                || transition.epoch_id.ok_or(SynthesisError::AssignmentMissing),
            )?;

            // Compute state_product = prev_state * new_state
            let state_product_val = match (transition.prev_state, transition.new_state) {
                (Some(p), Some(n)) => Some(p * n),
                _ => None,
            };

            let state_product_var = cs.alloc(
                || format!("{}/state_product", ns),
                || state_product_val.ok_or(SynthesisError::AssignmentMissing),
            )?;

            cs.enforce(
                || format!("{}/state_product_eq", ns),
                |lc| lc + prev_state_var,
                |lc| lc + new_state_var,
                |lc| lc + state_product_var,
            );

            // Compute transition_hash = state_product + origin + timestamp
            let transition_hash_val = match (state_product_val, transition.origin, transition.timestamp) {
                (Some(sp), Some(o), Some(t)) => Some(sp + o + t),
                _ => None,
            };

            let transition_hash_var = cs.alloc(
                || format!("{}/transition_hash", ns),
                || transition_hash_val.ok_or(SynthesisError::AssignmentMissing),
            )?;

            cs.enforce(
                || format!("{}/transition_hash_eq", ns),
                |lc| lc + state_product_var + origin_var + timestamp_var,
                |lc| lc + CS::one(),
                |lc| lc + transition_hash_var,
            );

            // Compute lineage_product = current_lineage * transition_hash
            let lineage_product_val = match (current_lineage_val, transition_hash_val) {
                (Some(cl), Some(th)) => Some(cl * th),
                _ => None,
            };

            let lineage_product_var = cs.alloc(
                || format!("{}/lineage_product", ns),
                || lineage_product_val.ok_or(SynthesisError::AssignmentMissing),
            )?;

            cs.enforce(
                || format!("{}/lineage_product_eq", ns),
                |lc| lc + current_lineage_var,
                |lc| lc + transition_hash_var,
                |lc| lc + lineage_product_var,
            );

            // Compute new_lineage = lineage_product + current_lineage + transition_hash
            let new_lineage_val = match (lineage_product_val, current_lineage_val, transition_hash_val) {
                (Some(lp), Some(cl), Some(th)) => Some(lp + cl + th),
                _ => None,
            };

            let new_lineage_var = cs.alloc(
                || format!("{}/new_lineage", ns),
                || new_lineage_val.ok_or(SynthesisError::AssignmentMissing),
            )?;

            cs.enforce(
                || format!("{}/new_lineage_eq", ns),
                |lc| lc + lineage_product_var + current_lineage_var + transition_hash_var,
                |lc| lc + CS::one(),
                |lc| lc + new_lineage_var,
            );

            // Compute new_counters = current_counters + origin + epoch_id
            let new_counters_val = match (current_counters_val, transition.origin, transition.epoch_id) {
                (Some(cc), Some(o), Some(e)) => Some(cc + o + e),
                _ => None,
            };

            let new_counters_var = cs.alloc(
                || format!("{}/new_counters", ns),
                || new_counters_val.ok_or(SynthesisError::AssignmentMissing),
            )?;

            cs.enforce(
                || format!("{}/new_counters_eq", ns),
                |lc| lc + current_counters_var + origin_var + epoch_id_var,
                |lc| lc + CS::one(),
                |lc| lc + new_counters_var,
            );

            // Update current state for next iteration
            current_lineage_var = new_lineage_var;
            current_counters_var = new_counters_var;
            current_lineage_val = new_lineage_val;
            current_counters_val = new_counters_val;
        }

        // Constrain final state equals public inputs
        cs.enforce(
            || "final_lineage_match",
            |lc| lc + current_lineage_var - final_lineage_var,
            |lc| lc + CS::one(),
            |lc| lc,
        );

        cs.enforce(
            || "final_counters_match",
            |lc| lc + current_counters_var - final_counters_var,
            |lc| lc + CS::one(),
            |lc| lc,
        );

        Ok(())
    }
}

// Non-compact-zk stubs
#[cfg(not(feature = "compact-zk"))]
pub const MAX_TRANSITIONS: usize = 16;

#[cfg(not(feature = "compact-zk"))]
#[derive(Clone, Debug, Default)]
pub struct CompactLineageCircuit;

#[cfg(not(feature = "compact-zk"))]
#[derive(Clone, Debug, Default)]
pub struct TransitionWitness;