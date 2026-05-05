//! Performance Benchmarking
//!
//! Measure proof generation, verification, and submission times

#[cfg(test)]
mod benchmarks {
    use zk_origin_core::state::*;
    use zk_origin_prover::witness::*;
    use zk_origin_prover::groth16::*;
    use std::time::Instant;
    
    #[test]
    fn bench_witness_generation() {
        let generator = WitnessGenerator::new([0u8; 32], [0u8; 32]);
        
        let iterations = 100;
        let start = Instant::now();
        
        for i in 0..iterations {
            let _ = generator.generate(
                [i as u8; 32],
                [(i + 1) as u8; 32],
                1,
                1,
                [0u8; 32],
                [0u8; 32],
                vec![0, i as u32, 0, 0, 0, 0, 0],
                i as u32,
                0,
                (i + 1) as u64,
                i as u64,
                1000 + i as u64,
                999 + i as u64,
                vec![],
                vec![],
            );
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed.as_micros() / iterations as u128;
        
        println!("\n📊 WITNESS GENERATION BENCHMARK");
        println!("   Iterations: {}", iterations);
        println!("   Total time: {:.2}ms", elapsed.as_millis());
        println!("   Avg time: {:.2}µs", avg_time);
        println!("   Throughput: {:.2} witnesses/sec", 
            1_000_000.0 / avg_time as f64);
    }
    
    #[test]
    fn bench_state_validation() {
        let iterations = 1000;
        let policy = OriginPolicy::default();
        
        let start = Instant::now();
        
        for i in 0..iterations {
            let prev = create_test_state(i as u64, 1000 + i as u64);
            let new = create_test_state((i + 1) as u64, 2000 + i as u64);
            
            let transition = Transition::new(
                prev,
                new,
                "user".to_string(),
                (i + 1) as u64,
            );
            
            if let Ok(t) = transition {
                let _ = t.is_valid(&policy);
            }
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed.as_nanos() / iterations as u128;
        
        println!("\n TRANSITION VALIDATION BENCHMARK");
        println!("   Iterations: {}", iterations);
        println!("   Total time: {:.2}ms", elapsed.as_millis());
        println!("   Avg time: {:.2}ns", avg_time);
        println!("   Throughput: {:.2} transitions/sec", 
            1_000_000_000.0 / avg_time as f64);
    }
    
    #[test]
    fn bench_lineage_commitment() {
        let generator = WitnessGenerator::new([0u8; 32], [0u8; 32]);
        let iterations = 1000;
        
        let start = Instant::now();
        
        for i in 0..iterations {
            let _ = generator.generate(
                [i as u8; 32],
                [(i + 1) as u8; 32],
                1,
                1,
                [0u8; 32],
                [0u8; 32],
                vec![],
                i as u32,
                0,
                (i + 1) as u64,
                i as u64,
                1000 + i as u64,
                999 + i as u64,
                vec![],
                vec![],
            );
        }
        
        let elapsed = start.elapsed();
        
        println!("\nLINEAGE COMMITMENT BENCHMARK");
        println!("   Iterations: {}", iterations);
        println!("   Total time: {:.2}ms", elapsed.as_millis());
        println!("   Avg time: {:.2}µs", elapsed.as_micros() / iterations as u128);
    }
    
    fn create_test_state(nonce: u64, timestamp: u64) -> State {
        let data = StateData {
            accounts: std::collections::HashMap::new(),
            balances: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        };
        State::new(data, timestamp, nonce)
    }
}