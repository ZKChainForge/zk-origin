//! Merkle tree gadget for policy verification

use bellpepper_core::{
    boolean::Boolean,
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use ff::PrimeField;

use super::poseidon::PoseidonGadget;

/// Merkle tree verification gadget
pub struct MerkleGadget;

impl MerkleGadget {
    /// Verify a Merkle proof
    pub fn verify<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        leaf: &AllocatedNum<F>,
        root: &AllocatedNum<F>,
        path: &[AllocatedNum<F>],
        indices: &[Boolean],
    ) -> Result<Boolean, SynthesisError> {
        assert_eq!(path.len(), indices.len());
        
        let mut current = leaf.clone();
        
        for (i, (sibling, is_right)) in path.iter().zip(indices.iter()).enumerate() {
            // Select left and right based on index
            let (left, right) = Self::select(
                &mut cs.namespace(|| format!("select_{}", i)),
                &current,
                sibling,
                is_right,
            )?;
            
            // Hash
            current = PoseidonGadget::hash2(
                &mut cs.namespace(|| format!("hash_{}", i)),
                &left,
                &right,
            )?;
        }
        
        // Check if computed root matches expected root
        let diff = current.sub(&mut cs.namespace(|| "root_diff"), root)?;
        
        // Return true if diff is zero
        Ok(Boolean::Constant(true)) // Placeholder
    }
    
    /// Select left/right based on boolean
    fn select<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        a: &AllocatedNum<F>,
        b: &AllocatedNum<F>,
        is_right: &Boolean,
    ) -> Result<(AllocatedNum<F>, AllocatedNum<F>), SynthesisError> {
        // If is_right is true: (b, a), else (a, b)
        let left = AllocatedNum::alloc(cs.namespace(|| "left"), || {
            let av = a.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let bv = b.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            match is_right.get_value() {
                Some(true) => Ok(bv),
                Some(false) => Ok(av),
                None => Err(SynthesisError::AssignmentMissing),
            }
        })?;
        
        let right = AllocatedNum::alloc(cs.namespace(|| "right"), || {
            let av = a.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let bv = b.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            match is_right.get_value() {
                Some(true) => Ok(av),
                Some(false) => Ok(bv),
                None => Err(SynthesisError::AssignmentMissing),
            }
        })?;
        
        Ok((left, right))
    }
}