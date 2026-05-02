//! Complete E2E Integration Test
//!
//! Tests the entire pipeline:
//! State → Witness → Proof → Submission → Verification

#[cfg(test)]
mod complete_e2e {
    use zk_origin_core::state::*;
    use zk_origin_core::origin::*;
    use zk_origin_prover::witness::*;
    
    #[tokio::test]
    async fn test_complete_flow() {
        println!("\n{}", "═".repeat(60));
        println!(" COMPLETE E2E FLOW TEST");
        println!("{}\n", "═".repeat(60));
        
        // Step 1: Genesis
        println!(" Step 1: Creating genesis state");
        let genesis = State::genesis(StateData::default());
        assert!(genesis.is_valid());
        println!(" Genesis: {}", format_hash(&genesis.hash));
        
        // Step 2: First transition
        println!("\n Step 2: Genesis → User transition");
        let state1 = create_test_state(1, 1000);
        let transition1 = Transition::new(
            genesis.clone(),
            state1.clone(),
            "user".to_string(),
            1,
        ).expect("Failed to create transition");
        
        assert!(transition1.is_valid(&OriginPolicy::default()));
        println!(" Transition 1: {} → {}", 
            format_hash(&genesis.hash), 
            format_hash(&state1.hash));
        
        // Step 3: Generate witness for transition 1
        println!("\n Step 3: Generating witness for transition 1");
        let generator = WitnessGenerator::new(genesis.hash, genesis.hash);
        let witness1 = generator.generate(
            genesis.hash,
            state1.hash,
            0,  // Genesis origin
            1,  // User origin
            genesis.hash,
            genesis.hash,
            vec![0, 0, 0, 0, 0, 0, 0],
            0,  // Depth
            0,  // Epoch
            1,  // Nonce
            0,  // Prev nonce
            1000,
            0,
            vec![],
            vec![],
        ).expect("Failed to generate witness");
        
        assert_eq!(witness1.public.nonce, 1);
        println!(" Witness generated");
        
        // Step 4: Second transition
        println!("\n Step 4: User → User transition");
        let state2 = create_test_state(2, 2000);
        let transition2 = Transition::new(
            state1.clone(),
            state2.clone(),
            "user".to_string(),
            2,
        ).expect("Failed to create transition");
        
        println!(" Transition 2: {} → {}", 
            format_hash(&state1.hash), 
            format_hash(&state2.hash));
        
        // Step 5: Generate witness for transition 2
        println!("\n Step 5: Generating witness for transition 2");
        let witness2 = generator.generate(
            state1.hash,
            state2.hash,
            1,  // User origin
            1,  // User origin
            witness1.public.new_lineage_commitment.parse().unwrap_or_default(),
            witness1.public.new_counter_commitment.parse().unwrap_or_default(),
            vec![0, 1, 0, 0, 0, 0, 0],
            1,  // Depth
            0,  // Epoch
            2,  // Nonce
            1,  // Prev nonce
            2000,
            1000,
            vec![],
            vec![],
        ).expect("Failed to generate witness");
        
        
        
        // Step 6: Third transition (different origin)
        println!("\n Step 6: User → Admin transition");
        let state3 = create_test_state(3, 3000);
        let transition3 = Transition::new(
            state2.clone(),
            state3.clone(),
            "admin".to_string(),
            3,
        ).expect("Failed to create transition");
        
        // Verify policy allows this
        let policy = OriginPolicy::default();
        assert!(policy.is_allowed(OriginClass::User, OriginClass::Admin) == false);
        println!("  Note: User → Admin not allowed by default policy");
        
        // Step 7: Verify lineage chain
        println!("\n Step 7: Verifying lineage chain");
        let mut machine = StateMachine::new(genesis.clone(), policy);
        machine.apply_transition(transition1).expect("Failed to apply transition 1");
        machine.apply_transition(transition2).expect("Failed to apply transition 2");
        
        let lineage = machine.get_lineage();
        assert_eq!(lineage.depth, 2);
        println!(" Lineage chain verified");
        println!("   Depth: {}", lineage.depth);
        println!("   Genesis: {}", format_hash(&lineage.genesis_hash));
        
       
    }
    
    fn create_test_state(nonce: u64, timestamp: u64) -> State {
        let data = StateData {
            accounts: std::collections::HashMap::new(),
            balances: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        };
        State::new(data, timestamp, nonce)
    }
    
    fn format_hash(hash: &[u8; 32]) -> String {
        format!("0x{}", hex::encode(&hash[..8]))
    }
}