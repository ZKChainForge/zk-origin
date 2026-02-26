//! Nova-compatible step circuit for lineage verification
//!
//! This circuit verifies a single step in the lineage chain:
//! 1. The origin transition is allowed by policy
//! 2. Rate limits are not exceeded
//! 3. The lineage commitment is correctly updated

use bellpepper_core::{
    boolean::Boolean,
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use ff::PrimeField;
use std::marker::PhantomData;

use crate::circuit::gadgets::{SelectorGadget, RangeCheckGadget};
use crate::circuit::constraints::ConstraintHelpers;
use crate::circuit::poseidon_circuit::PoseidonCircuit;

/// Number of origin classes
pub const NUM_ORIGIN_CLASSES: usize = 6;

/// Depth of policy Merkle tree
pub const POLICY_TREE_DEPTH: usize = 4;

/// The step circuit for ZK-ORIGIN lineage verification
#[derive(Clone)]
pub struct LineageStepCircuit<F: PrimeField> {
    // Witness data (None for circuit description, Some for actual proving)
    
    /// Previous state hash
    pub prev_state_hash: Option<F>,
    
    /// New state hash
    pub new_state_hash: Option<F>,
    
    /// Previous origin class (0-5)
    pub prev_origin: Option<u64>,
    
    /// New origin class (0-5)
    pub new_origin: Option<u64>,
    
    /// Timestamp
    pub timestamp: Option<u64>,
    
    /// Previous lineage depth
    pub prev_depth: Option<u64>,
    
    /// Policy Merkle root
    pub policy_root: Option<F>,
    
    /// Policy Merkle proof path
    pub policy_proof: Option<Vec<F>>,
    
    /// Policy proof indices (is_right flags)
    pub policy_indices: Option<Vec<bool>>,
    
    /// Epoch ID
    pub epoch_id: Option<u64>,
    
    /// Previous counters
    pub prev_counters: Option<[u32; NUM_ORIGIN_CLASSES]>,
    
    /// Rate limits
    pub rate_limits: Option<[u32; NUM_ORIGIN_CLASSES]>,
    
    /// Poseidon hasher for circuit
    poseidon: PoseidonCircuit<F>,
    
    _phantom: PhantomData<F>,
}

impl<F: PrimeField> Default for LineageStepCircuit<F> {
    fn default() -> Self {
        Self {
            prev_state_hash: None,
            new_state_hash: None,
            prev_origin: None,
            new_origin: None,
            timestamp: None,
            prev_depth: None,
            policy_root: None,
            policy_proof: None,
            policy_indices: None,
            epoch_id: None,
            prev_counters: None,
            rate_limits: None,
            poseidon: PoseidonCircuit::new(),
            _phantom: PhantomData,
        }
    }
}

impl<F: PrimeField> LineageStepCircuit<F> {
    /// Create a new step circuit with witness data
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        prev_state_hash: F,
        new_state_hash: F,
        prev_origin: u64,
        new_origin: u64,
        timestamp: u64,
        prev_depth: u64,
        policy_root: F,
        policy_proof: Vec<F>,
        policy_indices: Vec<bool>,
        epoch_id: u64,
        prev_counters: [u32; NUM_ORIGIN_CLASSES],
        rate_limits: [u32; NUM_ORIGIN_CLASSES],
    ) -> Self {
        Self {
            prev_state_hash: Some(prev_state_hash),
            new_state_hash: Some(new_state_hash),
            prev_origin: Some(prev_origin),
            new_origin: Some(new_origin),
            timestamp: Some(timestamp),
            prev_depth: Some(prev_depth),
            policy_root: Some(policy_root),
            policy_proof: Some(policy_proof),
            policy_indices: Some(policy_indices),
            epoch_id: Some(epoch_id),
            prev_counters: Some(prev_counters),
            rate_limits: Some(rate_limits),
            poseidon: PoseidonCircuit::new(),
            _phantom: PhantomData,
        }
    }

    /// Synthesize the step circuit
    /// 
    /// z[0] = previous lineage commitment
    /// z[1] = previous counter commitment
    /// 
    /// Returns:
    /// z'[0] = new lineage commitment
    /// z'[1] = new counter commitment
    pub fn synthesize_step<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        z: &[AllocatedNum<F>],
    ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
        assert_eq!(z.len(), 2, "Expected 2 state elements");

        let prev_lineage = &z[0];
        let prev_counter_commit = &z[1];

        // === ALLOCATE WITNESS VARIABLES ===
        
        let prev_state = AllocatedNum::alloc(cs.namespace(|| "prev_state"), || {
            self.prev_state_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let new_state = AllocatedNum::alloc(cs.namespace(|| "new_state"), || {
            self.new_state_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let prev_origin = AllocatedNum::alloc(cs.namespace(|| "prev_origin"), || {
            self.prev_origin.map(F::from).ok_or(SynthesisError::AssignmentMissing)
        })?;

        let new_origin = AllocatedNum::alloc(cs.namespace(|| "new_origin"), || {
            self.new_origin.map(F::from).ok_or(SynthesisError::AssignmentMissing)
        })?;

        let timestamp = AllocatedNum::alloc(cs.namespace(|| "timestamp"), || {
            self.timestamp.map(F::from).ok_or(SynthesisError::AssignmentMissing)
        })?;

        let prev_depth = AllocatedNum::alloc(cs.namespace(|| "prev_depth"), || {
            self.prev_depth.map(F::from).ok_or(SynthesisError::AssignmentMissing)
        })?;

        let policy_root = AllocatedNum::alloc(cs.namespace(|| "policy_root"), || {
            self.policy_root.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let epoch_id = AllocatedNum::alloc(cs.namespace(|| "epoch_id"), || {
            self.epoch_id.map(F::from).ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Allocate policy proof path
        let policy_proof: Vec<AllocatedNum<F>> = self.policy_proof
            .as_ref()
            .map(|proof| {
                proof.iter().enumerate().map(|(i, &p)| {
                    AllocatedNum::alloc(
                        cs.namespace(|| format!("policy_proof_{}", i)),
                        || Ok(p)
                    )
                }).collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_else(|| {
                (0..POLICY_TREE_DEPTH).map(|i| {
                    AllocatedNum::alloc(
                        cs.namespace(|| format!("policy_proof_{}", i)),
                        || Err(SynthesisError::AssignmentMissing)
                    )
                }).collect()
            })?;

        // Allocate policy indices as Boolean
        let policy_indices: Vec<Boolean> = self.policy_indices
            .as_ref()
            .map(|indices| {
                indices.iter().map(|&idx| {
                    Ok(Boolean::constant(idx))
                }).collect::<Result<Vec<_>, SynthesisError>>()
            })
            .unwrap_or_else(|| {
                (0..POLICY_TREE_DEPTH).map(|_| {
                    Ok(Boolean::constant(false))
                }).collect()
            })?;

        // Allocate counters
        let prev_counters: Vec<AllocatedNum<F>> = self.prev_counters
            .as_ref()
            .map(|counters| {
                counters.iter().enumerate().map(|(i, &c)| {
                    AllocatedNum::alloc(
                        cs.namespace(|| format!("prev_counter_{}", i)),
                        || Ok(F::from(c as u64))
                    )
                }).collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_else(|| {
                (0..NUM_ORIGIN_CLASSES).map(|i| {
                    AllocatedNum::alloc(
                        cs.namespace(|| format!("prev_counter_{}", i)),
                        || Err(SynthesisError::AssignmentMissing)
                    )
                }).collect()
            })?;

        let rate_limits: Vec<AllocatedNum<F>> = self.rate_limits
            .as_ref()
            .map(|limits| {
                limits.iter().enumerate().map(|(i, &l)| {
                    AllocatedNum::alloc(
                        cs.namespace(|| format!("rate_limit_{}", i)),
                        || Ok(F::from(l as u64))
                    )
                }).collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_else(|| {
                (0..NUM_ORIGIN_CLASSES).map(|i| {
                    AllocatedNum::alloc(
                        cs.namespace(|| format!("rate_limit_{}", i)),
                        || Err(SynthesisError::AssignmentMissing)
                    )
                }).collect()
            })?;

        // === SECTION 1: ORIGIN VALIDATION ===
        RangeCheckGadget::less_than(
            cs.namespace(|| "prev_origin_range"),
            &prev_origin,
            NUM_ORIGIN_CLASSES as u64,
            4,
        )?;

        RangeCheckGadget::less_than(
            cs.namespace(|| "new_origin_range"),
            &new_origin,
            NUM_ORIGIN_CLASSES as u64,
            4,
        )?;

        // === SECTION 2: POLICY VERIFICATION ===
        // Compute policy leaf = Poseidon(prev_origin, new_origin)
        let policy_leaf = self.poseidon.hash2(
            &mut cs.namespace(|| "policy_leaf"),
            &prev_origin,
            &new_origin,
        )?;

        // Verify Merkle proof using our inline implementation
        let _policy_valid = self.verify_merkle_proof(
            &mut cs.namespace(|| "policy_verify"),
            &policy_leaf,
            &policy_root,
            &policy_proof,
            &policy_indices,
        )?;

        // === SECTION 3: RATE LIMIT CHECK ===
        let selected_counter = SelectorGadget::select(
            &mut cs.namespace(|| "select_counter"),
            &prev_counters,
            &new_origin,
        )?;

        let selected_limit = SelectorGadget::select(
            &mut cs.namespace(|| "select_limit"),
            &rate_limits,
            &new_origin,
        )?;

        // Verify selected_counter < selected_limit
        let _rate_ok = self.verify_less_than(
            &mut cs.namespace(|| "rate_limit_check"),
            &selected_counter,
            &selected_limit,
        )?;

        // === SECTION 4: COMPUTE TRANSITION HASH ===
        let transition_hash = self.poseidon.hash5(
            &mut cs.namespace(|| "transition_hash"),
            &prev_state,
            &new_state,
            &new_origin,
            &timestamp,
            &epoch_id,
        )?;

        // === SECTION 5: UPDATE LINEAGE COMMITMENT ===
        let new_depth = ConstraintHelpers::add_constant(
            &mut cs.namespace(|| "new_depth"),
            &prev_depth,
            F::ONE,
        )?;

        let new_lineage = self.poseidon.hash3(
            &mut cs.namespace(|| "new_lineage"),
            prev_lineage,
            &transition_hash,
            &new_depth,
        )?;

        // === SECTION 6: UPDATE COUNTER COMMITMENT ===
        let new_counter_commit = self.compute_new_counter_commitment(
            &mut cs.namespace(|| "new_counter_commit"),
            &epoch_id,
            &prev_counters,
            &new_origin,
        )?;

        // Verify previous counter commitment matches
        let computed_prev_commit = self.compute_counter_commitment_from_parts(
            &mut cs.namespace(|| "verify_prev_counters"),
            &epoch_id,
            &prev_counters,
        )?;

        cs.enforce(
            || "prev_counter_commit_match",
            |lc| lc + computed_prev_commit.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + prev_counter_commit.get_variable(),
        );

        Ok(vec![new_lineage, new_counter_commit])
    }

    /// Verify Merkle proof inline (to avoid import issues)
    fn verify_merkle_proof<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        leaf: &AllocatedNum<F>,
        expected_root: &AllocatedNum<F>,
        path: &[AllocatedNum<F>],
        indices: &[Boolean],
    ) -> Result<Boolean, SynthesisError> {
        let mut current = leaf.clone();

        for (i, (sibling, is_right)) in path.iter().zip(indices.iter()).enumerate() {
            // Select order based on is_right
            let left = SelectorGadget::if_then_else(
                &mut cs.namespace(|| format!("select_left_{}", i)),
                is_right,
                sibling,
                &current,
            )?;
            
            let right = SelectorGadget::if_then_else(
                &mut cs.namespace(|| format!("select_right_{}", i)),
                is_right,
                &current,
                sibling,
            )?;
            
            // Hash with Poseidon
            current = self.poseidon.hash2(
                &mut cs.namespace(|| format!("merkle_hash_{}", i)),
                &left,
                &right,
            )?;
        }

        // Check equality
        let diff = AllocatedNum::alloc(cs.namespace(|| "root_diff"), || {
            let c = current.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let e = expected_root.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(c - e)
        })?;

        cs.enforce(
            || "root_diff_constraint",
            |lc| lc + current.get_variable() - expected_root.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + diff.get_variable(),
        );

        // For valid proof, diff should be 0
        // We just enforce it rather than returning a boolean
        cs.enforce(
            || "root_must_match",
            |lc| lc + diff.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc,
        );

        Ok(Boolean::constant(true))
    }

    /// Verify a < b using bit decomposition
    fn verify_less_than<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        a: &AllocatedNum<F>,
        b: &AllocatedNum<F>,
    ) -> Result<Boolean, SynthesisError> {
        // Compute diff = b - a - 1
        let diff = AllocatedNum::alloc(cs.namespace(|| "diff"), || {
            let a_val = a.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let b_val = b.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(b_val - a_val - F::ONE)
        })?;

        // Constrain: diff = b - a - 1
        cs.enforce(
            || "diff_constraint",
            |lc| lc + CS::one(),
            |lc| lc + diff.get_variable(),
            |lc| lc + b.get_variable() - a.get_variable() - CS::one(),
        );

        // Verify diff is positive by decomposing into 32 bits
        self.verify_bits(cs, &diff, 32)?;

        Ok(Boolean::constant(true))
    }

    /// Verify a value fits in n bits
    fn verify_bits<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        value: &AllocatedNum<F>,
        num_bits: usize,
    ) -> Result<(), SynthesisError> {
        use bellpepper_core::boolean::AllocatedBit;
        use bellpepper_core::LinearCombination;

        let value_bits: Option<Vec<bool>> = value.get_value().map(|v| {
            let repr = v.to_repr();
            let bytes = repr.as_ref();
            (0..num_bits).map(|i| {
                let byte_idx = i / 8;
                let bit_idx = i % 8;
                if byte_idx < bytes.len() {
                    (bytes[byte_idx] >> bit_idx) & 1 == 1
                } else {
                    false
                }
            }).collect()
        });

        let mut bit_lc = LinearCombination::<F>::zero();
        let mut coeff = F::ONE;

        for i in 0..num_bits {
            let bit = AllocatedBit::alloc(
                cs.namespace(|| format!("bit_{}", i)),
                value_bits.as_ref().map(|bits| bits[i]),
            )?;

            bit_lc = bit_lc + (coeff, bit.get_variable());
            coeff = coeff.double();
        }

        cs.enforce(
            || "bits_sum",
            |lc| lc + value.get_variable(),
            |lc| lc + CS::one(),
            |_| bit_lc,
        );

        Ok(())
    }

    /// Compute counter commitment from individual counters
    fn compute_counter_commitment_from_parts<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        epoch: &AllocatedNum<F>,
        counters: &[AllocatedNum<F>],
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        let h1 = self.poseidon.hash5(
            &mut cs.namespace(|| "counter_hash_1"),
            epoch,
            &counters[0],
            &counters[1],
            &counters[2],
            &counters[3],
        )?;

        self.poseidon.hash3(
            &mut cs.namespace(|| "counter_hash_2"),
            &h1,
            &counters[4],
            &counters[5],
        )
    }

    /// Compute new counter commitment after incrementing
    fn compute_new_counter_commitment<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        epoch: &AllocatedNum<F>,
        prev_counters: &[AllocatedNum<F>],
        new_origin: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        let mut new_counters = Vec::with_capacity(NUM_ORIGIN_CLASSES);
        
        for i in 0..NUM_ORIGIN_CLASSES {
            let is_this_origin = AllocatedNum::alloc(
                cs.namespace(|| format!("is_origin_{}", i)),
                || {
                    let origin_val = new_origin.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                    Ok(if origin_val == F::from(i as u64) { F::ONE } else { F::ZERO })
                },
            )?;
            
            ConstraintHelpers::enforce_boolean(
                &mut cs.namespace(|| format!("bool_check_{}", i)),
                &is_this_origin,
            );
            
            cs.enforce(
                || format!("indicator_correct_{}", i),
                |lc| lc + is_this_origin.get_variable(),
                |lc| lc + new_origin.get_variable() - (F::from(i as u64), CS::one()),
                |lc| lc,
            );
            
            let new_counter = ConstraintHelpers::add(
                &mut cs.namespace(|| format!("inc_counter_{}", i)),
                &prev_counters[i],
                &is_this_origin,
            )?;
            
            new_counters.push(new_counter);
        }
        
        self.compute_counter_commitment_from_parts(
            &mut cs.namespace(|| "new_counter_commit_hash"),
            epoch,
            &new_counters,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    fn create_test_circuit() -> LineageStepCircuit<Fp> {
        LineageStepCircuit::new(
            Fp::from(1u64),
            Fp::from(2u64),
            0,
            1,
            1000,
            0,
            Fp::from(100u64),
            vec![Fp::from(1u64); POLICY_TREE_DEPTH],
            vec![false; POLICY_TREE_DEPTH],
            0,
            [0; NUM_ORIGIN_CLASSES],
            [1, u32::MAX, 10, 100, 5, 1000],
        )
    }

    #[test]
    fn test_step_circuit_synthesis() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let circuit = create_test_circuit();
        
        let z0 = AllocatedNum::alloc(cs.namespace(|| "z0"), || Ok(Fp::from(0u64))).unwrap();
        let z1 = AllocatedNum::alloc(cs.namespace(|| "z1"), || Ok(Fp::from(0u64))).unwrap();
        
        let z = vec![z0, z1];
        
        let result = circuit.synthesize_step(&mut cs, &z);
        
        assert!(result.is_ok());
        let z_prime = result.unwrap();
        assert_eq!(z_prime.len(), 2);
        
        let num_constraints = cs.num_constraints();
        println!("Total constraints: {}", num_constraints);
    }

    #[test]
    fn test_step_circuit_default() {
        let circuit: LineageStepCircuit<Fp> = LineageStepCircuit::default();
        assert!(circuit.prev_state_hash.is_none());
    }
}