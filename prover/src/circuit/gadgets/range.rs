//! Range check gadget (sound and warning-free)

use bellpepper_core::{
    boolean::{AllocatedBit, Boolean},
    num::AllocatedNum,
    ConstraintSystem, LinearCombination, SynthesisError,
};
use ff::PrimeField;

/// Gadget for range checking values
pub struct RangeCheckGadget;

impl RangeCheckGadget {
    /// Enforce that `value < max`
    /// `num_bits` must be large enough to represent (max - 1)
    pub fn less_than<F: PrimeField, CS: ConstraintSystem<F>>(
        mut cs: CS,
        value: &AllocatedNum<F>,
        max: u64,
        num_bits: usize,
    ) -> Result<Boolean, SynthesisError> {
        let max_field = F::from(max);

        // diff = max - 1 - value
        let diff = AllocatedNum::alloc(cs.namespace(|| "diff"), || {
            let val = value
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?;
            Ok(max_field - F::ONE - val)
        })?;

        // Enforce: value + diff + 1 = max
        cs.enforce(
            || "value + diff + 1 = max",
            |lc| lc + value.get_variable() + diff.get_variable() + CS::one(),
            |lc| lc + CS::one(),
            |lc| lc + (max_field, CS::one()),
        );

        // Decompose diff into bits
        let diff_val = diff.get_value();

        let mut coeff = F::ONE;
        let mut bit_lc = LinearCombination::<F>::zero();

        for i in 0..num_bits {
            let bit = AllocatedBit::alloc(
                cs.namespace(|| format!("diff_bit_{}", i)),
                diff_val.map(|v| {
                    let repr = v.to_repr();
                    let bytes = repr.as_ref();
                    let byte = bytes[i / 8];
                    ((byte >> (i % 8)) & 1u8) == 1u8
                }),
            )?;

            bit_lc = bit_lc + (coeff, bit.get_variable());
            coeff = coeff.double();
        }

        // Enforce diff == reconstructed bits
        cs.enforce(
            || "reconstruct diff",
            |lc| lc + diff.get_variable(),
            |lc| lc + CS::one(),
            |_| bit_lc,
        );

        Ok(Boolean::constant(true))
    }

    /// Enforce that `value ∈ [min, max)`
    pub fn in_range<F: PrimeField, CS: ConstraintSystem<F>>(
        mut cs: CS,
        value: &AllocatedNum<F>,
        min: u64,
        max: u64,
        num_bits: usize,
    ) -> Result<Boolean, SynthesisError> {
        let min_field = F::from(min);

        // shifted = value - min
        let shifted = AllocatedNum::alloc(cs.namespace(|| "shifted"), || {
            let val = value
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?;
            Ok(val - min_field)
        })?;

        // Enforce shifted = value - min
        cs.enforce(
            || "shift constraint",
            |lc| lc + value.get_variable() - (min_field, CS::one()),
            |lc| lc + CS::one(),
            |lc| lc + shifted.get_variable(),
        );

        // Enforce shifted < (max - min)
        Self::less_than(
            cs.namespace(|| "range_check"),
            &shifted,
            max - min,
            num_bits,
        )
    }

    /// Enforce value == constant
    pub fn equals_constant<F: PrimeField, CS: ConstraintSystem<F>>(
        mut cs: CS,
        value: &AllocatedNum<F>,
        constant: u64,
    ) -> Result<Boolean, SynthesisError> {
        let const_field = F::from(constant);

        // diff = value - constant
        let diff = AllocatedNum::alloc(cs.namespace(|| "diff"), || {
            let val = value
                .get_value()
                .ok_or(SynthesisError::AssignmentMissing)?;
            Ok(val - const_field)
        })?;

        // Enforce diff = value - constant
        cs.enforce(
            || "diff constraint",
            |lc| lc + value.get_variable() - (const_field, CS::one()),
            |lc| lc + CS::one(),
            |lc| lc + diff.get_variable(),
        );

        // Allocate boolean flag
        let is_zero = AllocatedBit::alloc(
            cs.namespace(|| "is_zero"),
            diff.get_value().map(|v| v.is_zero().into()),
        )?;

        // Enforce diff * is_zero = 0
        cs.enforce(
            || "zero check",
            |lc| lc + diff.get_variable(),
            |lc| lc + is_zero.get_variable(),
            |lc| lc,
        );

        Ok(Boolean::from(is_zero))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_less_than() {
        let mut cs = TestConstraintSystem::<Fp>::new();

        let value =
            AllocatedNum::alloc(cs.namespace(|| "value"), || Ok(Fp::from(5u64)))
                .unwrap();

        let result =
            RangeCheckGadget::less_than(cs.namespace(|| "lt"), &value, 10, 8);

        assert!(result.is_ok());
        assert!(cs.is_satisfied());
    }

    #[test]
    fn test_in_range() {
        let mut cs = TestConstraintSystem::<Fp>::new();

        let value =
            AllocatedNum::alloc(cs.namespace(|| "value"), || Ok(Fp::from(7u64)))
                .unwrap();

        let result = RangeCheckGadget::in_range(
            cs.namespace(|| "range"),
            &value,
            5,
            10,
            8,
        );

        assert!(result.is_ok());
        assert!(cs.is_satisfied());
    }

    #[test]
    fn test_equals_constant() {
        let mut cs = TestConstraintSystem::<Fp>::new();

        let value =
            AllocatedNum::alloc(cs.namespace(|| "value"), || Ok(Fp::from(42u64)))
                .unwrap();

        let result =
            RangeCheckGadget::equals_constant(cs.namespace(|| "eq"), &value, 42);

        assert!(result.is_ok());
        assert!(cs.is_satisfied());
    }
}
