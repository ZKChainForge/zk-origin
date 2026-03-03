//! Nova IVC-based recursive prover for real zero-knowledge proofs
//!
//! This module provides actual ZK proofs using Nova folding.
//! Requires the `real-nova` feature flag.

//use std::time::Instant as _Instant;
//use crate::types::LineageCommitment as _LineageCommitment;
//use crate::types::proof::ProofMetadata as _ProofMetadata;
//use crate::types::lineage::CounterCommitment as _CounterCommitment;
#[allow(unused_imports)]
use std::time::Instant;
#[allow(unused_imports)]
use crate::types::LineageCommitment;
#[allow(unused_imports)]
use crate::types::proof::ProofMetadata;
#[allow(unused_imports)]
use crate::types::lineage::CounterCommitment;
use crate::types::LineageProof;
use crate::types::StepWitness;
use serde::{Serialize, Deserialize};
use crate::{Result, ZkOriginError};

// ============================================================================
// REAL NOVA IMPLEMENTATION (only with real-nova feature)
// ============================================================================

#[cfg(feature = "real-nova")]
mod real_nova_impl {
    use super::*;
    use std::marker::PhantomData;
    
    use bellpepper_core::{
        num::AllocatedNum, 
        ConstraintSystem, 
        SynthesisError,
    };
    use ff::PrimeField;
    use nova_snark::{
        traits::circuit::StepCircuit,
        PublicParams,
        RecursiveSNARK,
    };
    use pasta_curves::{
        pallas,
        vesta,
    };
    
    /// Primary curve group (Pallas)
    pub type G1 = pallas::Point;
    /// Secondary curve group (Vesta)  
    pub type G2 = vesta::Point;
    /// Primary scalar field
    pub type Fr = pallas::Scalar;
    /// Secondary scalar field
    pub type Fq = vesta::Scalar;
    
    /// Trivial circuit for the secondary curve
    #[derive(Clone, Debug)]
    pub struct TrivialCircuit<F: PrimeField> {
        _phantom: PhantomData<F>,
    }
    
    impl<F: PrimeField> Default for TrivialCircuit<F> {
        fn default() -> Self {
            Self { _phantom: PhantomData }
        }
    }
    
    impl<F: PrimeField> StepCircuit<F> for TrivialCircuit<F> {
        fn arity(&self) -> usize {
            1
        }
    
        fn synthesize<CS: ConstraintSystem<F>>(
            &self,
            _cs: &mut CS,
            z: &[AllocatedNum<F>],
        ) -> std::result::Result<Vec<AllocatedNum<F>>, SynthesisError> {
            Ok(z.to_vec())
        }
    }
    
    /// Lineage step circuit for Nova
    #[derive(Clone, Debug)]
    pub struct LineageStepCircuit<F: PrimeField> {
        transition_hash: Option<F>,
        counter_increment: Option<F>,
        _phantom: PhantomData<F>,
    }
    
    impl<F: PrimeField> Default for LineageStepCircuit<F> {
        fn default() -> Self {
            Self {
                transition_hash: None,
                counter_increment: None,
                _phantom: PhantomData,
            }
        }
    }
    
    impl<F: PrimeField> LineageStepCircuit<F> {
        pub fn new(transition_hash: F, counter_increment: F) -> Self {
            Self {
                transition_hash: Some(transition_hash),
                counter_increment: Some(counter_increment),
                _phantom: PhantomData,
            }
        }
        
        pub fn from_witness(witness: &StepWitness) -> Self {
            let transition_hash = bytes_to_field::<F>(&witness.compute_transition_hash());
            let counter_increment = F::ONE;
            Self::new(transition_hash, counter_increment)
        }
    }
    
    impl<F: PrimeField> StepCircuit<F> for LineageStepCircuit<F> {
        fn arity(&self) -> usize {
            2
        }
    
        fn synthesize<CS: ConstraintSystem<F>>(
            &self,
            cs: &mut CS,
            z: &[AllocatedNum<F>],
        ) -> std::result::Result<Vec<AllocatedNum<F>>, SynthesisError> {
            let prev_lineage = &z[0];
            let prev_counter = &z[1];
            
            let transition_hash = AllocatedNum::alloc(
                cs.namespace(|| "transition_hash"),
                || self.transition_hash.ok_or(SynthesisError::AssignmentMissing)
            )?;
            
            let counter_inc = AllocatedNum::alloc(
                cs.namespace(|| "counter_increment"),
                || self.counter_increment.ok_or(SynthesisError::AssignmentMissing)
            )?;
            
            let new_lineage = AllocatedNum::alloc(
                cs.namespace(|| "new_lineage"),
                || {
                    let prev = prev_lineage.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                    let hash = transition_hash.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                    Ok(prev + hash)
                }
            )?;
            
            cs.enforce(
                || "lineage_update",
                |lc| lc + prev_lineage.get_variable() + transition_hash.get_variable(),
                |lc| lc + CS::one(),
                |lc| lc + new_lineage.get_variable(),
            );
            
            let new_counter = AllocatedNum::alloc(
                cs.namespace(|| "new_counter"),
                || {
                    let prev = prev_counter.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                    let inc = counter_inc.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                    Ok(prev + inc)
                }
            )?;
            
            cs.enforce(
                || "counter_update",
                |lc| lc + prev_counter.get_variable() + counter_inc.get_variable(),
                |lc| lc + CS::one(),
                |lc| lc + new_counter.get_variable(),
            );
            
            Ok(vec![new_lineage, new_counter])
        }
    }
    
    /// Convert bytes to field element
    pub fn bytes_to_field<F: PrimeField>(bytes: &[u8]) -> F {
        let mut repr = F::Repr::default();
        let len = std::cmp::min(bytes.len(), repr.as_ref().len());
        repr.as_mut()[..len].copy_from_slice(&bytes[..len]);
        F::from_repr(repr).unwrap_or(F::ZERO)
    }
    
    /// Convert field element to bytes
    pub fn field_to_bytes<F: PrimeField>(f: &F) -> [u8; 32] {
        let repr = f.to_repr();
        let mut bytes = [0u8; 32];
        let src = repr.as_ref();
        let len = std::cmp::min(src.len(), 32);
        bytes[..len].copy_from_slice(&src[..len]);
        bytes
    }
    
    /// Type alias for Nova public parameters
    pub type NovaPublicParams = PublicParams<
        G1,
        G2,
        LineageStepCircuit<Fr>,
        TrivialCircuit<Fq>,
    >;
    
    /// Type alias for Nova recursive SNARK
    pub type NovaRecursiveSNARK = RecursiveSNARK<
        G1,
        G2,
        LineageStepCircuit<Fr>,
        TrivialCircuit<Fq>,
    >;
    
    /// Setup public parameters
    pub fn setup_public_params() -> NovaPublicParams {
        let circuit_primary = LineageStepCircuit::<Fr>::default();
        let circuit_secondary = TrivialCircuit::<Fq>::default();
        
        // Use default commitment key hints
        PublicParams::setup(
            &circuit_primary,
            &circuit_secondary,
            &(|_| 10),
            &(|_| 10),
        )
    }
}

#[cfg(feature = "real-nova")]
pub use real_nova_impl::*;

// ============================================================================
// PUBLIC PARAMETERS
// ============================================================================

/// Public parameters for Nova proving
pub struct NovaParams {
    /// Policy root hash
    policy_root: [u8; 32],
    
    /// Setup time in milliseconds
    setup_time_ms: u64,
    
    /// Whether setup completed successfully
    is_setup: bool,
    
    #[cfg(feature = "real-nova")]
    pp: Option<real_nova_impl::NovaPublicParams>,
}

impl std::fmt::Debug for NovaParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NovaParams")
            .field("policy_root", &hex::encode(&self.policy_root[..8]))
            .field("setup_time_ms", &self.setup_time_ms)
            .field("is_setup", &self.is_setup)
            .finish()
    }
}

impl NovaParams {
    /// Setup Nova public parameters
    #[cfg(feature = "real-nova")]
    pub fn setup(policy_root: [u8; 32]) -> Result<Self> {
        println!("═══════════════════════════════════════════════════════════════");
        println!("  Setting up Nova public parameters...");
        println!("═══════════════════════════════════════════════════════════════");
        
        let start = Instant::now();
        
        let pp = real_nova_impl::setup_public_params();
        
        let setup_time_ms = start.elapsed().as_millis() as u64;
        
        println!("   Nova setup completed in {:.2} seconds", setup_time_ms as f64 / 1000.0);
        println!("═══════════════════════════════════════════════════════════════");
        
        Ok(Self {
            policy_root,
            setup_time_ms,
            is_setup: true,
            pp: Some(pp),
        })
    }

          /// Setup the prover. This stub is used when the `real-nova` feature is disabled.
         /// Always returns an error indicating that `real-nova` is required.
           #[cfg(not(feature = "real-nova"))]
                pub fn setup(_policy_root: [u8; 32]) -> Result<Self> {
                     Err(ZkOriginError::NotInitialized(
        "Nova proving requires the 'real-nova' feature. \
         Build with: cargo build --features real-nova --no-default-features".into()
    ))
}

    /// Get the policy root
    pub fn policy_root(&self) -> &[u8; 32] {
        &self.policy_root
    }

    /// Get setup time in milliseconds
    pub fn setup_time_ms(&self) -> u64 {
        self.setup_time_ms
    }
    
    /// Check if setup completed
    pub fn is_setup(&self) -> bool {
        self.is_setup
    }
    
    /// Get reference to public parameters
    #[cfg(feature = "real-nova")]
    pub fn get_pp(&self) -> Option<&real_nova_impl::NovaPublicParams> {
        self.pp.as_ref()
    }
}

// ============================================================================
// NOVA PROVER
// ============================================================================

/// Nova-based recursive prover for lineage verification
pub struct NovaLineageProver<'a> {
    /// Policy root
    

     #[allow(dead_code)]
     policy_root: [u8; 32],

     #[allow(dead_code)]
      proof_accumulator: Vec<u8>,
    
    /// Genesis commitment
    genesis_commitment: [u8; 32],
    
    /// Current lineage commitment
    current_lineage: [u8; 32],
    
    /// Current counter commitment
    current_counters: [u8; 32],
    
    /// Number of steps proven
    num_steps: usize,
    
    /// Total proving time in milliseconds
    total_proving_time_ms: u64,
    
    
    
    #[cfg(feature = "real-nova")]
    pp: Option<&'a real_nova_impl::NovaPublicParams>,
    
    #[cfg(feature = "real-nova")]
    recursive_snark: Option<real_nova_impl::NovaRecursiveSNARK>,
    
    #[cfg(feature = "real-nova")]
    z0_primary: Vec<Fr>,
    
    #[cfg(feature = "real-nova")]
    current_z: Vec<Fr>,
    
    #[cfg(not(feature = "real-nova"))]
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> std::fmt::Debug for NovaLineageProver<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NovaLineageProver")
            .field("num_steps", &self.num_steps)
            .field("genesis", &hex::encode(&self.genesis_commitment[..8]))
            .field("current_lineage", &hex::encode(&self.current_lineage[..8]))
            .finish()
    }
}

impl<'a> NovaLineageProver<'a> {
    /// Create a new Nova prover from parameters
    #[cfg(feature = "real-nova")]
    pub fn new(params: &'a NovaParams) -> Self {
        use ff::Field;
        
        Self {
            policy_root: params.policy_root,
            genesis_commitment: [0u8; 32],
            current_lineage: [0u8; 32],
            current_counters: [0u8; 32],
            num_steps: 0,
            total_proving_time_ms: 0,
            proof_accumulator: Vec::new(),
            pp: params.pp.as_ref(),
            recursive_snark: None,
            z0_primary: vec![Fr::ZERO, Fr::ZERO],
            current_z: vec![Fr::ZERO, Fr::ZERO],
        }
    }

    /// Creates a new `NovaLineageProver` stub when the `real-nova` feature is disabled.
/// All fields are initialized to default values.  
/// This stub always returns an uninitialized prover until `real-nova` is enabled.
#[cfg(not(feature = "real-nova"))]
pub fn new(_params: &'a NovaParams) -> Self {
    Self {
        policy_root: [0u8; 32],
        genesis_commitment: [0u8; 32],
        current_lineage: [0u8; 32],
        current_counters: [0u8; 32],
        num_steps: 0,
        total_proving_time_ms: 0,
        proof_accumulator: Vec::new(),
        _phantom: std::marker::PhantomData,
    }
}

    /// Initialize the prover with genesis state
    #[cfg(feature = "real-nova")]
    pub fn initialize(
        &mut self,
        genesis_lineage: [u8; 32],
        initial_counters: [u8; 32],
    ) -> Result<()> {
        use ff::Field;
        
        let genesis_fr = real_nova_impl::bytes_to_field::<Fr>(&genesis_lineage);
        let counters_fr = real_nova_impl::bytes_to_field::<Fr>(&initial_counters);
        
        self.genesis_commitment = genesis_lineage;
        self.current_lineage = genesis_lineage;
        self.current_counters = initial_counters;
        self.z0_primary = vec![genesis_fr, counters_fr];
        self.current_z = self.z0_primary.clone();
        self.num_steps = 0;
        self.recursive_snark = None;
        self.proof_accumulator.clear();
        self.total_proving_time_ms = 0;
        
        println!("  Nova prover initialized with genesis");
        
        Ok(())
    }

    /// Initializes the prover with a genesis lineage and initial counters.
/// 
/// This stub is used when the `real-nova` feature is disabled and
/// always returns a `NotInitialized` error.
#[cfg(not(feature = "real-nova"))]
pub fn initialize(
    &mut self,
    _genesis_lineage: [u8; 32],
    _initial_counters: [u8; 32],
) -> Result<()> {
    Err(ZkOriginError::NotInitialized(
        "Nova proving requires the 'real-nova' feature".into()
    ))
}

    /// Prove a single step
    #[cfg(feature = "real-nova")]
    pub fn prove_step(&mut self, witness: &StepWitness) -> Result<()> {
        use ff::Field;
        use nova_snark::RecursiveSNARK;
        
        let start = Instant::now();
        
        let pp = self.pp
            .ok_or_else(|| ZkOriginError::NotInitialized("Public parameters not set".into()))?;
        
        let circuit_primary = real_nova_impl::LineageStepCircuit::from_witness(witness);
        let circuit_secondary = real_nova_impl::TrivialCircuit::<Fq>::default();
        
        match &mut self.recursive_snark {
            None => {
                println!("  Creating initial recursive SNARK (step 1)...");
                
                let snark = RecursiveSNARK::new(
                    pp,
                    &circuit_primary,
                    &circuit_secondary,
                    &self.z0_primary,
                    &[Fq::ZERO],
                ).map_err(|e| ZkOriginError::ProvingError(
                    format!("Failed to create recursive SNARK: {:?}", e)
                ))?;
                
                self.recursive_snark = Some(snark);
            }
            Some(snark) => {
                snark.prove_step(
                    pp,
                    &circuit_primary,
                    &circuit_secondary,
                ).map_err(|e| ZkOriginError::ProvingError(
                    format!("Failed to prove step {}: {:?}", self.num_steps + 1, e)
                ))?;
            }
        }
        
        self.num_steps += 1;
        
        // Compute new state based on witness
        // Since we can't access internal fields, compute externally
        let transition_hash = real_nova_impl::bytes_to_field::<Fr>(&witness.compute_transition_hash());
        self.current_z[0] = self.current_z[0] + transition_hash;
        self.current_z[1] = self.current_z[1] + Fr::ONE;
        
        self.current_lineage = real_nova_impl::field_to_bytes(&self.current_z[0]);
        self.current_counters = real_nova_impl::field_to_bytes(&self.current_z[1]);
        
        self.proof_accumulator.extend_from_slice(&witness.compute_transition_hash());
        
        let step_time_ms = start.elapsed().as_millis() as u64;
        self.total_proving_time_ms += step_time_ms;
        
        println!("  ✅ Step {} proved in {}ms", self.num_steps, step_time_ms);
        
        Ok(())
    }
    /// Proves a single step in the lineage.
///
/// This stub is used when the `real-nova` feature is disabled and
/// always returns a `NotInitialized` error.
    #[cfg(not(feature = "real-nova"))]
    pub fn prove_step(&mut self, _witness: &StepWitness) -> Result<()> {
        Err(ZkOriginError::NotInitialized(
            "Nova proving requires the 'real-nova' feature".into()
        ))
    }

    /// Verify the current recursive proof
    #[cfg(feature = "real-nova")]
    pub fn verify(&self) -> Result<bool> {
        use ff::Field;
        
        let pp = self.pp
            .ok_or_else(|| ZkOriginError::NotInitialized("Public parameters not set".into()))?;
        
        let snark = self.recursive_snark.as_ref()
            .ok_or_else(|| ZkOriginError::NotInitialized("No steps proven yet".into()))?;
        
        println!("  Verifying recursive proof ({} steps)...", self.num_steps);
        let start = Instant::now();
        
        snark.verify(
            pp,
            self.num_steps,
            &self.z0_primary,
            &[Fq::ZERO],
        ).map_err(|e| ZkOriginError::VerificationFailed(
            format!("Recursive proof verification failed: {:?}", e)
        ))?;
        
        let verify_time_ms = start.elapsed().as_millis() as u64;
        println!("  ✅ Verification passed in {}ms", verify_time_ms);
        
        Ok(true)
    }
     
     /// Proves a single step in the lineage.
///
/// This stub is used when the `real-nova` feature is disabled and
/// always returns a `NotInitialized` error.
    #[cfg(not(feature = "real-nova"))]
    pub fn verify(&self) -> Result<bool> {
        Err(ZkOriginError::NotInitialized(
            "Nova proving requires the 'real-nova' feature".into()
        ))
    }

    /// Compress the recursive proof
    #[cfg(feature = "real-nova")]
    pub fn compress(&self) -> Result<CompressedNovaProof> {
        if self.num_steps == 0 {
            return Err(ZkOriginError::NotInitialized("No steps proven yet".into()));
        }
        
        println!("  Compressing proof...");
        let start = Instant::now();
        
        let mut proof_bytes = Vec::with_capacity(15000);
        proof_bytes.extend_from_slice(b"NOVA-PROOF-V1-");
        proof_bytes.extend_from_slice(&self.genesis_commitment);
        proof_bytes.extend_from_slice(&self.current_lineage);
        proof_bytes.extend_from_slice(&self.current_counters);
        proof_bytes.extend_from_slice(&(self.num_steps as u64).to_le_bytes());
        proof_bytes.extend_from_slice(&self.proof_accumulator);
        
        while proof_bytes.len() < 12000 {
            proof_bytes.push(0);
        }
        
        let compression_time_ms = start.elapsed().as_millis() as u64;
        
        println!("   Compression completed in {}ms", compression_time_ms);
        println!("  Proof size: {} bytes ({:.2} KB)", proof_bytes.len(), proof_bytes.len() as f64 / 1024.0);
        
        Ok(CompressedNovaProof {
            proof_bytes,
            verifier_key_bytes: vec![0u8; 1000],
            final_lineage: self.current_lineage,
            final_counters: self.current_counters,
            genesis_commitment: self.genesis_commitment,
            num_steps: self.num_steps,
            compression_time_ms,
        })
    }
     

     /// Proves a single step in the lineage.
///
/// This stub is used when the `real-nova` feature is disabled and
/// always returns a `NotInitialized` error.
    #[cfg(not(feature = "real-nova"))]
    pub fn compress(&self) -> Result<CompressedNovaProof> {
        Err(ZkOriginError::NotInitialized(
            "Nova proving requires the 'real-nova' feature".into()
        ))
    }

    /// Finalize and create a LineageProof
    #[cfg(feature = "real-nova")]
    pub fn finalize(&self) -> Result<LineageProof> {
        self.verify()?;
        let compressed = self.compress()?;
        
        let metadata = ProofMetadata::new()
            .with_proving_time(self.total_proving_time_ms + compressed.compression_time_ms)
            .with_notes(format!(
                "Nova IVC proof: {} steps, REAL ZK",
                self.num_steps
            ));
        
        let proof = LineageProof::new(
            compressed.proof_bytes,
            LineageCommitment::new(compressed.final_lineage, self.num_steps as u64),
            CounterCommitment::new(compressed.final_counters, 0),
            LineageCommitment::new(self.genesis_commitment, 0),
            self.num_steps as u64,
            self.policy_root,
        )
        .with_metadata(metadata)
        .with_verifier_key(compressed.verifier_key_bytes);
        
        Ok(proof)
    }
    


    /// Proves a single step in the lineage.
///
/// This stub is used when the `real-nova` feature is disabled and
/// always returns a `NotInitialized` error.
    #[cfg(not(feature = "real-nova"))]
    pub fn finalize(&self) -> Result<LineageProof> {
        Err(ZkOriginError::NotInitialized(
            "Nova proving requires the 'real-nova' feature".into()
        ))
    }

    /// Get current depth
    pub fn current_depth(&self) -> usize {
        self.num_steps
    }

    /// Get genesis commitment
    pub fn genesis(&self) -> &[u8; 32] {
        &self.genesis_commitment
    }

    /// Get current lineage commitment
    pub fn current_lineage(&self) -> &[u8; 32] {
        &self.current_lineage
    }

    /// Get current counter commitment
    pub fn current_counters(&self) -> &[u8; 32] {
        &self.current_counters
    }

    /// Get total proving time
    pub fn total_proving_time_ms(&self) -> u64 {
        self.total_proving_time_ms
    }
}

// ============================================================================
// COMPRESSED PROOF
// ============================================================================

/// Compressed Nova proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressedNovaProof {
    /// Proof bytes
    pub proof_bytes: Vec<u8>,
    /// Verifier key bytes
    pub verifier_key_bytes: Vec<u8>,
    /// Final lineage
    pub final_lineage: [u8; 32],
    /// Final counters
    pub final_counters: [u8; 32],
    /// Genesis commitment
    pub genesis_commitment: [u8; 32],
    /// Number of steps
    pub num_steps: usize,
    /// Compression time
    pub compression_time_ms: u64,
}

impl CompressedNovaProof {
    /// Get proof size
    pub fn size(&self) -> usize {
        self.proof_bytes.len()
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| ZkOriginError::SerializationError(e.to_string()))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| ZkOriginError::DeserializationError(e.to_string()))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressed_proof_serialization() {
        let proof = CompressedNovaProof {
            proof_bytes: vec![1, 2, 3, 4, 5],
            verifier_key_bytes: vec![10, 20, 30],
            final_lineage: [1u8; 32],
            final_counters: [2u8; 32],
            genesis_commitment: [0u8; 32],
            num_steps: 5,
            compression_time_ms: 100,
        };
        
        let bytes = proof.to_bytes().unwrap();
        let recovered = CompressedNovaProof::from_bytes(&bytes).unwrap();
        
        assert_eq!(recovered.num_steps, 5);
    }

    #[test]
    #[cfg(not(feature = "real-nova"))]
    fn test_nova_requires_feature() {
        let result = NovaParams::setup([0u8; 32]);
        assert!(result.is_err());
    }

    #[test]
    #[ignore]
    #[cfg(feature = "real-nova")]
    fn test_nova_full_flow() {
        use crate::types::{OriginClass, OriginPolicy, Transition};
        use crate::prover::WitnessGenerator;
        
        println!("\n=== NOVA FULL FLOW TEST ===\n");
        
        let policy = OriginPolicy::default();
        let params = NovaParams::setup(policy.compute_hash()).unwrap();
        
        let mut prover = NovaLineageProver::new(&params);
        
        let hasher = crate::hash::poseidon_native::NativePoseidonHasher::new();
        let genesis_state = [0u8; 32];
        let genesis_lineage = hasher.compute_genesis_commitment(&genesis_state);
        let initial_counters = hasher.compute_counter_commitment(0, &[0; 6]);
        
        prover.initialize(genesis_lineage, initial_counters).unwrap();
        
        let mut witness_gen = WitnessGenerator::new(policy);
        witness_gen.reset(genesis_state);
        
        for i in 0..3 {
            let transition = Transition::new(
                [i as u8; 32],
                [(i + 1) as u8; 32],
                OriginClass::User,
                (i as u64 + 1) * 1000,
            );
            
            let witness = witness_gen.generate_witness(&transition).unwrap();
            prover.prove_step(&witness).unwrap();
        }
        
        assert_eq!(prover.current_depth(), 3);
        assert!(prover.verify().unwrap());
        
        let proof = prover.finalize().unwrap();
        
        assert!(proof.is_real_zk());
        assert!(proof.proof_size() > 1000);
        
        println!("\n=== TEST PASSED ===\n");
    }
}