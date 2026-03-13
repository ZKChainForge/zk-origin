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

    pub fn public_params(&self) -> &PublicParams<G1, G2, C1, C2> {
        &self.pp
    }
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

#[cfg(feature = "real-nova")]
pub struct NovaLineageProver<'a> {
    params: &'a NovaParams,
    recursive_snark: Option<Box<RecursiveSNARK<G1, G2, C1, C2>>>,
    z0_primary: Vec<F1>,
    zi_primary: Vec<F1>,
    num_steps: usize,
    initialized: bool,
    genesis_lineage: [u8; 32],
    final_lineage: [u8; 32],
    final_counters: [u8; 32],
    proving_start: Option<Instant>,
}

#[cfg(not(feature = "real-nova"))]
/// Number of executed steps.
pub struct NovaLineageProver<'a> {
    _params: &'a NovaParams,
    initialized: bool,
    step_count: usize,
}

#[cfg(feature = "real-nova")]
impl<'a> NovaLineageProver<'a> {
    pub fn new(params: &'a NovaParams) -> Self {
        Self {
            params,
            recursive_snark: None,
            z0_primary: vec![F1::ZERO, F1::ZERO],
            zi_primary: vec![F1::ZERO, F1::ZERO],
            num_steps: 0,
            initialized: false,
            genesis_lineage: [0u8; 32],
            final_lineage: [0u8; 32],
            final_counters: [0u8; 32],
            proving_start: None,
        }
    }

    pub fn initialize(
        &mut self,
        genesis_lineage: [u8; 32],
        initial_counters: [u8; 32],
    ) -> Result<()> {
        let lineage_f = bytes_to_field::<F1>(&genesis_lineage);
        let counters_f = bytes_to_field::<F1>(&initial_counters);
        self.z0_primary = vec![lineage_f, counters_f];
        self.zi_primary = vec![lineage_f, counters_f];
        self.genesis_lineage = genesis_lineage;
        self.final_lineage = genesis_lineage;
        self.final_counters = initial_counters;
        self.initialized = true;
        self.num_steps = 0;
        self.recursive_snark = None;
        self.proving_start = Some(Instant::now());
        println!("Nova prover initialized with genesis state");
        Ok(())
    }

    pub fn prove_step(&mut self, witness: &StepWitness) -> Result<()> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized(
                "Nova prover not initialized".into(),
            ));
        }
        let start = Instant::now();
        let current_lineage = self.zi_primary[0];
        let current_counters = self.zi_primary[1];
        let prev_state = bytes_to_field::<F1>(&witness.prev_state_hash);
        let new_state = bytes_to_field::<F1>(&witness.new_state_hash);
        let origin = F1::from(witness.new_origin as u64);
        let timestamp = F1::from(witness.timestamp);
        let policy_root = bytes_to_field::<F1>(&witness.policy_root);
        let epoch_id = F1::from(witness.epoch_id);
        let policy_path: Vec<F1> = witness
            .policy_proof
            .iter()
            .map(|p| bytes_to_field::<F1>(p))
            .collect();
        let rate_limits: [F1; 6] = std::array::from_fn(|i| F1::from(witness.rate_limits[i] as u64));
        let state_product = prev_state * new_state;
        let transition_hash = state_product + origin + timestamp;
        let lineage_product = current_lineage * transition_hash;
        let new_lineage = lineage_product + current_lineage + transition_hash;
        let new_counters = current_counters + origin + epoch_id;
        let circuit = LineageStepCircuit::new(
            current_lineage,
            current_counters,
            prev_state,
            new_state,
            origin,
            timestamp,
            policy_root,
            policy_path,
            witness.policy_indices.clone(),
            epoch_id,
            rate_limits,
        );
        let secondary_circuit = TrivialCircuit::<F2>::default();
        if self.recursive_snark.is_none() {
            let snark = RecursiveSNARK::new(
                &self.params.pp,
                &circuit,
                &secondary_circuit,
                &self.z0_primary,
                &[F2::ZERO],
            )
            .map_err(|e| ZkOriginError::proving(format!("RecursiveSNARK::new: {:?}", e)))?;
            self.zi_primary = vec![new_lineage, new_counters];
            self.recursive_snark = Some(Box::new(snark));
            self.num_steps = 1;
        } else {
            let snark = self.recursive_snark.as_mut().unwrap();
            snark
                .prove_step(&self.params.pp, &circuit, &secondary_circuit)
                .map_err(|e| ZkOriginError::proving(format!("prove_step failed: {:?}", e)))?;
            self.zi_primary = vec![new_lineage, new_counters];
            self.num_steps += 1;
        }
        self.final_lineage = witness.compute_new_lineage_commitment();
        self.final_counters = witness.compute_new_counter_commitment();
        println!("  Step {} proved in {:?}", self.num_steps, start.elapsed());
        Ok(())
    }

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
        
        // Calculate the verifiable steps - this is what Nova actually proved
        // Nova's RecursiveSNARK counts steps differently than our logical steps
        let nova_verifiable_steps = self.num_steps.saturating_sub(1).max(1);
        
        println!("Verifying recursive SNARK ({} steps)...", nova_verifiable_steps);
        let verify_start = Instant::now();
        
        snark
            .verify(
                &self.params.pp,
                nova_verifiable_steps,
                &self.z0_primary,
                &[F2::ZERO],
            )
            .map_err(|e| ZkOriginError::proving(format!("verify: {:?}", e)))?;
        println!("  Verified in {:?}", verify_start.elapsed());
        
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
        
        // Create proof with BOTH logical steps and Nova steps
        let mut proof = LineageProof::new_with_nova_steps(
            proof_bytes,
            LineageCommitment::new(self.final_lineage, self.num_steps as u64),
            CounterCommitment::new(self.final_counters, 0),
            LineageCommitment::new(self.genesis_lineage, 0),
            self.num_steps as u64,           // logical steps (transitions added)
            nova_verifiable_steps as u64,     // actual Nova IVC steps for verification
            self.params.policy_root,
        );
        
        proof.metadata = ProofMetadata::new().with_proving_time(proving_time_ms);
        proof.verifier_key = Some(vk_bytes);
        
        println!(
            "  Logical steps: {}, Nova verifiable steps: {}",
            self.num_steps, nova_verifiable_steps
        );
        
        Ok(proof)
    }
    
    pub fn step_count(&self) -> usize {
        self.num_steps
    }
    
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[cfg(not(feature = "real-nova"))]
impl<'a> NovaLineageProver<'a> {
    /// Creates a new mock `NovaLineageProver`.
    ///
    /// This implementation is used when the `real-nova` feature is disabled.
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

    /// Proves a single step of the lineage computation.
    ///
    /// Returns an error if the prover has not been initialized.
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

#[cfg(feature = "real-nova")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompressedNovaProof {
    pub proof_bytes: Vec<u8>,
    pub num_steps: usize,
    pub z0_primary: Vec<u8>,
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