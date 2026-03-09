//! Witness generation for lineage proofs

use crate::types::lineage::EpochCounters;
use crate::types::{LineageCommitment, OriginClass, OriginPolicy, StepWitness, Transition};

use crate::hash::merkle::{build_policy_tree, generate_policy_proof};
use crate::{Result, ZkOriginError};

/// Generates witnesses for lineage step proofs
pub struct WitnessGenerator {
    /// The origin policy
    policy: OriginPolicy,

    /// Policy Merkle tree
    policy_tree: crate::hash::MerkleTree,

    /// Mapping from (from, to) to tree index
    policy_mapping: Vec<(u8, u8, usize)>,

    /// Current lineage state
    current_lineage: LineageCommitment,

    /// Previous origin class
    prev_origin: OriginClass,

    /// Current epoch counters
    counters: EpochCounters,

    /// Epoch duration in seconds
    epoch_duration: u64,
}

impl WitnessGenerator {
    /// Create a new witness generator
    pub fn new(policy: OriginPolicy) -> Self {
        let epoch_duration = policy.epoch_duration;

        // Build policy tree
        let allowed: Vec<(u8, u8)> = policy
            .allowed_transitions()
            .iter()
            .map(|(f, t)| (*f as u8, *t as u8))
            .collect();

        let (policy_tree, policy_mapping) = build_policy_tree(&allowed);

        Self {
            policy,
            policy_tree,
            policy_mapping,
            current_lineage: LineageCommitment::zero(),
            prev_origin: OriginClass::Genesis,
            counters: EpochCounters::new(0),
            epoch_duration,
        }
    }

    /// Reset to genesis state
    pub fn reset(&mut self, genesis_state_hash: [u8; 32]) {
        self.current_lineage = LineageCommitment::genesis(genesis_state_hash);
        self.prev_origin = OriginClass::Genesis;
        self.counters = EpochCounters::new(0);
    }

    /// Generate witness for a transition
    pub fn generate_witness(&mut self, transition: &Transition) -> Result<StepWitness> {
        // Check if transition is allowed by policy
        if !self
            .policy
            .is_allowed(self.prev_origin, transition.origin_class)
        {
            return Err(ZkOriginError::policy_violation(
                self.prev_origin,
                transition.origin_class,
            ));
        }

        // Check epoch and update if needed
        let transition_epoch = transition.timestamp / self.epoch_duration;
        if transition_epoch != self.counters.epoch {
            self.counters = EpochCounters::new(transition_epoch);
        }

        // Check rate limit
        let limit = self.policy.get_rate_limit(transition.origin_class);
        if self
            .counters
            .would_exceed_limit(transition.origin_class, limit)
        {
            return Err(ZkOriginError::rate_limit(
                transition.origin_class,
                self.counters.get(transition.origin_class),
                limit,
                self.counters.epoch,
            ));
        }

        // Generate policy proof
        let policy_proof = generate_policy_proof(
            &self.policy_tree,
            &self.policy_mapping,
            self.prev_origin as u8,
            transition.origin_class as u8,
        )
        .ok_or_else(|| {
            ZkOriginError::InvalidLineage("Cannot generate policy proof for transition".into())
        })?;

        // Build witness
        let witness = StepWitness::new(
            transition,
            self.current_lineage.value,
            self.prev_origin,
            self.current_lineage.depth,
            policy_proof.path.clone(),
            policy_proof.indices.clone(),
            self.policy_tree.root(),
            self.counters.epoch,
            self.counters.counts,
            [
                self.policy.get_rate_limit(OriginClass::Genesis),
                self.policy.get_rate_limit(OriginClass::User),
                self.policy.get_rate_limit(OriginClass::Admin),
                self.policy.get_rate_limit(OriginClass::Bridge),
                self.policy.get_rate_limit(OriginClass::Governance),
                self.policy.get_rate_limit(OriginClass::System),
            ],
            self.counters.compute_commitment().value,
        );

        // Update state
        self.current_lineage = LineageCommitment::new(
            witness.compute_new_lineage_commitment(),
            self.current_lineage.depth + 1,
        );
        self.prev_origin = transition.origin_class;
        self.counters.increment(transition.origin_class);

        Ok(witness)
    }

    /// Get current lineage commitment
    pub fn current_lineage(&self) -> &LineageCommitment {
        &self.current_lineage
    }

    /// Get current depth
    pub fn current_depth(&self) -> u64 {
        self.current_lineage.depth
    }

    /// Get current epoch counters
    pub fn current_counters(&self) -> &EpochCounters {
        &self.counters
    }

    /// Get policy root
    pub fn policy_root(&self) -> [u8; 32] {
        self.policy_tree.root()
    }

    /// Check if a transition would be valid (without executing)
    pub fn would_be_valid(&self, transition: &Transition) -> Result<()> {
        // Check policy
        if !self
            .policy
            .is_allowed(self.prev_origin, transition.origin_class)
        {
            return Err(ZkOriginError::policy_violation(
                self.prev_origin,
                transition.origin_class,
            ));
        }

        // Check rate limit
        let transition_epoch = transition.timestamp / self.epoch_duration;
        let counters = if transition_epoch != self.counters.epoch {
            EpochCounters::new(transition_epoch)
        } else {
            self.counters.clone()
        };

        let limit = self.policy.get_rate_limit(transition.origin_class);
        if counters.would_exceed_limit(transition.origin_class, limit) {
            return Err(ZkOriginError::rate_limit(
                transition.origin_class,
                counters.get(transition.origin_class),
                limit,
                counters.epoch,
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OriginPolicy;

    fn create_generator() -> WitnessGenerator {
        let policy = OriginPolicy::default();
        let mut gen = WitnessGenerator::new(policy);
        gen.reset([0u8; 32]);
        gen
    }

    #[test]
    fn test_witness_generator_creation() {
        let gen = create_generator();

        assert_eq!(gen.current_depth(), 0);
        assert_eq!(gen.prev_origin, OriginClass::Genesis);
    }
}
