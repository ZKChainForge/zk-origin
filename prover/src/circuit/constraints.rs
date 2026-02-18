//! Constraint helpers for circuit construction

use bellpepper_core::{
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use ff::PrimeField;

/// Helper functions for common constraint patterns
pub struct ConstraintHelpers;

impl ConstraintHelpers {
    /// Constrain a value to equal a constant
    pub fn enforce_constant<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        var: &AllocatedNum<F>,
        constant: F,
    ) {
        cs.enforce(
            || "enforce_constant",
            |lc| lc + CS::one(),
            |lc| lc + var.get_variable(),
            |lc| lc + (constant, CS::one()),
        );
    }

    /// Constrain two values to be equal
    pub fn enforce_equal<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        a: &AllocatedNum<F>,
        b: &AllocatedNum<F>,
    ) {
        cs.enforce(
            || "enforce_equal",
            |lc| lc + CS::one(),
            |lc| lc + a.get_variable(),
            |lc| lc + b.get_variable(),
        );
    }

    /// Allocate a constant value
    pub fn alloc_constant<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        name: &str,
        value: F,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        let num = AllocatedNum::alloc(cs.namespace(|| name), || Ok(value))?;
        Self::enforce_constant(cs, &num, value);
        Ok(num)
    }

    /// Compute a + b with constraint
    pub fn add<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        a: &AllocatedNum<F>,
        b: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        let sum = AllocatedNum::alloc(cs.namespace(|| "sum"), || {
            let a_val = a.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let b_val = b.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(a_val + b_val)
        })?;

        cs.enforce(
            || "addition",
            |lc| lc + CS::one(),
            |lc| lc + sum.get_variable(),
            |lc| lc + a.get_variable() + b.get_variable(),
        );

        Ok(sum)
    }

    /// Compute a * b with constraint
    pub fn mul<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        a: &AllocatedNum<F>,
        b: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        let product = AllocatedNum::alloc(cs.namespace(|| "product"), || {
            let a_val = a.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            let b_val = b.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(a_val * b_val)
        })?;

        cs.enforce(
            || "multiplication",
            |lc| lc + a.get_variable(),
            |lc| lc + b.get_variable(),
            |lc| lc + product.get_variable(),
        );

        Ok(product)
    }

    /// Compute a + constant
    pub fn add_constant<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        a: &AllocatedNum<F>,
        constant: F,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        let sum = AllocatedNum::alloc(cs.namespace(|| "sum"), || {
            let a_val = a.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(a_val + constant)
        })?;

        cs.enforce(
            || "add_constant",
            |lc| lc + CS::one(),
            |lc| lc + sum.get_variable(),
            |lc| lc + a.get_variable() + (constant, CS::one()),
        );

        Ok(sum)
    }

    /// Compute a * constant
    pub fn mul_constant<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        a: &AllocatedNum<F>,
        constant: F,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        let product = AllocatedNum::alloc(cs.namespace(|| "product"), || {
            let a_val = a.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(a_val * constant)
        })?;

        cs.enforce(
            || "mul_constant",
            |lc| lc + a.get_variable(),
            |lc| lc + (constant, CS::one()),
            |lc| lc + product.get_variable(),
        );

        Ok(product)
    }

    /// Assert that a value is boolean (0 or 1)
    pub fn enforce_boolean<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        var: &AllocatedNum<F>,
    ) {
        // var * (1 - var) = 0
        cs.enforce(
            || "boolean_check",
            |lc| lc + var.get_variable(),
            |lc| lc + CS::one() - var.get_variable(),
            |lc| lc,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_add() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let a = AllocatedNum::alloc(cs.namespace(|| "a"), || Ok(Fp::from(5u64))).unwrap();
        let b = AllocatedNum::alloc(cs.namespace(|| "b"), || Ok(Fp::from(3u64))).unwrap();
        
        let sum = ConstraintHelpers::add(&mut cs, &a, &b).unwrap();
        
        assert_eq!(sum.get_value().unwrap(), Fp::from(8u64));
        assert!(cs.is_satisfied());
    }

    #[test]
    fn test_mul() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let a = AllocatedNum::alloc(cs.namespace(|| "a"), || Ok(Fp::from(5u64))).unwrap();
        let b = AllocatedNum::alloc(cs.namespace(|| "b"), || Ok(Fp::from(3u64))).unwrap();
        
        let product = ConstraintHelpers::mul(&mut cs, &a, &b).unwrap();
        
        assert_eq!(product.get_value().unwrap(), Fp::from(15u64));
        assert!(cs.is_satisfied());
    }

    #[test]
    fn test_enforce_boolean_valid() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let zero = AllocatedNum::alloc(cs.namespace(|| "zero"), || Ok(Fp::from(0u64))).unwrap();
        let one = AllocatedNum::alloc(cs.namespace(|| "one"), || Ok(Fp::from(1u64))).unwrap();
        
        ConstraintHelpers::enforce_boolean(&mut cs.namespace(|| "check_zero"), &zero);
        ConstraintHelpers::enforce_boolean(&mut cs.namespace(|| "check_one"), &one);
        
        assert!(cs.is_satisfied());
    }

    #[test]
    fn test_enforce_boolean_invalid() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let two = AllocatedNum::alloc(cs.namespace(|| "two"), || Ok(Fp::from(2u64))).unwrap();
        
        ConstraintHelpers::enforce_boolean(&mut cs, &two);
        
        assert!(!cs.is_satisfied());
    }
}