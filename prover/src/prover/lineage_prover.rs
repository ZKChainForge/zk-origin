//! Main lineage prover implementation

#[cfg(all(feature = "real-nova", feature = "commitment-mode"))]
compile_error!("Enable only one of 'real-nova' or 'commitment-mode'");

use crate::prover::WitnessGenerator;
use crate::types::{LineageCommitment, LineageProof, OriginPolicy, Transition};
use crate::{Result, ZkOriginError};

#[cfg(not(feature = "real-nova"))]
use std::marker::PhantomData;

#[cfg(feature = "real-nova")]
use crate::prover::nova_prover::{NovaLineageProver, NovaParams};

#[cfg(feature = "commitment-mode")]
use crate::prover::commitment_prover::{CommitmentParams, CommitmentProver};

/// The main prover for generating lineage proofs
pub struct LineageProver<'a> {
    /// Policy being enforced
    policy: OriginPolicy,

    /// Witness generator
    witness_gen: WitnessGenerator,

    /// Genesis commitment
    genesis_commitment: LineageCommitment,

    /// Number of transitions processed
    num_transitions: u64,

    /// Whether initialized
    initialized: bool,

    /// Backend prover (Nova or Commitment)
    #[cfg(feature = "real-nova")]
    backend: Option<NovaLineageProver<'a>>,

    #[cfg(feature = "commitment-mode")]
    backend: Option<CommitmentProver>,

    /// Nova params borrowed from caller
    #[cfg(feature = "real-nova")]
    nova_params: &'a NovaParams,

    #[cfg(not(feature = "real-nova"))]
    _marker: PhantomData<&'a ()>,
}

impl<'a> LineageProver<'a> {
    /// Create a new lineage prover with Nova backend.
    #[cfg(feature = "real-nova")]
    pub fn new(policy: OriginPolicy, params: &'a NovaParams) -> Result<Self> {
        println!("Initializing LineageProver with Nova backend...");
        Ok(Self {
            policy: policy.clone(),
            witness_gen: WitnessGenerator::new(policy),
            genesis_commitment: LineageCommitment::zero(),
            num_transitions: 0,
            initialized: false,
            backend: None,
            nova_params: params,
        })
    }

    /// Create a new lineage prover with commitment backend (fast, not ZK).
    #[cfg(not(feature = "real-nova"))]
    pub fn new(policy: OriginPolicy) -> Result<Self> {
        #[cfg(feature = "commitment-mode")]
        println!("Initializing LineageProver with Commitment backend (fast, NOT ZK)");
        Ok(Self {
            policy: policy.clone(),
            witness_gen: WitnessGenerator::new(policy),
            genesis_commitment: LineageCommitment::zero(),
            num_transitions: 0,
            initialized: false,
            backend: None,
            _marker: PhantomData,
        })
    }

    /// Setup Nova parameters (takes 30-120 seconds).
    #[cfg(feature = "real-nova")]
    pub fn setup_params(policy: &OriginPolicy) -> Result<NovaParams> {
        println!("Setting up Nova parameters (this takes 30-120 seconds)...");
        let policy_root = policy.compute_hash();
        NovaParams::setup(policy_root)
    }

    /// Initialize the prover with a genesis state
    pub fn initialize(&mut self, genesis_state_hash: [u8; 32]) -> Result<()> {
        self.witness_gen.reset(genesis_state_hash);
        self.genesis_commitment = LineageCommitment::genesis(genesis_state_hash);
        self.num_transitions = 0;
        self.initialized = true;

        #[cfg(feature = "real-nova")]
        {
            let params: &'a NovaParams = self.nova_params;
            let mut prover = NovaLineageProver::new(params);

            let hasher = crate::hash::poseidon_native::NativePoseidonHasher::new();
            let genesis_lineage = hasher.compute_genesis_commitment(&genesis_state_hash);
            let initial_counters = hasher.compute_counter_commitment(0, &[0; 6]);

            prover.initialize(genesis_lineage, initial_counters)?;
            self.backend = Some(prover);
        }

        #[cfg(feature = "commitment-mode")]
        {
            let policy_root = self.policy.compute_hash();
            let params = CommitmentParams::new(policy_root);
            let mut prover = CommitmentProver::new(params);
            prover.initialize(genesis_state_hash, 0)?;
            self.backend = Some(prover);
        }

        Ok(())
    }

    /// Add a transition to the lineage
    pub fn add_transition(&mut self, transition: Transition) -> Result<()> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized(
                "Call initialize() before adding transitions".into(),
            ));
        }

        // Debug: print transition details
        #[cfg(feature = "real-nova")]
        {
            println!("    [LineageProver] Adding transition:");
            println!(
                "      prev_state: {:02x}{:02x}{:02x}{:02x}...",
                transition.prev_state_hash[0],
                transition.prev_state_hash[1],
                transition.prev_state_hash[2],
                transition.prev_state_hash[3]
            );
            println!(
                "      new_state:  {:02x}{:02x}{:02x}{:02x}...",
                transition.new_state_hash[0],
                transition.new_state_hash[1],
                transition.new_state_hash[2],
                transition.new_state_hash[3]
            );
            println!(
                "      origin: {:?}, timestamp: {}",
                transition.origin_class, transition.timestamp
            );
        }

        // Generate witness (validates transition)
        let witness = self.witness_gen.generate_witness(&transition)?;

        #[cfg(feature = "real-nova")]
        {
            println!("    [LineageProver] Witness generated, calling Nova prove_step...");
        }

        // Add to backend
        #[cfg(feature = "real-nova")]
        {
            let backend = self.backend.as_mut().ok_or(ZkOriginError::NotInitialized(
                "Backend not initialized".into(),
            ))?;
            backend.prove_step(&witness)?;
        }

        #[cfg(feature = "commitment-mode")]
        {
            let backend = self.backend.as_mut().ok_or(ZkOriginError::NotInitialized(
                "Backend not initialized".into(),
            ))?;
            backend.add_step(&witness)?;
        }

        self.num_transitions += 1;

        #[cfg(feature = "real-nova")]
        {
            println!(
                "    [LineageProver] Transition {} complete, Nova step_count: {}",
                self.num_transitions,
                self.backend.as_ref().map(|b| b.step_count()).unwrap_or(0)
            );
        }

        Ok(())
    }

    /// Add multiple transitions
    pub fn add_transitions(&mut self, transitions: Vec<Transition>) -> Result<()> {
        for transition in transitions {
            self.add_transition(transition)?;
        }
        Ok(())
    }

    /// Check if a transition would be valid without adding it
    pub fn validate_transition(&self, transition: &Transition) -> Result<()> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized(
                "Prover not initialized".into(),
            ));
        }

        self.witness_gen.would_be_valid(transition)
    }

    /// Finalize and generate the proof
    pub fn finalize(&self) -> Result<LineageProof> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized(
                "Prover not initialized".into(),
            ));
        }

        if self.num_transitions == 0 {
            return Err(ZkOriginError::InvalidLineage(
                "No transitions to prove".into(),
            ));
        }

        #[cfg(feature = "real-nova")]
        {
            let backend = self.backend.as_ref().ok_or(ZkOriginError::NotInitialized(
                "Backend not initialized".into(),
            ))?;

            println!(
                "[LineageProver] Finalizing with {} transitions, Nova step_count: {}",
                self.num_transitions,
                backend.step_count()
            );

            return backend.finalize();
        }

        #[cfg(feature = "commitment-mode")]
        {
            let backend = self.backend.as_ref().ok_or(ZkOriginError::NotInitialized(
                "Backend not initialized".into(),
            ))?;
            backend.finalize()
        }
    }

    /// Get current lineage commitment
    pub fn current_lineage(&self) -> Option<&LineageCommitment> {
        if self.initialized {
            Some(self.witness_gen.current_lineage())
        } else {
            None
        }
    }

    /// Get current depth
    pub fn current_depth(&self) -> u64 {
        self.num_transitions
    }

    /// Get the policy
    pub fn policy(&self) -> &OriginPolicy {
        &self.policy
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the proving mode
    pub fn proving_mode(&self) -> &'static str {
        crate::proving_mode()
    }

    /// Check if using real ZK
    pub fn is_real_zk(&self) -> bool {
        crate::is_real_zk_enabled()
    }

    /// Reset the prover
    pub fn reset(&mut self) {
        self.genesis_commitment = LineageCommitment::zero();
        self.num_transitions = 0;
        self.initialized = false;
        self.backend = None;
    }
}

/// Builder for LineageProver
pub struct LineageProverBuilder<'a> {
    policy: Option<OriginPolicy>,
    genesis_hash: Option<[u8; 32]>,
    #[cfg(feature = "real-nova")]
    nova_params: Option<&'a NovaParams>,
    #[cfg(not(feature = "real-nova"))]
    _marker: PhantomData<&'a ()>,
}

impl<'a> LineageProverBuilder<'a> {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            policy: None,
            genesis_hash: None,
            #[cfg(feature = "real-nova")]
            nova_params: None,
            #[cfg(not(feature = "real-nova"))]
            _marker: PhantomData,
        }
    }

    /// Set the policy
    pub fn policy(mut self, policy: OriginPolicy) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Set genesis state hash
    pub fn genesis(mut self, hash: [u8; 32]) -> Self {
        self.genesis_hash = Some(hash);
        self
    }

    /// Set Nova parameters (required for real-nova feature)
    #[cfg(feature = "real-nova")]
    pub fn with_params(mut self, params: &'a NovaParams) -> Self {
        self.nova_params = Some(params);
        self
    }

    /// Build the prover
    pub fn build(self) -> Result<LineageProver<'a>> {
        let policy = self.policy.unwrap_or_default();

        #[cfg(feature = "real-nova")]
        let mut prover = {
            let params = self.nova_params.ok_or_else(|| {
                ZkOriginError::NotInitialized(
                    "Nova params required; call .with_params() on the builder".into(),
                )
            })?;
            LineageProver::new(policy, params)?
        };

        #[cfg(not(feature = "real-nova"))]
        let mut prover = LineageProver::new(policy)?;

        if let Some(genesis) = self.genesis_hash {
            prover.initialize(genesis)?;
        }

        Ok(prover)
    }
}

impl Default for LineageProverBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OriginClass;

    #[cfg(not(feature = "real-nova"))]
    fn create_prover() -> LineageProver<'static> {
        let mut prover = LineageProver::new(OriginPolicy::default()).unwrap();
        prover.initialize([0u8; 32]).unwrap();
        prover
    }

    #[test]
    #[cfg(not(feature = "real-nova"))]
    fn test_prover_creation() {
        let prover = LineageProver::new(OriginPolicy::default());
        assert!(prover.is_ok());

        let prover = prover.unwrap();
        assert!(!prover.is_initialized());
        println!("Proving mode: {}", prover.proving_mode());
    }

    #[test]
    #[cfg(not(feature = "real-nova"))]
    fn test_prover_initialization() {
        let mut prover = LineageProver::new(OriginPolicy::default()).unwrap();
        let result = prover.initialize([42u8; 32]);
        assert!(result.is_ok());
        assert!(prover.is_initialized());
    }

    #[test]
    #[cfg(not(feature = "real-nova"))]
    fn test_add_transition() {
        let mut prover = create_prover();

        let transition = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);

        let result = prover.add_transition(transition);
        assert!(result.is_ok());
        assert_eq!(prover.current_depth(), 1);
    }

    #[test]
    #[cfg(not(feature = "real-nova"))]
    fn test_add_transition_not_initialized() {
        let mut prover = LineageProver::new(OriginPolicy::default()).unwrap();

        let transition = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);

        let result = prover.add_transition(transition);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(feature = "real-nova"))]
    fn test_finalize() {
        let mut prover = create_prover();

        for i in 0..3 {
            let transition = Transition::new(
                [i as u8; 32],
                [(i + 1) as u8; 32],
                OriginClass::User,
                1000 + i as u64,
            );
            prover.add_transition(transition).unwrap();
        }

        let proof = prover.finalize();
        assert!(proof.is_ok());

        let proof = proof.unwrap();
        assert_eq!(proof.num_steps, 3);
        assert!(!proof.proof_bytes.is_empty());
    }

    #[test]
    #[cfg(not(feature = "real-nova"))]
    fn test_policy_violation() {
        let mut prover = create_prover();

        let t1 = Transition::new([0u8; 32], [1u8; 32], OriginClass::User, 1000);
        prover.add_transition(t1).unwrap();

        let t2 = Transition::new([1u8; 32], [2u8; 32], OriginClass::Admin, 2000);
        let result = prover.add_transition(t2);

        assert!(result.is_err());
    }
}
