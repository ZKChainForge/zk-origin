//! Groth16-based prover for compact ZK proofs (<1KB)

use crate::types::StepWitness;
use crate::{Result, ZkOriginError};

#[cfg(feature = "compact-zk")]
use {
    crate::prover::groth16_circuit::{CompactLineageCircuit, TransitionWitness, MAX_TRANSITIONS},
    crate::types::lineage::{CounterCommitment, LineageCommitment},
    crate::types::proof::{LineageProof, ProofMetadata},
    bellman::groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
        Parameters, PreparedVerifyingKey, Proof, VerifyingKey,
    },
    bls12_381::{Bls12, G1Affine, G2Affine, Scalar as Fr},
    ff::Field,
    rand::rngs::OsRng,
    std::time::Instant,
};

/// Groth16 proving parameters
#[cfg(feature = "compact-zk")]
pub struct Groth16Params {
    params: Parameters<Bls12>,
    pvk: PreparedVerifyingKey<Bls12>,
    policy_root: [u8; 32],
}

#[cfg(not(feature = "compact-zk"))]
#[derive(Clone)]

/// This struct holds the public inputs needed to verify
/// or construct Groth16 proofs.
pub struct Groth16Params {
    /// Merkle root of the policy tree.
    #[allow(dead_code)]
    policy_root: [u8; 32],
}

#[cfg(feature = "compact-zk")]
impl Groth16Params {
    /// Setup Groth16 parameters (trusted setup)
    pub fn setup(policy_root: [u8; 32]) -> Result<Self> {
        println!("Setting up Groth16 parameters (trusted setup)...");
        let start = Instant::now();

        let circuit = CompactLineageCircuit::empty();

        let params = generate_random_parameters::<Bls12, _, _>(circuit, &mut OsRng)
            .map_err(|e| ZkOriginError::proving(format!("Groth16 setup failed: {:?}", e)))?;

        let pvk = prepare_verifying_key(&params.vk);

        println!("Groth16 setup complete in {:?}", start.elapsed());

        Ok(Self {
            params,
            pvk,
            policy_root,
        })
    }

    /// Get the verifying key
    pub fn verifying_key(&self) -> &VerifyingKey<Bls12> {
        &self.params.vk
    }

    /// Get policy root
    pub fn policy_root(&self) -> [u8; 32] {
        self.policy_root
    }
}

#[cfg(not(feature = "compact-zk"))]
impl Groth16Params {
    /// Setup stub
    pub fn setup(policy_root: [u8; 32]) -> Result<Self> {
        Ok(Self { policy_root })
    }

    /// Get policy root
    pub fn policy_root(&self) -> [u8; 32] {
        self.policy_root
    }
}

/// Groth16 lineage prover for compact ZK proofs
#[cfg(feature = "compact-zk")]
pub struct Groth16LineageProver<'a> {
    params: &'a Groth16Params,
    witnesses: Vec<StoredWitness>,
    genesis_lineage: [u8; 32],
    initial_counters: [u8; 32],
    genesis_lineage_fr: Fr,
    initial_counters_fr: Fr,
    initialized: bool,
    proving_start: Option<Instant>,
}

#[cfg(feature = "compact-zk")]
#[derive(Clone)]
struct StoredWitness {
    prev_state: Fr,
    new_state: Fr,
    origin: Fr,
    timestamp: Fr,
    epoch_id: Fr,
}

#[cfg(not(feature = "compact-zk"))]
/// This struct manages the state required to incrementally
/// build a Groth16 proof across multiple steps.
pub struct Groth16LineageProver<'a> {
    _params: &'a Groth16Params,
    initialized: bool,
    step_count: usize,
}

#[cfg(feature = "compact-zk")]
impl<'a> Groth16LineageProver<'a> {
    /// Create a new Groth16 prover
    pub fn new(params: &'a Groth16Params) -> Self {
        Self {
            params,
            witnesses: Vec::new(),
            genesis_lineage: [0u8; 32],
            initial_counters: [0u8; 32],
            genesis_lineage_fr: Fr::zero(),
            initial_counters_fr: Fr::zero(),
            initialized: false,
            proving_start: None,
        }
    }

    /// Initialize the prover
    pub fn initialize(
        &mut self,
        genesis_lineage: [u8; 32],
        initial_counters: [u8; 32],
    ) -> Result<()> {
        self.genesis_lineage = genesis_lineage;
        self.initial_counters = initial_counters;
        self.genesis_lineage_fr = bytes_to_fr(&genesis_lineage);
        self.initial_counters_fr = bytes_to_fr(&initial_counters);
        self.witnesses.clear();
        self.initialized = true;
        self.proving_start = Some(Instant::now());

        println!("Groth16 prover initialized");
        println!("  Genesis lineage (Fr): {:?}", self.genesis_lineage_fr);
        println!("  Initial counters (Fr): {:?}", self.initial_counters_fr);
        Ok(())
    }

    /// Add a transition (stores witness, doesn't prove yet)
    pub fn prove_step(&mut self, witness: &StepWitness) -> Result<()> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized(
                "Groth16 prover not initialized".into(),
            ));
        }

        if self.witnesses.len() >= MAX_TRANSITIONS {
            return Err(ZkOriginError::InvalidLineage(format!(
                "Maximum {} transitions supported in compact mode",
                MAX_TRANSITIONS
            )));
        }

        let stored = StoredWitness {
            prev_state: bytes_to_fr(&witness.prev_state_hash),
            new_state: bytes_to_fr(&witness.new_state_hash),
            origin: Fr::from(witness.new_origin as u64),
            timestamp: Fr::from(witness.timestamp),
            epoch_id: Fr::from(witness.epoch_id),
        };

        self.witnesses.push(stored);
        println!(
            "  Step {} stored (will prove at finalization)",
            self.witnesses.len()
        );

        Ok(())
    }

    /// Compute final state by replaying transitions (must match circuit exactly)
    fn compute_final_state(&self) -> (Fr, Fr) {
        let mut lineage = self.genesis_lineage_fr;
        let mut counters = self.initial_counters_fr;

        for w in &self.witnesses {
            // This MUST match the circuit computation exactly
            let state_product = w.prev_state * w.new_state;
            let transition_hash = state_product + w.origin + w.timestamp;
            let lineage_product = lineage * transition_hash;
            lineage = lineage_product + lineage + transition_hash;
            counters = counters + w.origin + w.epoch_id;
        }

        (lineage, counters)
    }

    /// Generate the compact proof
    pub fn finalize(&self) -> Result<LineageProof> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized("Not initialized".into()));
        }
        if self.witnesses.is_empty() {
            return Err(ZkOriginError::InvalidLineage("No steps to prove".into()));
        }

        println!(
            "Generating Groth16 proof ({} transitions)...",
            self.witnesses.len()
        );
        let start = Instant::now();

        let (final_lineage, final_counters) = self.compute_final_state();
        
        println!("  Genesis lineage: {:?}", self.genesis_lineage_fr);
        println!("  Genesis counters: {:?}", self.initial_counters_fr);
        println!("  Final lineage: {:?}", final_lineage);
        println!("  Final counters: {:?}", final_counters);

        // Build transition witnesses for REAL transitions
        let mut transitions: Vec<TransitionWitness> = self
            .witnesses
            .iter()
            .map(|w| TransitionWitness {
                prev_state: Some(w.prev_state),
                new_state: Some(w.new_state),
                origin: Some(w.origin),
                timestamp: Some(w.timestamp),
                epoch_id: Some(w.epoch_id),
                is_real: true,
            })
            .collect();

        // Pad with IDENTITY transitions that don't change the state
        // For identity: new_lineage = old_lineage, new_counters = old_counters
        // This requires: lineage_product + lineage + transition_hash = lineage
        // Which means: lineage_product + transition_hash = 0
        // If transition_hash = 0 and lineage = 0, this works
        // But if lineage != 0, we need transition_hash = 0 and lineage_product = 0
        // transition_hash = state_product + origin + timestamp = 0
        // If all are zero, transition_hash = 0
        // lineage_product = lineage * transition_hash = lineage * 0 = 0
        // new_lineage = 0 + lineage + 0 = lineage ✓
        // new_counters = counters + 0 + 0 = counters ✓
        
        while transitions.len() < MAX_TRANSITIONS {
            transitions.push(TransitionWitness {
                prev_state: Some(Fr::zero()),
                new_state: Some(Fr::zero()),
                origin: Some(Fr::zero()),
                timestamp: Some(Fr::zero()),
                epoch_id: Some(Fr::zero()),
                is_real: false,
            });
        }

        let circuit = CompactLineageCircuit::new(
            self.genesis_lineage_fr,
            self.initial_counters_fr,
            final_lineage,
            final_counters,
            transitions,
        );

        // Generate proof
        let proof = create_random_proof(circuit, &self.params.params, &mut OsRng)
            .map_err(|e| ZkOriginError::proving(format!("Groth16 proving failed: {:?}", e)))?;

        let prove_time = start.elapsed();
        println!("  Proof generated in {:?}", prove_time);

        // Serialize proof (~192 bytes for Groth16)
        let proof_bytes = serialize_groth16_proof(&proof)?;

        // Serialize verifying key manually
        let vk_bytes = serialize_verifying_key(&self.params.params.vk)?;

        println!(
            "  Proof size: {} bytes ({} < 1KB!)",
            proof_bytes.len(),
            if proof_bytes.len() < 1024 { "✓" } else { "✗" }
        );

        let proving_time_ms = self
            .proving_start
            .map(|s| s.elapsed().as_millis() as u64)
            .unwrap_or(0);

        // Store the Fr values as bytes for verification
        let final_lineage_bytes = fr_to_bytes(&final_lineage);
        let final_counters_bytes = fr_to_bytes(&final_counters);

        let mut lineage_proof = LineageProof::new_with_initial_state(
            proof_bytes,
            LineageCommitment::new(final_lineage_bytes, self.witnesses.len() as u64),
            CounterCommitment::new(final_counters_bytes, 0),
            LineageCommitment::new(self.genesis_lineage, 0),
            self.witnesses.len() as u64,
            self.witnesses.len() as u64,
            self.params.policy_root,
            self.initial_counters,
        );

        lineage_proof.metadata = ProofMetadata::new().with_proving_time(proving_time_ms);
        lineage_proof.verifier_key = Some(vk_bytes);

        Ok(lineage_proof)
    }

    /// Get step count
    pub fn step_count(&self) -> usize {
        self.witnesses.len()
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[cfg(not(feature = "compact-zk"))]
impl<'a> Groth16LineageProver<'a> {
    /// Create stub
    pub fn new(params: &'a Groth16Params) -> Self {
        Self {
            _params: params,
            initialized: false,
            step_count: 0,
        }
    }

    /// Initialize stub
    pub fn initialize(&mut self, _: [u8; 32], _: [u8; 32]) -> Result<()> {
        self.initialized = true;
        self.step_count = 0;
        Ok(())
    }

    /// Prove step stub
    pub fn prove_step(&mut self, _: &StepWitness) -> Result<()> {
        if !self.initialized {
            return Err(ZkOriginError::NotInitialized("".into()));
        }
        self.step_count += 1;
        Ok(())
    }

    /// Finalize stub
    pub fn finalize(&self) -> Result<crate::types::proof::LineageProof> {
        Err(ZkOriginError::InternalError(
            "Compact ZK not enabled".into(),
        ))
    }

    /// Get step count
    pub fn step_count(&self) -> usize {
        self.step_count
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

// Helper functions
#[cfg(feature = "compact-zk")]
fn bytes_to_fr(bytes: &[u8; 32]) -> Fr {
    // BLS12-381 scalar field is ~255 bits, so we can use 31 bytes safely
    let mut repr = [0u8; 32];
    repr[..31].copy_from_slice(&bytes[..31]);
    Fr::from_bytes(&repr).unwrap_or(Fr::zero())
}

#[cfg(feature = "compact-zk")]
fn fr_to_bytes(fr: &Fr) -> [u8; 32] {
    fr.to_bytes()
}

#[cfg(feature = "compact-zk")]
fn serialize_groth16_proof(proof: &Proof<Bls12>) -> Result<Vec<u8>> {
    // Groth16 proof: A (G1) + B (G2) + C (G1)
    // Compressed: 48 + 96 + 48 = 192 bytes
    let mut bytes = Vec::with_capacity(192);

    let a_affine: G1Affine = proof.a.into();
    let b_affine: G2Affine = proof.b.into();
    let c_affine: G1Affine = proof.c.into();

    bytes.extend_from_slice(&a_affine.to_compressed());
    bytes.extend_from_slice(&b_affine.to_compressed());
    bytes.extend_from_slice(&c_affine.to_compressed());

    Ok(bytes)
}

/// Serialize verifying key to bytes
#[cfg(feature = "compact-zk")]
fn serialize_verifying_key(vk: &VerifyingKey<Bls12>) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();

    // Serialize alpha_g1 (G1 point - 48 bytes compressed)
    let alpha: G1Affine = vk.alpha_g1.into();
    bytes.extend_from_slice(&alpha.to_compressed());

    // Serialize beta_g1 (G1 point - 48 bytes compressed)
    let beta_g1: G1Affine = vk.beta_g1.into();
    bytes.extend_from_slice(&beta_g1.to_compressed());

    // Serialize beta_g2 (G2 point - 96 bytes compressed)
    let beta_g2: G2Affine = vk.beta_g2.into();
    bytes.extend_from_slice(&beta_g2.to_compressed());

    // Serialize gamma_g2 (G2 point - 96 bytes compressed)
    let gamma: G2Affine = vk.gamma_g2.into();
    bytes.extend_from_slice(&gamma.to_compressed());

    // Serialize delta_g1 (G1 point - 48 bytes compressed)
    let delta_g1: G1Affine = vk.delta_g1.into();
    bytes.extend_from_slice(&delta_g1.to_compressed());

    // Serialize delta_g2 (G2 point - 96 bytes compressed)
    let delta_g2: G2Affine = vk.delta_g2.into();
    bytes.extend_from_slice(&delta_g2.to_compressed());

    // Serialize ic (Vec<G1> - each 48 bytes)
    // First write count as 4 bytes
    let ic_count = vk.ic.len() as u32;
    bytes.extend_from_slice(&ic_count.to_le_bytes());

    for ic_point in &vk.ic {
        let ic_affine: G1Affine = (*ic_point).into();
        bytes.extend_from_slice(&ic_affine.to_compressed());
    }

    Ok(bytes)
}

/// Deserialize verifying key from bytes
#[cfg(feature = "compact-zk")]
fn deserialize_verifying_key(bytes: &[u8]) -> Result<VerifyingKey<Bls12>> {
    // Header: alpha_g1(48) + beta_g1(48) + beta_g2(96) + gamma_g2(96) + delta_g1(48) + delta_g2(96) + count(4) = 436 bytes minimum
    if bytes.len() < 436 {
        return Err(ZkOriginError::InvalidProof(format!(
            "VK too short: {} bytes",
            bytes.len()
        )));
    }

    let mut offset = 0;

    // Deserialize alpha_g1
    let alpha_bytes: [u8; 48] = bytes[offset..offset + 48].try_into().unwrap();
    let alpha = G1Affine::from_compressed(&alpha_bytes);
    if alpha.is_none().into() {
        return Err(ZkOriginError::InvalidProof("Invalid alpha_g1".into()));
    }
    offset += 48;

    // Deserialize beta_g1
    let beta_g1_bytes: [u8; 48] = bytes[offset..offset + 48].try_into().unwrap();
    let beta_g1 = G1Affine::from_compressed(&beta_g1_bytes);
    if beta_g1.is_none().into() {
        return Err(ZkOriginError::InvalidProof("Invalid beta_g1".into()));
    }
    offset += 48;

    // Deserialize beta_g2
    let beta_g2_bytes: [u8; 96] = bytes[offset..offset + 96].try_into().unwrap();
    let beta_g2 = G2Affine::from_compressed(&beta_g2_bytes);
    if beta_g2.is_none().into() {
        return Err(ZkOriginError::InvalidProof("Invalid beta_g2".into()));
    }
    offset += 96;

    // Deserialize gamma_g2
    let gamma_bytes: [u8; 96] = bytes[offset..offset + 96].try_into().unwrap();
    let gamma = G2Affine::from_compressed(&gamma_bytes);
    if gamma.is_none().into() {
        return Err(ZkOriginError::InvalidProof("Invalid gamma_g2".into()));
    }
    offset += 96;

    // Deserialize delta_g1
    let delta_g1_bytes: [u8; 48] = bytes[offset..offset + 48].try_into().unwrap();
    let delta_g1 = G1Affine::from_compressed(&delta_g1_bytes);
    if delta_g1.is_none().into() {
        return Err(ZkOriginError::InvalidProof("Invalid delta_g1".into()));
    }
    offset += 48;

    // Deserialize delta_g2
    let delta_g2_bytes: [u8; 96] = bytes[offset..offset + 96].try_into().unwrap();
    let delta_g2 = G2Affine::from_compressed(&delta_g2_bytes);
    if delta_g2.is_none().into() {
        return Err(ZkOriginError::InvalidProof("Invalid delta_g2".into()));
    }
    offset += 96;

    // Deserialize ic count
    let ic_count = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    // Deserialize ic points
    let mut ic = Vec::with_capacity(ic_count);
    for _ in 0..ic_count {
        if offset + 48 > bytes.len() {
            return Err(ZkOriginError::InvalidProof("VK truncated".into()));
        }
        let ic_bytes: [u8; 48] = bytes[offset..offset + 48].try_into().unwrap();
        let ic_point = G1Affine::from_compressed(&ic_bytes);
        if ic_point.is_none().into() {
            return Err(ZkOriginError::InvalidProof("Invalid ic point".into()));
        }
        // Store as G1Affine
        ic.push(ic_point.unwrap().into());
        offset += 48;
    }

    Ok(VerifyingKey {
        alpha_g1: alpha.unwrap().into(),
        beta_g1: beta_g1.unwrap().into(),
        beta_g2: beta_g2.unwrap().into(),
        gamma_g2: gamma.unwrap().into(),
        delta_g1: delta_g1.unwrap().into(),
        delta_g2: delta_g2.unwrap().into(),
        ic,
    })
}

/// Verify a Groth16 proof
#[cfg(feature = "compact-zk")]
pub fn verify_groth16_proof(
    proof_bytes: &[u8],
    vk_bytes: &[u8],
    genesis_lineage: &[u8; 32],
    genesis_counters: &[u8; 32],
    final_lineage: &[u8; 32],
    final_counters: &[u8; 32],
) -> Result<bool> {
    println!("═ Compact ZK Proof Verification (Groth16)");
    let start = Instant::now();

    // Deserialize proof
    if proof_bytes.len() != 192 {
        return Err(ZkOriginError::InvalidProof(format!(
            "Invalid proof size: {} (expected 192)",
            proof_bytes.len()
        )));
    }

    let a_bytes: [u8; 48] = proof_bytes[0..48].try_into().unwrap();
    let b_bytes: [u8; 96] = proof_bytes[48..144].try_into().unwrap();
    let c_bytes: [u8; 48] = proof_bytes[144..192].try_into().unwrap();

    let a = G1Affine::from_compressed(&a_bytes);
    let b = G2Affine::from_compressed(&b_bytes);
    let c = G1Affine::from_compressed(&c_bytes);

    if a.is_none().into() || b.is_none().into() || c.is_none().into() {
        return Err(ZkOriginError::InvalidProof(
            "Failed to deserialize proof points".into(),
        ));
    }

    let proof = Proof {
        a: a.unwrap().into(),
        b: b.unwrap().into(),
        c: c.unwrap().into(),
    };

    // Deserialize verifying key
    println!("  Deserializing verifying key ({} bytes)...", vk_bytes.len());
    let vk = deserialize_verifying_key(vk_bytes)?;
    let pvk = prepare_verifying_key(&vk);

    // Convert inputs to field elements
    let genesis_lineage_fr = bytes_to_fr(genesis_lineage);
    let genesis_counters_fr = bytes_to_fr(genesis_counters);
    let final_lineage_fr = bytes_to_fr(final_lineage);
    let final_counters_fr = bytes_to_fr(final_counters);

    println!("  Public inputs:");
    println!("    genesis_lineage: {:?}", genesis_lineage_fr);
    println!("    genesis_counters: {:?}", genesis_counters_fr);
    println!("    final_lineage: {:?}", final_lineage_fr);
    println!("    final_counters: {:?}", final_counters_fr);

    // Prepare public inputs (must match circuit order)
    let public_inputs = vec![
        genesis_lineage_fr,
        genesis_counters_fr,
        final_lineage_fr,
        final_counters_fr,
    ];

    // Verify
    match verify_proof(&pvk, &proof, &public_inputs) {
        Ok(_) => {
            println!("  ✓ ZK Verification PASSED in {:?}", start.elapsed());
            Ok(true)
        }
        Err(e) => {
            println!("  ✗ Verification failed: {:?}", e);
            Err(ZkOriginError::VerificationFailed(format!(
                "Groth16 verification failed: {:?}",
                e
            )))
        }
    }
}

/// Stub for non-compact-zk builds
#[cfg(not(feature = "compact-zk"))]
pub fn verify_groth16_proof(
    _proof_bytes: &[u8],
    _vk_bytes: &[u8],
    _genesis_lineage: &[u8; 32],
    _genesis_counters: &[u8; 32],
    _final_lineage: &[u8; 32],
    _final_counters: &[u8; 32],
) -> Result<bool> {
    Err(ZkOriginError::InternalError(
        "Compact ZK not enabled".into(),
    ))
}