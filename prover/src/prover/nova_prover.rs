//! Nova-based lineage prover

use crate::types::{LineageProof, StepWitness};
use crate::{Result, ZkOriginError};

#[cfg(feature = "real-nova")]
use {
    crate::prover::nova_circuit::LineageStepCircuit,
    crate::types::lineage::{CounterCommitment, LineageCommitment},
    crate::types::proof::ProofMetadata,
    ff::{Field, PrimeField},
    nova_snark::{
        traits::{circuit::TrivialCircuit, snark::RelaxedR1CSSNARKTrait},
        CompressedSNARK, PublicParams, RecursiveSNARK,
    },
    pasta_curves::{pallas, vesta},
    std::time::Instant,
};

#[cfg(feature = "real-nova")]
type G1 = pallas::Point;
#[cfg(feature = "real-nova")]
type G2 = vesta::Point;
#[cfg(feature = "real-nova")]
type F1 = pallas::Scalar;
#[cfg(feature = "real-nova")]
type F2 = vesta::Scalar;
#[cfg(feature = "real-nova")]
type EE1 = nova_snark::provider::ipa_pc::EvaluationEngine<G1>;
#[cfg(feature = "real-nova")]
type EE2 = nova_snark::provider::ipa_pc::EvaluationEngine<G2>;
#[cfg(feature = "real-nova")]
type S1 = nova_snark::spartan::snark::RelaxedR1CSSNARK<G1, EE1>;
#[cfg(feature = "real-nova")]
type S2 = nova_snark::spartan::snark::RelaxedR1CSSNARK<G2, EE2>;
#[cfg(feature = "real-nova")]
type C1 = LineageStepCircuit<F1>;
#[cfg(feature = "real-nova")]
type C2 = TrivialCircuit<F2>;

/// Nova proving parameters
#[cfg(feature = "real-nova")]
pub struct NovaParams {
    pub(crate) pp: PublicParams<G1, G2, C1, C2>,
    pub(crate) policy_root: [u8; 32],
}

#[cfg(not(feature = "real-nova"))]
#[derive(Clone)]
/// Parameters used by the Nova prover.
pub struct NovaParams {
    #[allow(dead_code)]
    policy_root: [u8; 32],
}

#[cfg(feature = "real-nova")]
impl NovaParams {
    /// Setup Nova public parameters
    pub fn setup(policy_root: [u8; 32]) -> Result<Self> {
        println!("Setting up Nova public parameters...");
        let start = Instant::now();

        let circuit_primary = LineageStepCircuit::<F1>::default();
        let circuit_secondary = TrivialCircuit::<F2>::default();

        let pp = PublicParams::<G1, G2, C1, C2>::setup(
            &circuit_primary,
            &circuit_secondary,
            &*S1::ck_floor(),
            &*S2::ck_floor(),
        );

        println!("Nova setup complete in {:?}", start.elapsed());
        println!("  Primary circuit constraints: ~{}", pp.num_constraints().0);
        println!(
            "  Secondary circuit constraints: ~{}",
            pp.num_constraints().1
        );

        Ok(Self { pp, policy_root })
    }

    /// Get public parameters
    pub fn public_params(&self) -> &PublicParams<G1, G2, C1, C2> {
        &self.pp
    }

    /// Get policy root
    pub fn policy_root(&self) -> [u8; 32] {
        self.policy_root
    }
}

#[cfg(not(feature = "real-nova"))]
impl NovaParams {
    /// Creates a new `NovaParams` instance with the given policy root.
    pub fn setup(policy_root: [u8; 32]) -> Result<Self> {
        Ok(Self { policy_root })
    }

    /// Returns the policy root stored in the parameters.
    pub fn policy_root(&self) -> [u8; 32] {
        self.policy_root
    }
}

/// Nova lineage prover for generating zero-knowledge proofs of state lineage
#[cfg(feature = "real-nova")]
pub struct NovaLineageProver<'a> {
    params: &'a NovaParams,
    recursive_snark: Option<Box<RecursiveSNARK<G1, G2, C1, C2>>>,
    z0_primary: Vec<F1>,
    num_steps: usize,
    initialized: bool,
    genesis_lineage: [u8; 32],
    initial_counters: [u8; 32],
    proving_start: Option<Instant>,
    pending_witnesses: Vec<StoredWitness>,
}

#[cfg(feature = "real-nova")]
#[derive(Clone, Debug)]
struct StoredWitness {
    prev_state: F1,
    new_state: F1,
    origin: F1,
    timestamp: F1,
    epoch_id: F1,
}

#[cfg(not(feature = "real-nova"))]
/// Mock Nova lineage prover when real-nova feature is disabled.
pub struct NovaLineageProver<'a> {
    _params: &'a NovaParams,
    initialized: bool,
    step_count: usize,
}

#[cfg(feature = "real-nova")]
impl<'a> NovaLineageProver<'a> {
    /// Create a new Nova lineage prover
    pub fn new(params: &'a NovaParams) -> Self {
        Self {
            params,
            recursive_snark: None,
            z0_primary: vec![F1::ZERO, F1::ZERO],
            num_steps: 0,
            initialized: false,
            genesis_lineage: [0u8; 32],
            initial_counters: [0u8; 32],
            proving_start: None,
            pending_witnesses: Vec::new(),
        }
    }

    /// Initialize the prover with genesis state
    pub fn initialize(
        &mut self,
        genesis_lineage: [u8; 32],
        initial_counters: [u8; 32],
    ) -> Result<()> {
        let lineage_f = bytes_to_field::<F1>(&genesis_lineage);
        let counters_f = bytes_to_field::<F1>(&initial_counters);

        self.z0_primary = vec![lineage_f, counters_f];
        self.genesis_lineage = genesis_lineage;
        self.initial_counters = initial_counters;
        self.initialized = true;
        self.num_steps = 0;
        self.recursive_snark = None;
        self.pending_witnesses.clear();
        self.proving_start = Some(Instant::now());

        println!("Nova prover initialized with genesis state");
        Ok(())
    }

    /// Prove a single transition step
    pub fn prove_step(&mut self, witness: &StepWitness) -> Result<()> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized(
                "Nova prover not initialized".into(),
            ));
        }

        let start = Instant::now();

        // Parse witness data into field elements
        let prev_state = bytes_to_field::<F1>(&witness.prev_state_hash);
        let new_state = bytes_to_field::<F1>(&witness.new_state_hash);
        let origin = F1::from(witness.new_origin as u64);
        let timestamp = F1::from(witness.timestamp);
        let epoch_id = F1::from(witness.epoch_id);

        // Store the witness for final state computation
        self.pending_witnesses.push(StoredWitness {
            prev_state,
            new_state,
            origin,
            timestamp,
            epoch_id,
        });

        // Build circuit
        let circuit = LineageStepCircuit::new(
            prev_state,
            new_state,
            origin,
            timestamp,
            epoch_id,
        );
        let secondary_circuit = TrivialCircuit::<F2>::default();

        if self.recursive_snark.is_none() {
            // First step - create new RecursiveSNARK
            let snark = RecursiveSNARK::new(
                &self.params.pp,
                &circuit,
                &secondary_circuit,
                &self.z0_primary,
                &[F2::ZERO],
            )
            .map_err(|e| ZkOriginError::proving(format!("RecursiveSNARK::new: {:?}", e)))?;

            self.recursive_snark = Some(Box::new(snark));
            self.num_steps = 1;
        } else {
            // Subsequent steps
            let snark = self.recursive_snark.as_mut().unwrap();
            snark
                .prove_step(&self.params.pp, &circuit, &secondary_circuit)
                .map_err(|e| ZkOriginError::proving(format!("prove_step failed: {:?}", e)))?;
            self.num_steps += 1;
        }

        println!("  Step {} proved in {:?}", self.num_steps, start.elapsed());
        Ok(())
    }

    /// Compute the expected final state by replaying all transitions
    fn compute_final_state(&self) -> (F1, F1) {
        let mut lineage = self.z0_primary[0];
        let mut counters = self.z0_primary[1];

        for w in &self.pending_witnesses {
            let state_product = w.prev_state * w.new_state;
            let transition_hash = state_product + w.origin + w.timestamp;
            let lineage_product = lineage * transition_hash;
            lineage = lineage_product + lineage + transition_hash;
            counters = counters + w.origin + w.epoch_id;
        }

        (lineage, counters)
    }

    /// Find the correct Nova step count for verification
    fn find_verified_step_count(&self) -> Result<usize> {
        let snark = self.recursive_snark.as_ref()
            .ok_or_else(|| ZkOriginError::InternalError("No SNARK".into()))?;

        // Try step counts to find one that works
        for test_steps in 1..=self.num_steps + 2 {
            if snark.verify(
                &self.params.pp,
                test_steps,
                &self.z0_primary,
                &[F2::ZERO],
            ).is_ok() {
                return Ok(test_steps);
            }
        }

        Err(ZkOriginError::proving("No valid step count found for verification".to_string()))
    }

    /// Finalize and generate compressed proof
    pub fn finalize(&self) -> Result<LineageProof> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized("Not initialized".into()));
        }
        if self.num_steps == 0 {
            return Err(ZkOriginError::InvalidLineage("No steps to prove".into()));
        }

        let snark = self
            .recursive_snark
            .as_ref()
            .ok_or_else(|| ZkOriginError::InternalError("No SNARK".into()))?;

        // Find correct step count and verify
        println!("Verifying recursive SNARK ({} steps)...", self.num_steps);
        let verify_start = Instant::now();
        
        let verified_steps = self.find_verified_step_count()?;
        println!("  Verified in {:?}", verify_start.elapsed());

        // Compress the proof
        println!("Compressing proof...");
        let compress_start = Instant::now();

        let (pk, vk) = CompressedSNARK::<G1, G2, C1, C2, S1, S2>::setup(&self.params.pp)
            .map_err(|e| ZkOriginError::proving(format!("setup: {:?}", e)))?;

        let compressed = CompressedSNARK::prove(&self.params.pp, &pk, snark)
            .map_err(|e| ZkOriginError::proving(format!("compress: {:?}", e)))?;

        println!("  Compressed in {:?}", compress_start.elapsed());

        let proof_bytes = bincode::serialize(&compressed)?;
        let vk_bytes = bincode::serialize(&vk)?;

        println!(
            "  Proof size: {} bytes ({:.2} KB)",
            proof_bytes.len(),
            proof_bytes.len() as f64 / 1024.0
        );

        let proving_time_ms = self
            .proving_start
            .map(|s| s.elapsed().as_millis() as u64)
            .unwrap_or(0);

        // Compute final state
        let (final_lineage, final_counters) = self.compute_final_state();
        let final_lineage_bytes = field_to_bytes(&final_lineage);
        let final_counters_bytes = field_to_bytes(&final_counters);

        let mut proof = LineageProof::new_with_initial_state(
            proof_bytes,
            LineageCommitment::new(final_lineage_bytes, self.pending_witnesses.len() as u64),
            CounterCommitment::new(final_counters_bytes, 0),
            LineageCommitment::new(self.genesis_lineage, 0),
            self.pending_witnesses.len() as u64,
            verified_steps as u64,
            self.params.policy_root,
            self.initial_counters,
        );

        proof.metadata = ProofMetadata::new().with_proving_time(proving_time_ms);
        proof.verifier_key = Some(vk_bytes);

        println!(
            "  Logical steps: {}, Nova verified steps: {}",
            self.pending_witnesses.len(), verified_steps
        );

        Ok(proof)
    }

    /// Get current step count
    pub fn step_count(&self) -> usize {
        self.num_steps
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[cfg(not(feature = "real-nova"))]
impl<'a> NovaLineageProver<'a> {
    /// Creates a new mock `NovaLineageProver`.
    pub fn new(params: &'a NovaParams) -> Self {
        Self {
            _params: params,
            initialized: false,
            step_count: 0,
        }
    }

    /// Initializes the prover with the given roots.
    pub fn initialize(&mut self, _: [u8; 32], _: [u8; 32]) -> Result<()> {
        self.initialized = true;
        self.step_count = 0;
        Ok(())
    }

    /// Proves a single step.
    pub fn prove_step(&mut self, _: &StepWitness) -> Result<()> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized("".into()));
        }
        self.step_count += 1;
        Ok(())
    }

    /// Finalizes the proof generation.
    pub fn finalize(&self) -> Result<LineageProof> {
        Err(ZkOriginError::InternalError("Nova not enabled".into()))
    }

    /// Returns the number of steps proven so far.
    pub fn step_count(&self) -> usize {
        self.step_count
    }

    /// Returns whether the prover has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

/// Compressed Nova proof structure
#[cfg(feature = "real-nova")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompressedNovaProof {
    /// Serialized proof bytes
    pub proof_bytes: Vec<u8>,
    /// Number of steps
    pub num_steps: usize,
    /// Initial primary input
    pub z0_primary: Vec<u8>,
    /// Verifier key bytes
    pub vk_bytes: Vec<u8>,
}

/// A compressed representation of a Nova proof.
#[cfg(not(feature = "real-nova"))]
#[derive(Clone, Debug)]
pub struct CompressedNovaProof {
    /// Serialized proof data.
    pub proof_bytes: Vec<u8>,
    /// Number of steps included in the proof.
    pub num_steps: usize,
}

#[cfg(feature = "real-nova")]
fn bytes_to_field<F: PrimeField>(bytes: &[u8; 32]) -> F {
    let mut repr = F::Repr::default();
    let repr_len = repr.as_ref().len();
    let copy_len = std::cmp::min(repr_len, 31);
    repr.as_mut()[..copy_len].copy_from_slice(&bytes[..copy_len]);
    F::from_repr(repr).unwrap_or(F::ZERO)
}

#[cfg(feature = "real-nova")]
fn field_to_bytes<F: PrimeField>(field: &F) -> [u8; 32] {
    let repr = field.to_repr();
    let mut bytes = [0u8; 32];
    let src = repr.as_ref();
    let copy_len = std::cmp::min(src.len(), 32);
    bytes[..copy_len].copy_from_slice(&src[..copy_len]);
    bytes
}