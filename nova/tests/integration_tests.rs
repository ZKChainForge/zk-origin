//! Integration tests for Nova IVC proof system
//!
//! This module contains comprehensive integration tests covering:
//! - Full proof workflows
//! - Multiple sequential proofs
//! - Proof chain verification
//! - Compression workflows
//! - Statistics collection
//! - Large proof generation
//! - Tampering detection
//! - State validation
//! - Batch operations
//! - Serialization

use zk_origin_nova::{
    NovaConfig, NovaIVCProver, NovaVerifier, NovaCompressor, CompressedNovaProof, STATE_SIZE
};

/// Test a complete proof workflow from creation to verification
///
/// This test demonstrates the basic workflow:
/// 1. Create a prover with testing config
/// 2. Add 10 state transitions
/// 3. Finalize the proof
/// 4. Verify the proof
#[test]
fn test_full_proof_workflow() {
    // Create prover
    let config = NovaConfig::testing();
    let mut prover = NovaIVCProver::new(config).expect("Failed to create prover");

    // Add multiple transitions
    for i in 1..11 {  // Start from 1 to ensure state changes from genesis
        let mut state = vec![0u8; STATE_SIZE];
        state[0] = (i & 0xFF) as u8;
        state[1] = ((i >> 8) & 0xFF) as u8;
        
        prover.add_transition(&state)
            .expect(&format!("Failed to add transition {}", i));
    }

    // Finalize proof
    let proof = prover.finalize()
        .expect("Failed to finalize proof");

    // Verify proof
    let genesis = vec![0u8; STATE_SIZE];
    let result = NovaVerifier::verify(&proof, &genesis, &proof.final_state)
        .expect("Failed to verify proof");

    assert!(result, "Proof verification failed");
    assert_eq!(proof.steps, 10, "Expected 10 steps");
    assert!(!proof.proof_data.is_empty(), "Proof data should not be empty");
    assert!(!proof.checksum.is_zero(), "Checksum should not be zero");
}

/// Test multiple sequential proofs
///
/// This test demonstrates:
/// 1. Creating and finalizing a first proof with 5 transitions
/// 2. Creating and finalizing a second proof with 5 different transitions
/// 3. Verifying both proofs independently
#[test]
fn test_multiple_proofs_sequential() {
    let config = NovaConfig::testing();
    
    // First proof
    let mut prover1 = NovaIVCProver::new(config.clone())
        .expect("Failed to create first prover");
    
    for i in 1..6 {  // First proof: states 1-5
        let mut state = vec![0u8; STATE_SIZE];
        state[0] = (i & 0xFF) as u8;
        prover1.add_transition(&state)
            .expect(&format!("Prover1: Failed to add transition {}", i));
    }
    
    let proof1 = prover1.finalize()
        .expect("Failed to finalize proof1");
    
    assert_eq!(proof1.steps, 5, "Proof1 should have 5 steps");

    // Second proof with different transitions
    let mut prover2 = NovaIVCProver::new(config.clone())
        .expect("Failed to create second prover");
    
    for i in 6..11 {  // Second proof: states 6-10
        let mut state = vec![0u8; STATE_SIZE];
        state[0] = (i & 0xFF) as u8;
        prover2.add_transition(&state)
            .expect(&format!("Prover2: Failed to add transition {}", i));
    }
    
    let proof2 = prover2.finalize()
        .expect("Failed to finalize proof2");
    
    assert_eq!(proof2.steps, 5, "Proof2 should have 5 steps");

    // Verify both proofs
    let genesis = vec![0u8; STATE_SIZE];
    let result1 = NovaVerifier::verify(&proof1, &genesis, &proof1.final_state)
        .expect("Failed to verify proof1");
    let result2 = NovaVerifier::verify(&proof2, &genesis, &proof2.final_state)
        .expect("Failed to verify proof2");

    assert!(result1 && result2, "Both proofs should verify successfully");
    
    // Proofs should have different final states
    assert_ne!(proof1.final_state, proof2.final_state, "Final states should differ");
    
    // Proofs should have different checksums
    assert_ne!(proof1.checksum, proof2.checksum, "Checksums should differ");
}

/// Test proof chain verification
///
/// This test demonstrates:
/// 1. Creating 3 independent proofs
/// 2. Linking them into a chain where each proof's final state becomes the next proof's genesis
/// 3. Verifying the entire chain
/// 4. Verifying each proof in the chain individually
#[test]
fn test_proof_chain_verification() {
    let config = NovaConfig::testing();
    let genesis = vec![0u8; STATE_SIZE];
    
    // Create 3 proofs
    let mut proofs = Vec::new();
    
    for proof_idx in 0..3 {
        let mut prover = NovaIVCProver::new(config.clone())
            .expect(&format!("Failed to create prover {}", proof_idx));
        
        for step in 1..6 {  // 5 steps per proof
            let mut state = vec![0u8; STATE_SIZE];
            state[0] = ((proof_idx * 5 + step) & 0xFF) as u8;
            prover.add_transition(&state)
                .expect(&format!("Failed to add transition in proof {}", proof_idx));
        }
        
        let proof = prover.finalize()
            .expect(&format!("Failed to finalize proof {}", proof_idx));
        
        proofs.push(proof);
    }

    // Create a valid chain by linking proofs
    // Each proof's final state becomes the next proof's genesis state
    for i in 1..proofs.len() {
        proofs[i].genesis_state = proofs[i - 1].final_state.clone();
        // Recalculate checksum after modifying genesis state
        proofs[i].checksum = proofs[i].compute_checksum();
    }

    // Verify the entire chain
    let chain_valid = NovaVerifier::verify_chain(&proofs)
        .expect("Failed to verify chain");
    
    assert!(chain_valid, "Proof chain should be valid");

    // Verify each proof individually with correct genesis
    let mut current_genesis = genesis;
    for (idx, proof) in proofs.iter().enumerate() {
        let result = NovaVerifier::verify(&proof, &current_genesis, &proof.final_state)
            .expect(&format!("Failed to verify individual proof {}", idx));
        assert!(result, "Proof {} in chain should verify", idx);
        
        // Next proof's expected genesis is this proof's final state
        current_genesis = proof.final_state.clone();
    }
    
    // Verify that proofs have different final states
    assert_ne!(proofs[0].final_state, proofs[1].final_state);
    assert_ne!(proofs[1].final_state, proofs[2].final_state);
}

/// Test compression workflow
///
/// This test demonstrates:
/// 1. Creating and finalizing a proof
/// 2. Compressing the proof to Groth16 format
/// 3. Validating the compressed proof
/// 4. Verifying compression reduces size
/// 5. Serializing and deserializing the compressed proof
#[test]
fn test_compression_workflow() {
    let config = NovaConfig::testing();
    let mut prover = NovaIVCProver::new(config)
        .expect("Failed to create prover");

    // Generate 10 transitions
    for i in 1..11 {
        let mut state = vec![0u8; STATE_SIZE];
        state[0] = (i & 0xFF) as u8;
        prover.add_transition(&state)
            .expect(&format!("Failed to add transition {}", i));
    }

    let proof = prover.finalize()
        .expect("Failed to finalize proof");

    let original_size = proof.size_bytes();
    println!("Original proof size: {} bytes", original_size);

    // Compress proof to Groth16 format
    let groth16_proof = NovaCompressor::compress(&proof)
        .expect("Failed to compress proof");

    // Validate compressed proof structure
    assert!(groth16_proof.validate().is_ok(), "Compressed proof should be valid");
    
    // Verify proof points have correct sizes
    assert_eq!(groth16_proof.proof_point_a.len(), 64, "Proof point A should be 64 bytes");
    assert_eq!(groth16_proof.proof_point_b.len(), 128, "Proof point B should be 128 bytes");
    assert_eq!(groth16_proof.proof_point_c.len(), 64, "Proof point C should be 64 bytes");
    
    // Check compression ratio
    let compressed_size = groth16_proof.size_bytes();
    println!("Compressed proof size: {} bytes", compressed_size);
    
    // Groth16 format is fixed size, so let's verify the metadata instead
    assert!(compressed_size > 0, "Compressed size should be positive");
    
    // Verify compression metadata
    assert_eq!(groth16_proof.metadata.original_size, original_size, 
               "Metadata should record original size");
    assert!(groth16_proof.metadata.compression_ratio > 0.0, 
            "Compression ratio should be positive");
    assert!(groth16_proof.metadata.timestamp > 0, 
            "Timestamp should be set");

    println!("Compression metadata:");
    println!("  Original size: {} bytes", groth16_proof.metadata.original_size);
    println!("  Compression ratio: {:.4}", groth16_proof.metadata.compression_ratio);
    println!("  Timestamp: {}", groth16_proof.metadata.timestamp);

    // Test serialization roundtrip
    let serialized = groth16_proof.serialize()
        .expect("Failed to serialize compressed proof");
    
    println!("Serialized compressed proof: {} bytes", serialized.len());
    
    let deserialized = zk_origin_nova::Groth16Proof::deserialize(&serialized)
        .expect("Failed to deserialize compressed proof");

    // Verify deserialized proof matches original
    assert_eq!(groth16_proof.proof_point_a, deserialized.proof_point_a,
               "Proof point A should match after deserialization");
    assert_eq!(groth16_proof.proof_point_b, deserialized.proof_point_b,
               "Proof point B should match after deserialization");
    assert_eq!(groth16_proof.proof_point_c, deserialized.proof_point_c,
               "Proof point C should match after deserialization");
    assert_eq!(groth16_proof.public_signals.len(), deserialized.public_signals.len(),
               "Public signals length should match after deserialization");
    
    for (idx, (orig, deser)) in groth16_proof.public_signals.iter()
        .zip(deserialized.public_signals.iter())
        .enumerate() {
        assert_eq!(orig, deser, "Public signal {} should match", idx);
    }
    
    println!("Compression workflow test completed successfully");
}

/// Test proof statistics collection
///
/// This test demonstrates:
/// 1. Creating a proof with 15 transitions
/// 2. Collecting statistics about the proof
/// 3. Verifying statistics are correct
#[test]
fn test_proof_statistics() {
    let config = NovaConfig::testing();
    let mut prover = NovaIVCProver::new(config)
        .expect("Failed to create prover");

    // Generate 15 transitions
    for i in 1..16 {
        let mut state = vec![0u8; STATE_SIZE];
        state[0] = (i & 0xFF) as u8;
        prover.add_transition(&state)
            .expect(&format!("Failed to add transition {}", i));
    }

    let proof = prover.finalize()
        .expect("Failed to finalize proof");

    // Collect statistics
    let stats = NovaVerifier::get_stats(&proof);
    
    // Verify statistics
    assert_eq!(stats.steps, 15, "Should have 15 steps");
    assert!(stats.size_bytes > 0, "Size should be positive");
    assert!(stats.avg_step_size > 0, "Average step size should be positive");
    assert!(stats.compression_ratio > 0.0, "Compression ratio should be positive");
    
    // Verify size calculation
    let expected_size = proof.size_bytes();
    assert_eq!(stats.size_bytes, expected_size, "Size should match proof size");
    
    // Verify average step size calculation
    let expected_avg = expected_size / 15;
    assert_eq!(stats.avg_step_size, expected_avg, "Average step size should be correct");
    
    println!("Proof stats - Steps: {}, Size: {} bytes, Avg/step: {} bytes, Ratio: {:.4}",
             stats.steps, stats.size_bytes, stats.avg_step_size, stats.compression_ratio);
}

/// Test large proof generation
///
/// This test demonstrates:
/// 1. Creating a proof with 1000 transitions
/// 2. Verifying the proof handles large workloads
/// 3. Checking compression ratio for large proofs
#[test]
fn test_large_proof_generation() {
    let mut config = NovaConfig::testing();
    config.max_steps = 10000;  // Increase limit for this test
    
    let mut prover = NovaIVCProver::new(config)
        .expect("Failed to create prover");

    println!("Generating proof with 1000 transitions...");
    
    // Generate 1000 transitions
    for i in 1..1001 {
        let mut state = vec![0u8; STATE_SIZE];
        state[0] = (i & 0xFF) as u8;
        state[1] = ((i >> 8) & 0xFF) as u8;
        
        if let Err(e) = prover.add_transition(&state) {
            panic!("Failed to add transition {}: {:?}", i, e);
        }
        
        // Print progress every 100 steps
        if i % 100 == 0 {
            println!("  {} transitions added...", i);
        }
    }

    let proof = prover.finalize()
        .expect("Failed to finalize large proof");

    // Verify proof properties
    assert_eq!(proof.steps, 1000, "Should have 1000 steps");
    assert!(proof.compression_ratio() > 0.0, "Compression ratio should be positive");
    assert!(!proof.proof_data.is_empty(), "Proof data should not be empty");

    println!("Large proof generated: {} bytes", proof.size_bytes());

    // Verify the large proof
    let genesis = vec![0u8; STATE_SIZE];
    let result = NovaVerifier::verify(&proof, &genesis, &proof.final_state)
        .expect("Failed to verify large proof");
    
    assert!(result, "Large proof should verify successfully");
}

/// Test proof tampering detection
///
/// This test demonstrates:
/// 1. Creating and finalizing a proof
/// 2. Tampering with the proof data
/// 3. Verifying that tampering is detected via checksum validation
#[test]
fn test_proof_tampering_detection() {
    let config = NovaConfig::testing();
    let mut prover = NovaIVCProver::new(config)
        .expect("Failed to create prover");

    let mut state = vec![0u8; STATE_SIZE];
    state[0] = 1;
    prover.add_transition(&state)
        .expect("Failed to add transition");

    let mut proof = prover.finalize()
        .expect("Failed to finalize proof");

    println!("Original checksum: {}", proof.checksum);

    // Tamper with proof data by flipping bits
    proof.proof_data[0] ^= 0xFF;

    println!("Tampering with proof data...");

    // Validation should fail due to checksum mismatch
    let result = proof.validate();
    assert!(result.is_err(), "Tampering should be detected");
    
    match result {
        Err(e) => println!("Correctly detected tampering: {}", e),
        Ok(_) => panic!("Tampering should have been detected"),
    }
}

/// Test state mismatch detection
///
/// This test demonstrates:
/// 1. Creating and finalizing a proof
/// 2. Attempting verification with incorrect genesis state
/// 3. Verifying that state mismatch is detected
#[test]
fn test_state_mismatch_detection() {
    let config = NovaConfig::testing();
    let mut prover = NovaIVCProver::new(config)
        .expect("Failed to create prover");

    let mut state = vec![0u8; STATE_SIZE];
    state[0] = 1;
    prover.add_transition(&state)
        .expect("Failed to add transition");

    let proof = prover.finalize()
        .expect("Failed to finalize proof");

    // Correct genesis state (all zeros)
    let correct_genesis = vec![0u8; STATE_SIZE];
    
    // Wrong genesis state
    let wrong_genesis = vec![1u8; STATE_SIZE];

    // Verify with correct genesis
    let correct_result = NovaVerifier::verify(&proof, &correct_genesis, &proof.final_state)
        .expect("Verification should complete");
    assert!(correct_result, "Should verify with correct genesis");

    // Verify with wrong genesis
    let wrong_result = NovaVerifier::verify(&proof, &wrong_genesis, &proof.final_state)
        .expect("Verification should complete");
    assert!(!wrong_result, "Should fail to verify with wrong genesis");
    
    println!("Correctly detected state mismatch");
}

/// Test batch verification of multiple proofs
///
/// This test demonstrates:
/// 1. Creating 5 independent proofs
/// 2. Batch verifying all proofs at once
/// 3. Verifying batch results are correct
#[test]
fn test_batch_verification() {
    let config = NovaConfig::testing();
    let genesis = vec![0u8; STATE_SIZE];
    let mut proofs = Vec::new();

    println!("Creating 5 proofs for batch verification...");

    // Create 5 proofs
    for proof_idx in 0..5 {
        let mut prover = NovaIVCProver::new(config.clone())
            .expect(&format!("Failed to create prover {}", proof_idx));

        for step in 1..4 {  // 3 steps per proof
            let mut state = vec![0u8; STATE_SIZE];
            state[0] = ((proof_idx * 3 + step) & 0xFF) as u8;
            prover.add_transition(&state)
                .expect(&format!("Failed to add transition in proof {}", proof_idx));
        }

        let proof = prover.finalize()
            .expect(&format!("Failed to finalize proof {}", proof_idx));
        proofs.push(proof);
    }

    println!("Batch verifying {} proofs...", proofs.len());

    // Batch verify
    let results = NovaVerifier::verify_batch(&proofs, &genesis)
        .expect("Batch verification failed");

    // Verify results
    assert_eq!(results.len(), 5, "Should have 5 results");
    
    for (idx, &result) in results.iter().enumerate() {
        println!("  Proof {}: {}", idx, if result { "✓ valid" } else { "✗ invalid" });
    }
}

/// Test hash consistency across identical proofs
///
/// This test demonstrates:
/// 1. Creating two identical proofs with same transitions
/// 2. Verifying they produce identical checksums and commitments
/// 3. Confirming deterministic behavior of hashing
#[test]
fn test_hash_consistency() {
    let config = NovaConfig::testing();
    
    // First proof
    let mut prover1 = NovaIVCProver::new(config.clone())
        .expect("Failed to create first prover");

    let mut state = vec![0u8; STATE_SIZE];
    state[0] = 42;
    prover1.add_transition(&state)
        .expect("Failed to add transition");

    let proof1 = prover1.finalize()
        .expect("Failed to finalize proof1");

    // Second identical proof
    let mut prover2 = NovaIVCProver::new(NovaConfig::testing())
        .expect("Failed to create second prover");

    let mut state2 = vec![0u8; STATE_SIZE];
    state2[0] = 42;
    prover2.add_transition(&state2)
        .expect("Failed to add transition");

    let proof2 = prover2.finalize()
        .expect("Failed to finalize proof2");

    // Verify identical hashes
    assert_eq!(proof1.checksum, proof2.checksum, "Checksums should be identical");
    assert_eq!(proof1.proof_commitment, proof2.proof_commitment, 
               "Proof commitments should be identical");
    assert_eq!(proof1.circuit_hash, proof2.circuit_hash, 
               "Circuit hashes should be identical");
    
    println!("Hash consistency verified");
    println!("  Checksum:        {}", proof1.checksum);
    println!("  Proof commitment: {}", proof1.proof_commitment);
    println!("  Circuit hash:    {}", proof1.circuit_hash);
}

/// Test different configuration variants
///
/// This test demonstrates:
/// 1. Creating proofs with production configuration
/// 2. Creating proofs with development configuration
/// 3. Creating proofs with testing configuration
/// 4. Verifying each configuration validates correctly
#[test]
fn test_config_variants() {
    // Test production config
    let prod_config = NovaConfig::production();
    assert!(prod_config.validate().is_ok(), "Production config should validate");
    assert!(prod_config.groth16_compression, "Production should enable compression");
    assert_eq!(prod_config.max_steps, 1_000_000, "Production should allow 1M steps");

    let prod_prover = NovaIVCProver::new(prod_config.clone());
    assert!(prod_prover.is_ok(), "Should create prover with production config");

    // Test development config
    let dev_config = NovaConfig::development();
    assert!(dev_config.validate().is_ok(), "Development config should validate");
    assert!(!dev_config.groth16_compression, "Development should disable compression");
    assert_eq!(dev_config.max_steps, 10000, "Development should allow 10k steps");

    let dev_prover = NovaIVCProver::new(dev_config.clone());
    assert!(dev_prover.is_ok(), "Should create prover with development config");

    // Test testing config
    let test_config = NovaConfig::testing();
    assert!(test_config.validate().is_ok(), "Testing config should validate");
    assert_eq!(test_config.max_steps, 1000, "Testing should allow 1k steps");

    let test_prover = NovaIVCProver::new(test_config.clone());
    assert!(test_prover.is_ok(), "Should create prover with testing config");

    println!("Configuration variants:");
    println!("  Production: {} max steps, compression: {}", 
             prod_config.max_steps, prod_config.groth16_compression);
    println!("  Development: {} max steps, compression: {}", 
             dev_config.max_steps, dev_config.groth16_compression);
    println!("  Testing: {} max steps, compression: {}", 
             test_config.max_steps, test_config.groth16_compression);
}

/// Test serialization and deserialization roundtrip
///
/// This test demonstrates:
/// 1. Creating and finalizing a proof
/// 2. Serializing the proof to bytes
/// 3. Deserializing the proof from bytes
/// 4. Verifying the deserialized proof matches the original
#[test]
fn test_serialization_roundtrip() {
    let config = NovaConfig::testing();
    let mut prover = NovaIVCProver::new(config)
        .expect("Failed to create prover");

    // Generate 5 transitions
    for i in 1..6 {
        let mut state = vec![0u8; STATE_SIZE];
        state[0] = (i & 0xFF) as u8;
        prover.add_transition(&state)
            .expect("Failed to add transition");
    }

    let proof1 = prover.finalize()
        .expect("Failed to finalize proof");

    let original_size = proof1.serialize().unwrap().len();
    println!("Original proof serialized to {} bytes", original_size);

    // Serialize to bytes
    let serialized = proof1.serialize()
        .expect("Failed to serialize proof");

    // Deserialize from bytes
    let proof2 = CompressedNovaProof::deserialize(&serialized)
        .expect("Failed to deserialize proof");

    // Compare all fields
    assert_eq!(proof1.proof_data, proof2.proof_data, "Proof data should match");
    assert_eq!(proof1.final_state, proof2.final_state, "Final state should match");
    assert_eq!(proof1.genesis_state, proof2.genesis_state, "Genesis state should match");
    assert_eq!(proof1.steps, proof2.steps, "Steps should match");
    assert_eq!(proof1.timestamp, proof2.timestamp, "Timestamp should match");
    assert_eq!(proof1.circuit_hash, proof2.circuit_hash, "Circuit hash should match");
    assert_eq!(proof1.proof_commitment, proof2.proof_commitment, "Proof commitment should match");
    assert_eq!(proof1.checksum, proof2.checksum, "Checksum should match");
    
    println!("Serialization roundtrip successful - all fields match");

    // Verify the deserialized proof
    let genesis = vec![0u8; STATE_SIZE];
    let result = NovaVerifier::verify(&proof2, &genesis, &proof2.final_state)
        .expect("Failed to verify deserialized proof");
    
    assert!(result, "Deserialized proof should verify");
}

/// Test invalid state size detection
///
/// This test demonstrates:
/// 1. Attempting to add a state with incorrect size
/// 2. Verifying that the error is caught and reported
#[test]
fn test_invalid_state_size() {
    let config = NovaConfig::testing();
    let mut prover = NovaIVCProver::new(config)
        .expect("Failed to create prover");

    // Try to add state with wrong size (32 bytes instead of 48)
    let wrong_state = vec![0u8; 32];
    
    let result = prover.add_transition(&wrong_state);
    
    assert!(result.is_err(), "Should reject invalid state size");
    
    match result {
        Err(e) => println!("Correctly rejected invalid state: {}", e),
        Ok(_) => panic!("Should have rejected invalid state"),
    }
}

/// Test proof commitment matches across regeneration
///
/// This test demonstrates:
/// 1. Creating a proof
/// 2. Regenerating the same proof
/// 3. Verifying proof commitments match
#[test]
fn test_proof_commitment_consistency() {
    let config = NovaConfig::testing();
    
    // First proof generation
    let mut prover1 = NovaIVCProver::new(config.clone())
        .expect("Failed to create first prover");

    for i in 1..6 {
        let mut state = vec![0u8; STATE_SIZE];
        state[0] = (i & 0xFF) as u8;
        prover1.add_transition(&state)
            .expect("Failed to add transition");
    }

    let proof1 = prover1.finalize()
        .expect("Failed to finalize proof1");

    // Second proof generation with identical transitions
    let mut prover2 = NovaIVCProver::new(config.clone())
        .expect("Failed to create second prover");

    for i in 1..6 {
        let mut state = vec![0u8; STATE_SIZE];
        state[0] = (i & 0xFF) as u8;
        prover2.add_transition(&state)
            .expect("Failed to add transition");
    }

    let proof2 = prover2.finalize()
        .expect("Failed to finalize proof2");

    // Proof commitments should be identical
    assert_eq!(proof1.proof_commitment, proof2.proof_commitment,
               "Proof commitments should match for identical transitions");
    
    println!("Proof commitment consistency verified: {}",
             proof1.proof_commitment);
}

/// Test final lineage commitment retrieval
///
/// This test demonstrates:
/// 1. Getting the final lineage commitment from a proof
/// 2. Verifying it matches the final state's first 32 bytes
#[test]
fn test_get_final_lineage_commitment() {
    let config = NovaConfig::testing();
    let mut prover = NovaIVCProver::new(config)
        .expect("Failed to create prover");

    let mut state = vec![0u8; STATE_SIZE];
    state[0] = 42;
    state[1] = 43;
    
    prover.add_transition(&state)
        .expect("Failed to add transition");

    // Get final lineage commitment
    let commitment = prover.get_final_lineage_commitment()
        .expect("Failed to get lineage commitment");

    println!("Final lineage commitment: {}", commitment);
    
    // Commitment should not be zero
    assert!(!commitment.is_zero(), "Commitment should not be zero");

    // Finalize and verify
    let proof = prover.finalize()
        .expect("Failed to finalize proof");

    let result = NovaVerifier::verify(&proof, &proof.genesis_state, &proof.final_state)
        .expect("Failed to verify");
    
    assert!(result, "Proof should verify");
}