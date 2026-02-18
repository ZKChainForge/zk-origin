//! Selector/multiplexer gadget

use bellpepper_core::{
    boolean::Boolean,
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use ff::PrimeField;

/// Gadget for selecting values based on index
pub struct SelectorGadget;

impl SelectorGadget {
    /// Select one value from an array based on an index
    /// 
    /// Returns values[index] with constraints ensuring correct selection
    pub fn select<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        values: &[AllocatedNum<F>],
        index: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        if values.is_empty() {
            return Err(SynthesisError::Unsatisfiable);
        }

        let n = values.len();
        
        // Create indicator variables: is_i = (index == i) ? 1 : 0
        let mut indicators = Vec::with_capacity(n);
        
        for i in 0..n {
            let indicator = AllocatedNum::alloc(
                cs.namespace(|| format!("indicator_{}", i)),
                || {
                    let idx = index.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                    Ok(if idx == F::from(i as u64) { F::ONE } else { F::ZERO })
                },
            )?;
            
            // Constrain indicator to be boolean
            cs.enforce(
                || format!("indicator_{}_boolean", i),
                |lc| lc + indicator.get_variable(),
                |lc| lc + CS::one() - indicator.get_variable(),
                |lc| lc,
            );
            
            indicators.push(indicator);
        }
        
        // Constrain sum of indicators = 1 (exactly one selected)
        cs.enforce(
            || "sum_indicators_one",
            |lc| {
                let mut sum = lc;
                for ind in &indicators {
                    sum = sum + ind.get_variable();
                }
                sum
            },
            |lc| lc + CS::one(),
            |lc| lc + CS::one(),
        );
        
        // Constrain: for each i, indicator_i * (index - i) = 0
        for (i, indicator) in indicators.iter().enumerate() {
            cs.enforce(
                || format!("indicator_{}_correct", i),
                |lc| lc + indicator.get_variable(),
                |lc| lc + index.get_variable() - (F::from(i as u64), CS::one()),
                |lc| lc,
            );
        }
        
        // Compute selected value as sum of indicator_i * value_i
        let selected = AllocatedNum::alloc(
            cs.namespace(|| "selected"),
            || {
                let idx_val = index.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                
                // Find which index
                for (i, val) in values.iter().enumerate() {
                    if idx_val == F::from(i as u64) {
                        return val.get_value().ok_or(SynthesisError::AssignmentMissing);
                    }
                }
                
                Err(SynthesisError::Unsatisfiable)
            },
        )?;
        
        // Constrain: selected = sum(indicator_i * value_i)
        // This is done by: selected - sum(indicator_i * value_i) = 0
        // We need auxiliary variables for each product
        let mut products = Vec::with_capacity(n);
        for (i, (indicator, value)) in indicators.iter().zip(values.iter()).enumerate() {
            let product = AllocatedNum::alloc(
                cs.namespace(|| format!("product_{}", i)),
                || {
                    let ind = indicator.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                    let val = value.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                    Ok(ind * val)
                },
            )?;
            
            // Constrain: product = indicator * value
            cs.enforce(
                || format!("product_{}_correct", i),
                |lc| lc + indicator.get_variable(),
                |lc| lc + value.get_variable(),
                |lc| lc + product.get_variable(),
            );
            
            products.push(product);
        }
        
        // Constrain: selected = sum(products)
        cs.enforce(
            || "selected_equals_sum",
            |lc| lc + CS::one(),
            |lc| lc + selected.get_variable(),
            |lc| {
                let mut sum = lc;
                for prod in &products {
                    sum = sum + prod.get_variable();
                }
                sum
            },
        );
        
        Ok(selected)
    }

    /// Conditional selection: if condition then a else b
    pub fn if_then_else<F: PrimeField, CS: ConstraintSystem<F>>(
        cs: &mut CS,
        condition: &Boolean,
        if_true: &AllocatedNum<F>,
        if_false: &AllocatedNum<F>,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        // result = condition * if_true + (1 - condition) * if_false
        //        = if_false + condition * (if_true - if_false)
        
        let result = AllocatedNum::alloc(
            cs.namespace(|| "result"),
            || {
                let cond = condition.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                let t = if_true.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                let f = if_false.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                
                Ok(if cond { t } else { f })
            },
        )?;
        
        // Constrain: result = if_false + condition * (if_true - if_false)
        match condition {
            Boolean::Is(bit) => {
                let bit_num = AllocatedNum::alloc(
                    cs.namespace(|| "bit_as_num"),
                    || {
                        bit.get_value()
                            .map(|b| if b { F::ONE } else { F::ZERO })
                            .ok_or(SynthesisError::AssignmentMissing)
                    },
                )?;
                
                cs.enforce(
                    || "conditional_select",
                    |lc| lc + bit_num.get_variable(),
                    |lc| lc + if_true.get_variable() - if_false.get_variable(),
                    |lc| lc + result.get_variable() - if_false.get_variable(),
                );
            }
            Boolean::Not(bit) => {
                let bit_num = AllocatedNum::alloc(
                    cs.namespace(|| "bit_as_num"),
                    || {
                        bit.get_value()
                            .map(|b| if b { F::ONE } else { F::ZERO })
                            .ok_or(SynthesisError::AssignmentMissing)
                    },
                )?;
                
                // NOT condition = 1 - bit
                cs.enforce(
                    || "conditional_select_not",
                    |lc| lc + CS::one() - bit_num.get_variable(),
                    |lc| lc + if_true.get_variable() - if_false.get_variable(),
                    |lc| lc + result.get_variable() - if_false.get_variable(),
                );
            }
            Boolean::Constant(b) => {
                if *b {
                    cs.enforce(
                        || "result_equals_true",
                        |lc| lc + CS::one(),
                        |lc| lc + result.get_variable(),
                        |lc| lc + if_true.get_variable(),
                    );
                } else {
                    cs.enforce(
                        || "result_equals_false",
                        |lc| lc + CS::one(),
                        |lc| lc + result.get_variable(),
                        |lc| lc + if_false.get_variable(),
                    );
                }
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_selector() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let values: Vec<_> = (0..4)
            .map(|i| {
                AllocatedNum::alloc(
                    cs.namespace(|| format!("val_{}", i)),
                    || Ok(Fp::from((i * 10) as u64)),
                ).unwrap()
            })
            .collect();
        
        let index = AllocatedNum::alloc(
            cs.namespace(|| "index"),
            || Ok(Fp::from(2u64)),
        ).unwrap();
        
        let selected = SelectorGadget::select(&mut cs, &values, &index).unwrap();
        
        // Should select values[2] = 20
        assert_eq!(selected.get_value().unwrap(), Fp::from(20u64));
        assert!(cs.is_satisfied());
    }

    #[test]
    fn test_if_then_else() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        
        let if_true = AllocatedNum::alloc(
            cs.namespace(|| "if_true"),
            || Ok(Fp::from(100u64)),
        ).unwrap();
        
        let if_false = AllocatedNum::alloc(
            cs.namespace(|| "if_false"),
            || Ok(Fp::from(200u64)),
        ).unwrap();
        
        // Test with true condition
        let cond_true = Boolean::constant(true);
        let result = SelectorGadget::if_then_else(
            &mut cs.namespace(|| "select_true"),
            &cond_true,
            &if_true,
            &if_false,
        ).unwrap();
        
        assert_eq!(result.get_value().unwrap(), Fp::from(100u64));
    }
}