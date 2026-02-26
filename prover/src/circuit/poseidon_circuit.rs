//! Poseidon hash gadget for in-circuit computation
//!
//! Uses Neptune's circuit-compatible Poseidon implementation.

use bellpepper_core::{
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use ff::PrimeField;
use neptune::circuit2::poseidon_hash_allocated;
use neptune::poseidon::PoseidonConstants;
use generic_array::typenum::{U2, U3, U4, U5};

/// Poseidon circuit gadget
pub struct PoseidonCircuit<F: PrimeField> {
    constants_2: PoseidonConstants<F, U2>,
    constants_3: PoseidonConstants<F, U3>,
    constants_4: PoseidonConstants<F, U4>,
    constants_5: PoseidonConstants<F, U5>,
}

impl<F: PrimeField> PoseidonCircuit<F> {
    /// Create new Poseidon circuit gadget
    pub fn new() -> Self {
        Self {
            constants_2: PoseidonConstants::new(),
            constants_3: PoseidonConstants::new(),
            constants_4: PoseidonConstants::new(),
            constants_5: PoseidonConstants::new(),
        }
    }

    /// Hash two allocated numbers
    pub fn hash2<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        a: &AllocatedNum<F>,
        b: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        poseidon_hash_allocated(
            cs.namespace(|| "poseidon2"),
            vec![a.clone(), b.clone()],
            &self.constants_2,
        )
    }

    /// Hash three allocated numbers
    pub fn hash3<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        a: &AllocatedNum<F>,
        b: &AllocatedNum<F>,
        c: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        poseidon_hash_allocated(
            cs.namespace(|| "poseidon3"),
            vec![a.clone(), b.clone(), c.clone()],
            &self.constants_3,
        )
    }

    /// Hash four allocated numbers
    pub fn hash4<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        a: &AllocatedNum<F>,
        b: &AllocatedNum<F>,
        c: &AllocatedNum<F>,
        d: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        poseidon_hash_allocated(
            cs.namespace(|| "poseidon4"),
            vec![a.clone(), b.clone(), c.clone(), d.clone()],
            &self.constants_4,
        )
    }

    /// Hash five allocated numbers
    pub fn hash5<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        a: &AllocatedNum<F>,
        b: &AllocatedNum<F>,
        c: &AllocatedNum<F>,
        d: &AllocatedNum<F>,
        e: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        poseidon_hash_allocated(
            cs.namespace(|| "poseidon5"),
            vec![a.clone(), b.clone(), c.clone(), d.clone(), e.clone()],
            &self.constants_5,
        )
    }
}

impl<F: PrimeField> Default for PoseidonCircuit<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: PrimeField> Clone for PoseidonCircuit<F> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_poseidon2_circuit() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        let poseidon = PoseidonCircuit::new();
        
        let a = AllocatedNum::alloc(
            cs.namespace(|| "a"),
            || Ok(Fp::from(1u64)),
        ).unwrap();
        
        let b = AllocatedNum::alloc(
            cs.namespace(|| "b"),
            || Ok(Fp::from(2u64)),
        ).unwrap();
        
        let result = poseidon.hash2(&mut cs, &a, &b);
        
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        
        // Count constraints
        let num_constraints = cs.num_constraints();
        println!("Poseidon2 constraints: {}", num_constraints);
        assert!(num_constraints > 200); // Real Poseidon has ~300 constraints
    }

    #[test]
    fn test_poseidon3_circuit() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        let poseidon = PoseidonCircuit::new();
        
        let a = AllocatedNum::alloc(cs.namespace(|| "a"), || Ok(Fp::from(1u64))).unwrap();
        let b = AllocatedNum::alloc(cs.namespace(|| "b"), || Ok(Fp::from(2u64))).unwrap();
        let c = AllocatedNum::alloc(cs.namespace(|| "c"), || Ok(Fp::from(3u64))).unwrap();
        
        let result = poseidon.hash3(&mut cs, &a, &b, &c);
        
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
    }

    #[test]
    fn test_poseidon_deterministic() {
        let mut cs1 = TestConstraintSystem::<Fp>::new();
        let mut cs2 = TestConstraintSystem::<Fp>::new();
        let poseidon = PoseidonCircuit::new();
        
        let a1 = AllocatedNum::alloc(cs1.namespace(|| "a"), || Ok(Fp::from(42u64))).unwrap();
        let b1 = AllocatedNum::alloc(cs1.namespace(|| "b"), || Ok(Fp::from(99u64))).unwrap();
        
        let a2 = AllocatedNum::alloc(cs2.namespace(|| "a"), || Ok(Fp::from(42u64))).unwrap();
        let b2 = AllocatedNum::alloc(cs2.namespace(|| "b"), || Ok(Fp::from(99u64))).unwrap();
        
        let r1 = poseidon.hash2(&mut cs1, &a1, &b1).unwrap();
        let r2 = poseidon.hash2(&mut cs2, &a2, &b2).unwrap();
        
        assert_eq!(r1.get_value(), r2.get_value());
    }
}