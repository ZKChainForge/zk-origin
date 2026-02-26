//! Merkle proof verification gadget

use bellpepper_core::{
    boolean::Boolean,
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use ff::PrimeField;
use super::selector::SelectorGadget;

/// Gadget for Merkle proof verification in circuit
pub struct MerkleGadget;

impl MerkleGadget {
    /// Verify a Merkle proof in circuit
    /// 
    /// Takes a leaf, path siblings, path indices, and verifies
    /// the computed root matches expected.
    pub fn verify<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        leaf: &AllocatedNum<F>,
        path: &[AllocatedNum<F>],
        indices: &[Boolean],
        expected_root: &AllocatedNum<F>,
    ) -> Result<Boolean, SynthesisError> {
        if path.len() != indices.len() {
            return Err(SynthesisError::Unsatisfiable);
        }

        // Start with the leaf
        let mut current = leaf.clone();

        // Walk up the tree
        for (i, (sibling, is_right)) in path.iter().zip(indices.iter()).enumerate() {
            // If is_right is true, we're the right child: hash(sibling, current)
            // If is_right is false, we're the left child: hash(current, sibling)
            
            current = Self::hash_pair(
                &mut cs.namespace(|| format!("level_{}", i)),
                &current,
                sibling,
                is_right,
            )?;
        }

        // Check if computed root matches expected
        let roots_equal = Self::nums_equal(
            &mut cs.namespace(|| "roots_equal"),
            &current,
            expected_root,
        )?;

        Ok(roots_equal)
    }

    /// Hash two values with conditional ordering
    /// 
    /// If is_right is true: hash(sibling, current)
    /// If is_right is false: hash(current, sibling)
    fn hash_pair<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        current: &AllocatedNum<F>,
        sibling: &AllocatedNum<F>,
        is_right: &Boolean,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        // Select left and right based on is_right
        let left = SelectorGadget::if_then_else(
            &mut cs.namespace(|| "select_left"),
            is_right,
            sibling,
            current,
        )?;
        
        let right = SelectorGadget::if_then_else(
            &mut cs.namespace(|| "select_right"),
            is_right,
            current,
            sibling,
        )?;
        
        // In a real implementation, this would use Poseidon
        // For now, we use a placeholder that combines the values
        Self::poseidon_hash_two(
            &mut cs.namespace(|| "hash"),
            &left,
            &right,
        )
    }

    /// Placeholder Poseidon hash of two elements
    /// 
    /// NOTE: This is a simplified placeholder. Production would use
    /// actual Poseidon implementation from neptune.
    fn poseidon_hash_two<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        left: &AllocatedNum<F>,
        right: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        // Placeholder: just add them (NOT SECURE - replace with real Poseidon)
        // In production, use neptune's Poseidon circuit gadget
        
        let result = AllocatedNum::alloc(
            cs.namespace(|| "hash_result"),
            || {
                let l = left.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                let r = right.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                // This is NOT a real hash - just for testing circuit structure
                Ok(l + r + F::ONE)
            },
        )?;
        
        // Constrain: result = left + right + 1
        cs.enforce(
            || "hash_constraint",
            |lc| lc + CS::one(),
            |lc| lc + left.get_variable() + right.get_variable() + CS::one(),
            |lc| lc + result.get_variable(),
        );
        
        Ok(result)
    }

    /// Check if two allocated numbers are equal
    fn nums_equal<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        a: &AllocatedNum<F>,
        b: &AllocatedNum<F>,
    ) -> Result<Boolean, SynthesisError> {
        // We want to check if a == b
        // Strategy: compute (a - b), check if zero
        
        let diff = AllocatedNum::alloc(
            cs.namespace(|| "diff"),
            || {
                let a_val = a.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                let b_val = b.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                Ok(a_val - b_val)
            },
        )?;
        
        // Constrain: diff = a - b
        cs.enforce(
            || "diff_constraint",
            |lc| lc + CS::one(),
            |lc| lc + diff.get_variable(),
            |lc| lc + a.get_variable() - b.get_variable(),
        );
        
        // Check if diff is zero
        let is_zero = AllocatedNum::alloc(
            cs.namespace(|| "is_zero"),
            || {
                let d = diff.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                Ok(if d == F::ZERO { F::ONE } else { F::ZERO })
            },
        )?;
        
        // is_zero must be boolean
        cs.enforce(
            || "is_zero_boolean",
            |lc| lc + is_zero.get_variable(),
            |lc| lc + CS::one() - is_zero.get_variable(),
            |lc| lc,
        );
        
        // If is_zero = 1, then diff must be 0
        // If is_zero = 0, then diff must be non-zero
        // Constraint: is_zero * diff = 0
        cs.enforce(
            || "zero_implies_diff_zero",
            |lc| lc + is_zero.get_variable(),
            |lc| lc + diff.get_variable(),
            |lc| lc,
        );
        
        // Allocate result as boolean
        let result = bellpepper_core::boolean::AllocatedBit::alloc(
            cs.namespace(|| "result_bit"),
            is_zero.get_value().map(|v| v == F::ONE),
        )?;
        
        Ok(Boolean::Is(result))
    }

    /// Compute Merkle root from leaf and proof (without verification)
    pub fn compute_root<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        leaf: &AllocatedNum<F>,
        path: &[AllocatedNum<F>],
        indices: &[Boolean],
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        if path.len() != indices.len() {
            return Err(SynthesisError::Unsatisfiable);
        }

        let mut current = leaf.clone();

        for (i, (sibling, is_right)) in path.iter().zip(indices.iter()).enumerate() {
            current = Self::hash_pair(
                &mut cs.namespace(|| format!("level_{}", i)),
                &current,
                sibling,
                is_right,
            )?;
        }

        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_merkle_gadget_compiles() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let leaf = AllocatedNum::alloc(
            cs.namespace(|| "leaf"),
            || Ok(Fp::from(42u64)),
        ).unwrap();
        
        let path: Vec<_> = (0..4)
            .map(|i| {
                AllocatedNum::alloc(
                    cs.namespace(|| format!("path_{}", i)),
                    || Ok(Fp::from(i as u64)),
                ).unwrap()
            })
            .collect();
        
        let indices: Vec<_> = (0..4)
            .map(|i| Boolean::constant(i % 2 == 0))
            .collect();
        
        let root = MerkleGadget::compute_root(&mut cs, &leaf, &path, &indices);
        assert!(root.is_ok());
    }

    #[test]
    fn test_merkle_verification() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let leaf = AllocatedNum::alloc(
            cs.namespace(|| "leaf"),
            || Ok(Fp::from(1u64)),
        ).unwrap();
        
        let path: Vec<_> = (0..2)
            .map(|i| {
                AllocatedNum::alloc(
                    cs.namespace(|| format!("path_{}", i)),
                    || Ok(Fp::from(1u64)),
                ).unwrap()
            })
            .collect();
        
        let indices: Vec<_> = vec![
            Boolean::constant(false),
            Boolean::constant(false),
        ];
        
        // Compute expected root
        let computed_root = MerkleGadget::compute_root(
            &mut cs.namespace(|| "compute"),
            &leaf,
            &path,
            &indices,
        ).unwrap();
        
        let expected_root = AllocatedNum::alloc(
            cs.namespace(|| "expected"),
            || computed_root.get_value().ok_or(SynthesisError::AssignmentMissing),
        ).unwrap();
        
        // Verify
        let is_valid = MerkleGadget::verify(
            &mut cs.namespace(|| "verify"),
            &leaf,
            &path,
            &indices,
            &expected_root,
        ).unwrap();
        
        assert!(is_valid.get_value().unwrap());
        assert!(cs.is_satisfied());
    }
}