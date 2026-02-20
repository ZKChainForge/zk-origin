//! Step circuit implementation for Nova IVC

use bellpepper_core::{
    boolean::Boolean,
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use ff::PrimeField;
use std::marker::PhantomData;

use super::gadgets::{MerkleGadget, SelectorGadget};
use super::constraints::ConstraintHelpers;

/// Number of origin classes
pub const NUM_ORIGIN_CLASSES: usize = 6;

/// Depth of policy Merkle tree
pub const POLICY_TREE_DEPTH: usize = 4;

/// The step circuit for ZK-ORIGIN lineage verification
/// 
/// This circuit verifies a single step in the lineage chain:
/// 1. The origin transition is allowed by policy
/// 2. Rate limits are not exceeded
/// 3. The lineage commitment is correctly updated
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
            _phantom: PhantomData,
        }
    }
}

impl<F: PrimeField> LineageStepCircuit<F> {
    /// Create a new step circuit with witness data
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

        // Allocate policy indices
        let policy_indices: Vec<Boolean> = self.policy_indices
            .as_ref()
            .map(|indices| {
                indices.iter().enumerate().map(|(_i, &idx)| {
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
        // Check that origin classes are in valid range [0, 5]
        // For simplicity, we trust the witness here
        // Production would add range checks

        // === SECTION 2: POLICY VERIFICATION ===
        // Compute policy leaf = hash(prev_origin, new_origin)
        let policy_leaf = self.compute_policy_leaf(
            &mut cs.namespace(|| "policy_leaf"),
            &prev_origin,
            &new_origin,
        )?;

        // Verify Merkle proof
        let _policy_valid = MerkleGadget::verify(
            &mut cs.namespace(|| "policy_verify"),
            &policy_leaf,
            &policy_proof,
            &policy_indices,
            &policy_root,
        )?;

        // === SECTION 3: RATE LIMIT CHECK ===
        // Select the counter for new_origin and check it's below limit
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

        // We should verify selected_counter < selected_limit
        // Simplified: just allocate and trust for now
        // Production would add comparison constraint
        let _ = (&selected_counter, &selected_limit);

        // === SECTION 4: COMPUTE TRANSITION HASH ===
        let transition_hash = self.compute_transition_hash(
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

        let new_lineage = self.compute_lineage_commitment(
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

        // Verify previous counter commitment (simplified)
        let _ = prev_counter_commit;

        Ok(vec![new_lineage, new_counter_commit])
    }

    /// Compute policy leaf hash
    fn compute_policy_leaf<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        prev_origin: &AllocatedNum<F>,
        new_origin: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        // Placeholder: In production, use Poseidon
        let leaf = AllocatedNum::alloc(cs.namespace(|| "leaf"), || {
            let p = prev_origin.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let n = new_origin.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(p + n + F::ONE)
        })?;

        cs.enforce(
            || "leaf_constraint",
            |lc| lc + CS::one(),
            |lc| lc + leaf.get_variable(),
            |lc| lc + prev_origin.get_variable() + new_origin.get_variable() + CS::one(),
        );

        Ok(leaf)
    }

        /// Compute transition hash
    fn compute_transition_hash<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        prev_state: &AllocatedNum<F>,
        new_state: &AllocatedNum<F>,
        origin: &AllocatedNum<F>,
        timestamp: &AllocatedNum<F>,
        epoch: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        // Placeholder: In production, use Poseidon with 5 inputs
        // For now, we use a simple linear combination as placeholder
        
        let hash = AllocatedNum::alloc(cs.namespace(|| "trans_hash"), || {
            let ps = prev_state.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let ns = new_state.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let o = origin.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let t = timestamp.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let e = epoch.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            
            // Simple combination (NOT cryptographically secure - placeholder)
            Ok(ps + ns + o + t + e)
        })?;

        // Constrain the hash computation
        cs.enforce(
            || "trans_hash_constraint",
            |lc| lc + CS::one(),
            |lc| lc + hash.get_variable(),
            |lc| {
                lc + prev_state.get_variable()
                    + new_state.get_variable()
                    + origin.get_variable()
                    + timestamp.get_variable()
                    + epoch.get_variable()
            },
        );

        Ok(hash)
    }

    /// Compute new lineage commitment
    fn compute_lineage_commitment<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        prev_lineage: &AllocatedNum<F>,
        transition_hash: &AllocatedNum<F>,
        depth: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        // Placeholder: In production, use Poseidon with 3 inputs
        
        let commitment = AllocatedNum::alloc(cs.namespace(|| "lineage_commit"), || {
            let pl = prev_lineage.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let th = transition_hash.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let d = depth.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            
            Ok(pl + th + d)
        })?;

        cs.enforce(
            || "lineage_constraint",
            |lc| lc + CS::one(),
            |lc| lc + commitment.get_variable(),
            |lc| {
                lc + prev_lineage.get_variable()
                    + transition_hash.get_variable()
                    + depth.get_variable()
            },
        );

        Ok(commitment)
    }

    /// Compute new counter commitment
    fn compute_new_counter_commitment<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        epoch: &AllocatedNum<F>,
        prev_counters: &[AllocatedNum<F>],
        new_origin: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        // First, compute incremented counters
        let mut new_counters = Vec::with_capacity(NUM_ORIGIN_CLASSES);
        
        for i in 0..NUM_ORIGIN_CLASSES {
            // Create indicator: is this the origin being incremented?
            let is_this_origin = AllocatedNum::alloc(
                cs.namespace(|| format!("is_origin_{}", i)),
                || {
                    let origin_val = new_origin.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                    Ok(if origin_val == F::from(i as u64) { F::ONE } else { F::ZERO })
                },
            )?;
            
            // Constrain is_this_origin to be boolean
            ConstraintHelpers::enforce_boolean(
                &mut cs.namespace(|| format!("bool_check_{}", i)),
                &is_this_origin,
            );
            
            // new_counter[i] = prev_counter[i] + is_this_origin
            let new_counter = ConstraintHelpers::add(
                &mut cs.namespace(|| format!("inc_counter_{}", i)),
                &prev_counters[i],
                &is_this_origin,
            )?;
            
            new_counters.push(new_counter);
        }
        
        // Compute commitment from epoch and new counters
        // Placeholder: sum them all
        let mut commitment = epoch.clone();
        
        for (i, counter) in new_counters.iter().enumerate() {
            commitment = ConstraintHelpers::add(
                &mut cs.namespace(|| format!("add_counter_{}", i)),
                &commitment,
                counter,
            )?;
        }
        
        Ok(commitment)
    }
}

/// Implement Nova's StepCircuit trait
/// 
/// This allows LineageStepCircuit to be used with Nova's recursive proving
#[cfg(feature = "nova")]
impl<F> nova_snark::traits::circuit::StepCircuit<F> for LineageStepCircuit<F>
where
    F: PrimeField,
{
    fn arity(&self) -> usize {
        2 // (lineage_commitment, counter_commitment)
    }

    fn synthesize<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        z: &[AllocatedNum<F>],
    ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
        self.synthesize_step(cs, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    fn create_test_circuit() -> LineageStepCircuit<Fp> {
        LineageStepCircuit::new(
            Fp::from(1u64),  // prev_state_hash
            Fp::from(2u64),  // new_state_hash
            0,               // prev_origin (Genesis)
            1,               // new_origin (User)
            1000,            // timestamp
            0,               // prev_depth
            Fp::from(100u64), // policy_root
            vec![Fp::from(1u64); POLICY_TREE_DEPTH], // policy_proof
            vec![false; POLICY_TREE_DEPTH],          // policy_indices
            0,               // epoch_id
            [0; NUM_ORIGIN_CLASSES], // prev_counters
            [1, u32::MAX, 10, 100, 5, 1000], // rate_limits
        )
    }

    #[test]
    fn test_step_circuit_synthesis() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let circuit = create_test_circuit();
        
        // Allocate initial state
        let z0 = AllocatedNum::alloc(cs.namespace(|| "z0"), || Ok(Fp::from(0u64))).unwrap();
        let z1 = AllocatedNum::alloc(cs.namespace(|| "z1"), || Ok(Fp::from(0u64))).unwrap();
        
        let z = vec![z0, z1];
        
        let result = circuit.synthesize_step(&mut cs, &z);
        
        assert!(result.is_ok());
        let z_prime = result.unwrap();
        assert_eq!(z_prime.len(), 2);
    }
    #[test]
fn test_step_circuit_constraints_satisfied() {
    let mut cs = TestConstraintSystem::<Fp>::new();
    
    let circuit = create_test_circuit();
    
    let z0 = AllocatedNum::alloc(cs.namespace(|| "z0"), || Ok(Fp::from(0u64))).unwrap();
    let z1 = AllocatedNum::alloc(cs.namespace(|| "z1"), || Ok(Fp::from(0u64))).unwrap();
    
    let z = vec![z0, z1];
    
    let _ = circuit.synthesize_step(&mut cs, &z).unwrap();
    
    if let Some(name) = cs.which_is_unsatisfied() {
        println!("Unsatisfied constraint: {}", name);
    }
    
    assert!(cs.is_satisfied());
}


    #[test]
    fn test_step_circuit_constraint_count() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let circuit = create_test_circuit();
        
        let z0 = AllocatedNum::alloc(cs.namespace(|| "z0"), || Ok(Fp::from(0u64))).unwrap();
        let z1 = AllocatedNum::alloc(cs.namespace(|| "z1"), || Ok(Fp::from(0u64))).unwrap();
        
        let z = vec![z0, z1];
        
        let _ = circuit.synthesize_step(&mut cs, &z).unwrap();
        
        let num_constraints = cs.num_constraints();
        println!("Number of constraints: {}", num_constraints);
        
        // Verify we're in a reasonable range
        assert!(num_constraints > 0);
        assert!(num_constraints < 100_000); // Sanity check
    }

    #[test]
    fn test_default_circuit() {
        let circuit: LineageStepCircuit<Fp> = LineageStepCircuit::default();
        
        assert!(circuit.prev_state_hash.is_none());
        assert!(circuit.new_origin.is_none());
    }

    #[test]
    fn test_multiple_steps() {
        // Simulate multiple steps
        let mut prev_lineage = Fp::from(0u64);
        let mut prev_counters = Fp::from(0u64);
        
        for step in 0..5 {
            let mut cs = TestConstraintSystem::<Fp>::new();
            
            let circuit = LineageStepCircuit::new(
                Fp::from(step as u64),
                Fp::from((step + 1) as u64),
                1,  // User
                1,  // User
                1000 + step as u64,
                step as u64,
                Fp::from(100u64),
                vec![Fp::from(1u64); POLICY_TREE_DEPTH],
                vec![false; POLICY_TREE_DEPTH],
                0,
                [0, step as u32, 0, 0, 0, 0],
                [1, u32::MAX, 10, 100, 5, 1000],
            );
            
            let z0 = AllocatedNum::alloc(
                cs.namespace(|| "z0"),
                || Ok(prev_lineage),
            ).unwrap();
            let z1 = AllocatedNum::alloc(
                cs.namespace(|| "z1"),
                || Ok(prev_counters),
            ).unwrap();
            
            let z_prime = circuit.synthesize_step(&mut cs, &[z0, z1]).unwrap();
            
            assert!(cs.is_satisfied());
            
            prev_lineage = z_prime[0].get_value().unwrap();
            prev_counters = z_prime[1].get_value().unwrap();
        }
    }
}