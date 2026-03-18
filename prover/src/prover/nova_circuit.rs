//! Nova circuit for lineage step verification

#[cfg(feature = "real-nova")]
use bellpepper_core::{num::AllocatedNum, ConstraintSystem, SynthesisError};

#[cfg(feature = "real-nova")]
use ff::{Field, PrimeField};

#[cfg(feature = "real-nova")]
use nova_snark::traits::circuit::StepCircuit;

/// The main circuit that verifies one lineage step.
#[cfg(feature = "real-nova")]
#[derive(Clone, Debug)]
pub struct LineageStepCircuit<F: PrimeField> {
    prev_state_hash: F,
    new_state_hash: F,
    origin_class: F,
    timestamp: F,
    epoch_id: F,
}

#[cfg(feature = "real-nova")]
impl<F: PrimeField> Default for LineageStepCircuit<F> {
    fn default() -> Self {
        Self {
            prev_state_hash: F::ZERO,
            new_state_hash: F::ZERO,
            origin_class: F::ZERO,
            timestamp: F::ZERO,
            epoch_id: F::ZERO,
        }
    }
}

#[cfg(feature = "real-nova")]
impl<F: PrimeField> LineageStepCircuit<F> {
    /// Create a new circuit with witness values
    pub fn new(
        prev_state_hash: F,
        new_state_hash: F,
        origin_class: F,
        timestamp: F,
        epoch_id: F,
    ) -> Self {
        Self {
            prev_state_hash,
            new_state_hash,
            origin_class,
            timestamp,
            epoch_id,
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
        if z.len() != 2 {
            return Err(SynthesisError::Unsatisfiable);
        }

        // z[0] = previous lineage commitment (from Nova IVC)
        // z[1] = previous counter commitment (from Nova IVC)
        let prev_lineage = &z[0];
        let prev_counters = &z[1];

        // === Allocate witness variables (using stored values, not Option) ===
        let prev_state =
            AllocatedNum::alloc(cs.namespace(|| "prev_state"), || Ok(self.prev_state_hash))?;

        let new_state =
            AllocatedNum::alloc(cs.namespace(|| "new_state"), || Ok(self.new_state_hash))?;

        let origin =
            AllocatedNum::alloc(cs.namespace(|| "origin_class"), || Ok(self.origin_class))?;

        let timestamp = AllocatedNum::alloc(cs.namespace(|| "timestamp"), || Ok(self.timestamp))?;

        let epoch_id = AllocatedNum::alloc(cs.namespace(|| "epoch_id"), || Ok(self.epoch_id))?;

        // === Constraint 1: Compute state_product = prev_state * new_state ===
        let state_product = AllocatedNum::alloc(cs.namespace(|| "state_product"), || {
            Ok(self.prev_state_hash * self.new_state_hash)
        })?;

        cs.enforce(
            || "state_product = prev_state * new_state",
            |lc| lc + prev_state.get_variable(),
            |lc| lc + new_state.get_variable(),
            |lc| lc + state_product.get_variable(),
        );

        // === Constraint 2: Compute transition_hash = state_product + origin + timestamp ===
        let transition_hash_val =
            self.prev_state_hash * self.new_state_hash + self.origin_class + self.timestamp;

        let transition_hash = AllocatedNum::alloc(cs.namespace(|| "transition_hash"), || {
            Ok(transition_hash_val)
        })?;

        cs.enforce(
            || "transition_hash = state_product + origin + timestamp",
            |lc| {
                lc + state_product.get_variable() + origin.get_variable() + timestamp.get_variable()
            },
            |lc| lc + CS::one(),
            |lc| lc + transition_hash.get_variable(),
        );

        // === Constraint 3: Compute lineage_product = prev_lineage * transition_hash ===
        let lineage_product = AllocatedNum::alloc(cs.namespace(|| "lineage_product"), || {
            let pl = prev_lineage
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?;
            Ok(pl * transition_hash_val)
        })?;

        cs.enforce(
            || "lineage_product = prev_lineage * transition_hash",
            |lc| lc + prev_lineage.get_variable(),
            |lc| lc + transition_hash.get_variable(),
            |lc| lc + lineage_product.get_variable(),
        );

        // === Constraint 4: Compute new_lineage = lineage_product + prev_lineage + transition_hash ===
        let new_lineage = AllocatedNum::alloc(cs.namespace(|| "new_lineage"), || {
            let lp = lineage_product
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?;
            let pl = prev_lineage
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?;
            Ok(lp + pl + transition_hash_val)
        })?;

        cs.enforce(
            || "new_lineage = lineage_product + prev_lineage + transition_hash",
            |lc| {
                lc + lineage_product.get_variable()
                    + prev_lineage.get_variable()
                    + transition_hash.get_variable()
            },
            |lc| lc + CS::one(),
            |lc| lc + new_lineage.get_variable(),
        );

        // === Constraint 5: Compute new_counters = prev_counters + origin + epoch_id ===
        let new_counters = AllocatedNum::alloc(cs.namespace(|| "new_counters"), || {
            let pc = prev_counters
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?;
            Ok(pc + self.origin_class + self.epoch_id)
        })?;

        cs.enforce(
            || "new_counters = prev_counters + origin + epoch_id",
            |lc| {
                lc + prev_counters.get_variable() + origin.get_variable() + epoch_id.get_variable()
            },
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
