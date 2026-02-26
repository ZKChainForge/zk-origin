//! Merkle proof verification gadget using real Poseidon

use bellpepper_core::{
    boolean::Boolean,
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use ff::PrimeField;

use super::selector::SelectorGadget;
use crate::circuit::poseidon_circuit::PoseidonCircuit;

/// Gadget for Merkle proof verification in circuit
pub struct MerkleGadget;

impl MerkleGadget {
    /// Verify a Merkle proof in circuit
    pub fn verify<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        leaf: &AllocatedNum<F>,
        expected_root: &AllocatedNum<F>,
        path: &[AllocatedNum<F>],
        indices: &[Boolean],
    ) -> Result<Boolean, SynthesisError> {
        if path.len() != indices.len() {
            return Err(SynthesisError::Unsatisfiable);
        }

        let poseidon = PoseidonCircuit::<F>::new();
        let mut current = leaf.clone();

        // Walk up the tree
        for (i, (sibling, is_right)) in path.iter().zip(indices.iter()).enumerate() {
            // If is_right is true: hash(sibling, current)
            // If is_right is false: hash(current, sibling)
            
            let (left, right) = Self::select_order(
                &mut cs.namespace(|| format!("select_{}", i)),
                &current,
                sibling,
                is_right,
            )?;
            
            // Hash with real Poseidon
            current = poseidon.hash2(
                &mut cs.namespace(|| format!("hash_{}", i)),
                &left,
                &right,
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

    /// Select left/right ordering based on boolean
    fn select_order<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        current: &AllocatedNum<F>,
        sibling: &AllocatedNum<F>,
        is_right: &Boolean,
    ) -> Result<(AllocatedNum<F>, AllocatedNum<F>), SynthesisError> {
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
        
        Ok((left, right))
    }

    /// Check if two allocated numbers are equal
    fn nums_equal<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        a: &AllocatedNum<F>,
        b: &AllocatedNum<F>,
    ) -> Result<Boolean, SynthesisError> {
        use bellpepper_core::boolean::AllocatedBit;
        
        // Compute difference
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
            |lc| lc + a.get_variable() - b.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + diff.get_variable(),
        );
        
        // is_zero = (diff == 0) ? 1 : 0
        let is_zero = AllocatedBit::alloc(
            cs.namespace(|| "is_zero"),
            diff.get_value().map(|d| d.is_zero().into()),
        )?;
        
        // If is_zero = 1, then diff must be 0
        cs.enforce(
            || "zero_check",
            |lc| lc + is_zero.get_variable(),
            |lc| lc + diff.get_variable(),
            |lc| lc,
        );
        
        Ok(Boolean::Is(is_zero))
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

        let poseidon = PoseidonCircuit::<F>::new();
        let mut current = leaf.clone();

        for (i, (sibling, is_right)) in path.iter().zip(indices.iter()).enumerate() {
            let (left, right) = Self::select_order(
                &mut cs.namespace(|| format!("select_{}", i)),
                &current,
                sibling,
                is_right,
            )?;
            
            current = poseidon.hash2(
                &mut cs.namespace(|| format!("hash_{}", i)),
                &left,
                &right,
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
    fn test_merkle_gadget_with_poseidon() {
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
        
        // With real Poseidon, we expect ~300 * 4 = 1200 constraints for hashing
        let num_constraints = cs.num_constraints();
        println!("Merkle gadget constraints: {}", num_constraints);
        assert!(num_constraints > 1000);
    }

    #[test]
    fn test_merkle_verification_satisfied() {
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
            &expected_root,
            &path,
            &indices,
        ).unwrap();
        
        assert!(is_valid.get_value().unwrap());
        assert!(cs.is_satisfied());
    }
}