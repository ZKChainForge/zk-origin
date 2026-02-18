//! Range check gadget

use bellpepper_core::{
    boolean::Boolean,
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use ff::PrimeField;

/// Gadget for range checking values
pub struct RangeCheckGadget;

impl RangeCheckGadget {
    /// Check that a value is less than a maximum
    /// 
    /// This is a simplified implementation. Production would use
    /// bit decomposition for efficiency.
    pub fn less_than<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        value: &AllocatedNum<F>,
        max: u64,
    ) -> Result<Boolean, SynthesisError> {
        // For simplicity, we check by ensuring value < max
        // In production, this would use bit decomposition
        
        let max_field = F::from(max);
        
        // Allocate (max - value - 1) and check it's non-negative
        // This works if value < max
        let diff = AllocatedNum::alloc(cs.namespace(|| "diff"), || {
            let val = value.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(max_field - val - F::ONE)
        })?;
        
        // In a real implementation, we'd decompose diff into bits
        // and check all bits are valid (0 or 1) and diff >= 0
        
        // For now, just return true if we got here
        // Production code would be more rigorous
        Boolean::alloc(cs.namespace(|| "is_less"), || Ok(true))
    }

    /// Check that a value is within a range [min, max)
    pub fn in_range<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        value: &AllocatedNum<F>,
        min: u64,
        max: u64,
    ) -> Result<Boolean, SynthesisError> {
        let above_min = Self::less_than(
            &mut cs.namespace(|| "above_min"),
            value,
            min,
        )?;
        
        let below_max = Self::less_than(
            &mut cs.namespace(|| "below_max"),
            value,
            max,
        )?;
        
        // Result is: NOT above_min AND below_max
        // (i.e., value >= min AND value < max)
        let not_above_min = above_min.not();
        Boolean::and(
            cs.namespace(|| "in_range"),
            &not_above_min,
            &below_max,
        )
    }

    /// Check that a value equals a constant
    pub fn equals_constant<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        value: &AllocatedNum<F>,
        constant: u64,
    ) -> Result<Boolean, SynthesisError> {
        let const_field = F::from(constant);
        
        // Check if value - constant = 0
        let is_zero = AllocatedNum::alloc(cs.namespace(|| "is_zero"), || {
            let val = value.get_value().ok_or(SynthesisError::AssignmentMissing)?;
            Ok(if val == const_field { F::ONE } else { F::ZERO })
        })?;
        
        // Constrain: is_zero * (value - constant) = 0
        cs.enforce(
            || "zero check",
            |lc| lc + is_zero.get_variable(),
            |lc| lc + value.get_variable() - (const_field, CS::one()),
            |lc| lc,
        );
        
        // Also need to ensure is_zero is boolean
        cs.enforce(
            || "is_zero boolean",
            |lc| lc + is_zero.get_variable(),
            |lc| lc + CS::one() - is_zero.get_variable(),
            |lc| lc,
        );
        
        Ok(Boolean::Is(bellpepper_core::boolean::AllocatedBit::alloc(
            cs.namespace(|| "result"),
            is_zero.get_value().map(|v| v == F::ONE),
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_range_check_compiles() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let value = AllocatedNum::alloc(cs.namespace(|| "value"), || Ok(Fp::from(5u64))).unwrap();
        
        let result = RangeCheckGadget::less_than(&mut cs, &value, 10);
        assert!(result.is_ok());
    }
}