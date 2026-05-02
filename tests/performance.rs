//! Performance tests

#[cfg(test)]
mod tests {
    use zk_origin_core::{State, StateData, Transition};
    use std::time::Instant;
    
    #[test]
    fn bench_transition_creation() {
        let iterations = 1000;
        let start = Instant::now();
        
        for i in 0..iterations {
            let prev = State::new(StateData::default(), 1000 + i as u64, i as u64);
            let new = State::new(StateData::default(), 2000 + i as u64, (i + 1) as u64);
            
            let _ = Transition::new(prev, new, "user".to_string(), (i + 1) as u64);
        }
        
        let elapsed = start.elapsed();
        let avg = elapsed.as_micros() / iterations as u128;
        
        println!("\nTransaction creation benchmark:");
        println!("  Iterations: {}", iterations);
        println!("  Total time: {:.2}ms", elapsed.as_millis());
        println!("  Average: {:.2}µs", avg);
    }
}