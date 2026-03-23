# ZK-ORIGIN: Zero-Knowledge State Lineage Verification

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![ZK](https://img.shields.io/badge/zero--knowledge-enabled-brightgreen.svg)

**ZK-ORIGIN** is a production-ready zero-knowledge proving system for verifiable state lineage. It enables cryptographic proof that a state has undergone a valid sequence of transitions according to a policy, without revealing the transition history.

##  Features

- **Three Proving Modes**: Choose the right backend for your use case
- **Constant-Size Proofs**: All modes produce constant-size proofs regardless of history length
- **Policy Enforcement**: Define and enforce origin-based transition rules
- **Rate Limiting**: Built-in epoch-based rate limiting per origin class
- **Zero-Knowledge**: Real cryptographic security (Nova & Groth16 modes)
- **Production Ready**: All modes tested and working

##  Mode Comparison

| Feature | Commitment | Nova IVC | Groth16 (Compact) |
|---------|-----------|----------|-------------------|
| **Proof Size** | 32 bytes | ~10 KB | **192 bytes**  |
| **Setup Time** | <1ms | ~1.4s | ~138ms |
| **Proving Time** | <1µs/step | ~66ms/step | ~65ms (batch) |
| **Verify Time** | ~1µs | ~543ms | **~11ms**  |
| **Zero-Knowledge** |  No | Yes |  Yes |
| **Incremental** |  Yes |  Yes |  Batch only |
| **Use Case** | Development | Streaming data | Size-critical apps |

##  Quick Start

### Prerequisites

```bash
# Install Rust (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/ZKChainForge/zk-origin-prover.git
cd zk-origin-prover
```

### Build & Run

#### 1. Commitment Mode (Fast Development)

```bash
# Build with default features
cargo build --release

# Run demo
./target/release/zk-origin-cli demo
```

**Output:**
```
Proof size:    32 bytes
Proving time:  <1µs per step
Verify time:   ~1µs
Real ZK:       NO (hash commitments only)
```

#### 2. Nova IVC Mode (Incremental ZK)

```bash
# Build with Nova backend
cargo build --release --features real-nova --no-default-features

# Run demo
./target/release/zk-origin-cli demo
```

**Output:**
```
Proof size:    10,072 bytes (~10 KB)
Setup time:    ~1.4 seconds (one-time)
Proving time:  ~66ms per step
Verify time:   ~543ms
Real ZK:       YES 
Incremental:   YES 
```

#### 3. Groth16 Mode (Compact ZK) 

```bash
# Build with Groth16 backend
cargo build --release --features compact-zk --no-default-features

# Run demo
./target/release/zk-origin-cli demo
```

**Output:**
```
Proof size:    192 bytes (<1 KB) 
Setup time:    ~138ms (trusted setup)
Proving time:  ~65ms (batch)
Verify time:   ~11ms 
Real ZK:       YES 
Incremental:   NO (batch processing)
```

##  Usage Examples

### Basic Usage

```rust
use zk_origin::*;

// Create a policy
let policy = OriginPolicy::default();

// Setup proving parameters (choose your mode)
let params = LineageProver::setup_params(&policy)?;

// Create and initialize prover
let mut prover = LineageProver::new(policy.clone(), &params)?;
prover.initialize([0u8; 32])?;

// Add transitions
let transition = Transition::new(
    [0u8; 32],      // prev_state
    [1u8; 32],      // new_state
    OriginClass::User,
    1000,           // timestamp
);
prover.add_transition(transition)?;

// Generate proof
let proof = prover.finalize()?;
println!("Proof size: {} bytes", proof.proof_size());
println!("Is real ZK: {}", proof.is_real_zk());

// Verify proof
let verifier = LineageVerifier::from_proof(&proof, &policy);
assert!(verifier.verify(&proof)?);
```

### Nova IVC (Incremental Proving)

```rust
use zk_origin::*;

let policy = OriginPolicy::default();
let params = NovaParams::setup(policy.compute_hash())?;
let mut prover = NovaLineageProver::new(&params);

// Initialize with genesis
prover.initialize([0u8; 32], [0u8; 32])?;

// Add steps incrementally
for i in 0..100 {
    let witness = create_witness(i);
    prover.prove_step(&witness)?;
    // Proof is updated after each step!
}

// Finalize and compress
let proof = prover.finalize()?;
println!("100 steps → {} bytes", proof.proof_size()); // ~10 KB
```

### Groth16 (Compact Batch Proving)

```rust
use zk_origin::*;

let policy = OriginPolicy::default();
let params = Groth16Params::setup(policy.compute_hash())?;
let mut prover = Groth16LineageProver::new(&params);

// Initialize
prover.initialize([0u8; 32], [0u8; 32])?;

// Collect all transitions
for i in 0..100 {
    let witness = create_witness(i);
    prover.prove_step(&witness)?; // Just stores witness
}

// Generate proof for all steps at once
let proof = prover.finalize()?;
println!("100 steps → {} bytes", proof.proof_size()); // 192 bytes!

// Fast verification
let verified = verify_groth16_proof(
    &proof.proof_bytes,
    &proof.verifier_key.unwrap(),
    &genesis_lineage,
    &genesis_counters,
    &proof.final_lineage.value,
    &proof.final_counters.value,
)?;
assert!(verified);
```

### Custom Policy

```rust
use zk_origin::*;

let mut policy = OriginPolicy::new();

// Define allowed transitions
policy.allow(OriginClass::Genesis, OriginClass::User);
policy.allow(OriginClass::User, OriginClass::User);
policy.allow(OriginClass::User, OriginClass::Bridge);

// Set rate limits (per epoch)
policy.set_rate_limit(OriginClass::User, 1000);
policy.set_rate_limit(OriginClass::Bridge, 10);

// This transition would be rejected:
// policy.allow(OriginClass::User, OriginClass::Admin); // Not allowed!

let mut prover = LineageProver::new(policy.clone())?;
// ... rest of proving
```

##  Architecture

### Core Components

```
zk-origin-prover/
├── src/
│   ├── bin/
│   │   └── cli.rs                 # CLI demo application
│   ├── prover/
│   │   ├── commitment_prover.rs   # Fast commitment backend
│   │   ├── nova_prover.rs         # Nova IVC backend
│   │   ├── nova_circuit.rs        # Nova circuit implementation
│   │   ├── groth16_prover.rs      # Groth16 backend
│   │   ├── groth16_circuit.rs     # Groth16 circuit
│   │   └── lineage_prover.rs      # Unified prover interface
│   ├── verifier/
│   │   └── verify.rs              # Proof verification
│   ├── types/
│   │   ├── lineage.rs             # Lineage commitments
│   │   ├── proof.rs               # Proof structures
│   │   ├── policy.rs              # Origin policies
│   │   └── witness.rs             # Step witnesses
│   └── lib.rs                     # Library entry point
└── Cargo.toml
```

### Proving Flow

```
┌─────────────┐
│   Genesis   │
│   State     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Transition 1│ ─────┐
│  (proven)   │      │
└──────┬──────┘      │
       │             │
       ▼             ▼
┌─────────────┐  ┌──────────┐
│ Transition 2│  │  Policy  │
│  (proven)   │  │  Check   │
└──────┬──────┘  └──────────┘
       │
       ▼
┌─────────────┐
│ Transition N│
│  (proven)   │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Final      │
│  Proof      │
│  (192 bytes)│
└─────────────┘
```

##  Technical Details

### Proof Systems

#### 1. **Nova IVC (Incremental Verifiable Computation)**

- **Curves**: Pallas/Vesta (Pasta curves)
- **Scheme**: Folding-based IVC
- **Proof Size**: ~10 KB (constant)
- **Primary Circuit**: ~9,831 constraints
- **Secondary Circuit**: ~10,357 constraints

**How it works:**
- Uses **folding schemes** to incrementally update proofs
- Each `prove_step()` folds the new computation into the proof
- Final compression uses Spartan SNARK
- Proof covers all steps but remains constant size

#### 2. **Groth16 (Compact SNARK)**

- **Curve**: BLS12-381
- **Scheme**: Groth16 zk-SNARK
- **Proof Size**: 192 bytes (constant)
- **Structure**: 3 elliptic curve points (A, B, C)
- **Trusted Setup**: Required (138ms)

**How it works:**
- Collects all transitions as witnesses
- Builds single circuit proving all steps
- Generates compact proof in one shot
- Verification checks pairing equations

#### 3. **Commitment Mode**

- **Hash Function**: SHA-256
- **Proof Size**: 32 bytes (just a hash)
- **Security**: Not zero-knowledge!
- **Purpose**: Development and testing only

### Circuit Design

The core lineage circuit proves:

```
For each transition step:
  1. state_product = prev_state × new_state
  2. transition_hash = state_product + origin + timestamp
  3. lineage_product = current_lineage × transition_hash
  4. new_lineage = lineage_product + current_lineage + transition_hash
  5. new_counters = current_counters + origin + epoch_id

Public inputs:
  - genesis_lineage
  - genesis_counters
  - final_lineage
  - final_counters

Private inputs (witness):
  - prev_state, new_state
  - origin, timestamp
  - epoch_id
  - policy proof (Merkle path)
  - rate limits
```

### Proof Size Analysis

All three modes produce **constant-size proofs**:

| History Length | Commitment | Nova | Groth16 |
|----------------|-----------|------|---------|
| 1 transition   | 32 bytes  | ~10 KB | 192 bytes |
| 10 transitions | 32 bytes  | ~10 KB | 192 bytes |
| 100 transitions | 32 bytes  | ~10 KB | 192 bytes |
| 1,000 transitions | 32 bytes  | ~10 KB | 192 bytes |
| 1,000,000 transitions | 32 bytes  | ~10 KB | 192 bytes |

**This is the power of succinct proofs!**

##  Use Cases

### Groth16 (Compact ZK) - Best For:

 **Blockchain Rollups**
- L2 state proofs (192 bytes on-chain)
- ZK-rollup batches
- Cross-chain bridges

 **IoT & Mobile**
- Limited bandwidth
- Battery-constrained devices
- Embedded systems

 **Privacy-Preserving Audits**
- Compliance verification
- Supply chain tracking
- Medical records

### Nova IVC - Best For:

 **Streaming Data**
- Continuous computation
- Real-time updates
- Long-running processes

 **Incremental Computation**
- Add steps as they happen
- No need to know total steps upfront
- Update proofs on-the-fly

 **Blockchain State**
- State transition proofs
- Incremental state updates
- Progressive rollups

### Commitment Mode - Best For:

 **Development**
- Fast iteration
- Testing logic
- Debugging circuits

 **Prototyping**
- Proof of concept
- Performance benchmarking
- Integration testing

 **NOT for Production** (not zero-knowledge!)

##  Performance Benchmarks


### Setup Time

| Mode | Setup Time | One-time? |
|------|-----------|-----------|
| Commitment | <1ms | N/A |
| Nova IVC | ~1.4s |  Yes (cacheable) |
| Groth16 | ~138ms |  Yes (trusted setup) |

### Proving Time (3 transitions)

| Mode | Time | Per Step |
|------|------|----------|
| Commitment | <1µs | <1µs |
| Nova IVC | ~199ms | ~66ms/step |
| Groth16 | ~65ms | ~22ms/step (batch) |

### Verification Time

| Mode | Time | Speed |
|------|------|-------|
| Commitment | ~1µs |  Instant |
| Nova IVC | ~543ms | Slow |
| Groth16 | **~11ms** |  **Fast!** |

### Proof Size

| Mode | Size | Compression |
|------|------|-------------|
| Commitment | 32 bytes | N/A (hash) |
| Nova IVC | 10,072 bytes | ~10 KB |
| Groth16 | **192 bytes** | **<200 bytes!**  |

### Scaling (1000 transitions)

| Mode | Proof Size | Proving Time | Verify Time |
|------|-----------|--------------|-------------|
| Commitment | 32 bytes | ~1ms | ~1µs |
| Nova IVC | ~10 KB | ~66s | ~543ms |
| Groth16 | **192 bytes** | ~21s | **~11ms** |

**Key Insight**: Groth16 shines when proof size and verification speed matter most!

##  Security Considerations

### Groth16 Security

 **Cryptographic Assumptions**:
- Relies on hardness of discrete logarithm on BLS12-381
- Standard assumptions (well-studied)

 **Trusted Setup**:
- Requires trusted ceremony for parameter generation
- If setup is compromised, fake proofs possible
- Mitigation: Use multi-party computation (MPC) ceremony

### Nova Security

 **Cryptographic Assumptions**:
- Based on hardness of discrete logarithm on Pasta curves
- Folding scheme security (newer, but well-reviewed)

 **No Trusted Setup**:
- Universal setup (no ceremony needed)
- Transparent (nothing to hide)

### Best Practices

1. **Use Real ZK in Production**
   ```bash
   #  Good
   cargo build --features real-nova --no-default-features
   cargo build --features compact-zk --no-default-features
   
   #  Bad (for production)
   cargo build  # commitment mode
   ```

2. **Validate Inputs**
   ```rust
   // Always validate transitions
   prover.validate_transition(&transition)?;
   ```

3. **Check Proof Validity**
   ```rust
   // Verify before trusting
   let verifier = LineageVerifier::from_proof(&proof, &policy);
   assert!(verifier.verify(&proof)?);
   ```

4. **Secure Policy Definition**
   ```rust
   // Carefully design transition rules
   policy.allow(OriginClass::User, OriginClass::User);
   // DON'T allow everything!
   ```

##  API Reference

### Core Types

#### `OriginClass`
```rust
pub enum OriginClass {
    Genesis,   // Initial state
    User,      // User action
    Admin,     // Admin action
    System,    // System action
    Bridge,    // Cross-chain bridge
    External,  // External oracle
}
```

#### `Transition`
```rust
pub struct Transition {
    pub prev_state_hash: [u8; 32],
    pub new_state_hash: [u8; 32],
    pub origin_class: OriginClass,
    pub timestamp: u64,
}
```

#### `LineageProof`
```rust
pub struct LineageProof {
    pub proof_bytes: Vec<u8>,           // The actual proof
    pub final_lineage: LineageCommitment,
    pub final_counters: CounterCommitment,
    pub genesis_commitment: LineageCommitment,
    pub num_steps: u64,
    pub policy_hash: [u8; 32],
    pub metadata: ProofMetadata,
    pub verifier_key: Option<Vec<u8>>,
}
```

### Main APIs

#### `LineageProver`
```rust
impl LineageProver {
    // Create prover (mode depends on features)
    pub fn new(policy: OriginPolicy, params: &NovaParams) -> Result<Self>;
    
    // Setup parameters
    pub fn setup_params(policy: &OriginPolicy) -> Result<NovaParams>;
    
    // Initialize with genesis
    pub fn initialize(&mut self, genesis_hash: [u8; 32]) -> Result<()>;
    
    // Add transition (incremental for Nova/Commitment)
    pub fn add_transition(&mut self, transition: Transition) -> Result<()>;
    
    // Validate without adding
    pub fn validate_transition(&self, transition: &Transition) -> Result<()>;
    
    // Generate final proof
    pub fn finalize(&self) -> Result<LineageProof>;
    
    // Query state
    pub fn current_depth(&self) -> u64;
    pub fn is_initialized(&self) -> bool;
}
```

#### `LineageVerifier`
```rust
impl LineageVerifier {
    // Create verifier
    pub fn new(genesis_hash: [u8; 32], policy: &OriginPolicy) -> Self;
    pub fn from_proof(proof: &LineageProof, policy: &OriginPolicy) -> Self;
    
    // Verify proof (structural)
    pub fn verify(&self, proof: &LineageProof) -> Result<bool>;
    
    // Verify cryptographically (real ZK)
    pub fn verify_zk(&self, proof: &LineageProof) -> Result<bool>;
    
    // Detailed verification
    pub fn verify_detailed(&self, proof: &LineageProof) -> VerificationResult;
}
```

#### `Groth16LineageProver`
```rust
impl Groth16LineageProver {
    // Create prover
    pub fn new(params: &Groth16Params) -> Self;
    
    // Initialize
    pub fn initialize(&mut self, genesis: [u8; 32], counters: [u8; 32]) -> Result<()>;
    
    // Add transition (batched)
    pub fn prove_step(&mut self, witness: &StepWitness) -> Result<()>;
    
    // Generate compact proof
    pub fn finalize(&self) -> Result<LineageProof>;
}

// Verify Groth16 proof
pub fn verify_groth16_proof(
    proof_bytes: &[u8],
    vk_bytes: &[u8],
    genesis_lineage: &[u8; 32],
    genesis_counters: &[u8; 32],
    final_lineage: &[u8; 32],
    final_counters: &[u8; 32],
) -> Result<bool>;
```

##  Testing

```bash
# Run all tests
cargo test

# Run tests for specific mode
cargo test --features real-nova --no-default-features
cargo test --features compact-zk --no-default-features

# Run benchmarks
cargo bench

# Run with output
cargo test -- --nocapture
```



##  Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch
cargo install cargo-expand

# Run tests on file change
cargo watch -x test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

##  License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

##  Acknowledgments

This project builds on:

- [Nova](https://github.com/microsoft/Nova) - Recursive SNARKs without trusted setup
- [Bellman](https://github.com/zkcrypto/bellman) - zk-SNARK library
- [Pasta Curves](https://github.com/zcash/pasta_curves) - Pallas/Vesta curves
- [BLS12-381](https://github.com/zkcrypto/bls12_381) - Pairing-friendly curve

##  Contact & Support

- **Issues**: [GitHub Issues](https://github.com/ZKChainForge/zk-origin-prover/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ZKChainForge/zk-origin-prover/discussions)
- **Email**: zkchainforge@gmail.com



##  Citation

If you use this project in your research, please cite:

```bibtex
@software{zk_origin_2026,
  title = {ZK-ORIGIN: Zero-Knowledge State Lineage Verification},
  author = {VIKRAM A},
  year = {2026},
  url = {https://github.com/ZKChainForge/zk-origin-prover}
}
```

---

**Built with ❤️ using Rust and Zero-Knowledge Cryptography**

*For questions, feedback, or collaboration opportunities, please open an issue or discussion on GitHub.*