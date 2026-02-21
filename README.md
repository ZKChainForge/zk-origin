
#  ZK-ORIGIN: Zero-Knowledge State Lineage Protocol

<div align="center">

![ZK-ORIGIN Banner](https://via.placeholder.com/1200x400/0A66C2/FFFFFF?text=ZK-ORIGIN)

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Circom](https://img.shields.io/badge/Circom-2.1%2B-blue.svg)](https://docs.circom.io)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Twitter](https://img.shields.io/twitter/follow/zkorigin?style=social)](https://x.co/zkchain_z41420)
[![LinkedIn](https://img.shields.io/badge/LinkedIn-Connect-blue)](https://linkedin.com/in/vikram-a-a6a252395)

**Prove where your state came from, not just that it's valid**

[Getting Started](#-getting-started) •
[Architecture](#-architecture) •
[Benchmarks](#-benchmarks) •
[Installation](#-installation) •
[Usage](#-usage) •
[API](#-api-reference)

</div>

---

##  **Table of Contents**

- [Why ZK-ORIGIN?](#-why-zk-origin)
- [Features](#-features)
- [Architecture](#-architecture)
- [Benchmarks](#-benchmarks)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Usage Guide](#-usage-guide)
- [API Reference](#-api-reference)
- [Project Structure](#-project-structure)
- [Contributing](#-contributing)
- [License](#-license)
- [Acknowledgments](#-acknowledgments)

---

##  **Why ZK-ORIGIN?**

```
┌─────────────────────────────────────────────────────────────────┐
│                    THE SILENT CRISIS                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Every ZK system today answers:                                 │
│   "Is this state valid?"                                        │
│                                                                 │
│  But NONE can answer:                                           │
│   "Where did this state come from?"                             │
│                                                                 │
│  This gap has caused:                                           │
│  • $2B+ in bridge exploits                                      │
│  • $500M+ in governance attacks                                 │
│  • $1B+ in admin key compromises                                │
│                                                                 │
│  ZK-ORIGIN solves this by proving STATE LINEAGE.                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Real Attack Scenarios:**

| Attack Type          | What Current ZK Misses                   | How ZK-ORIGIN Prevents                                                                               |
|----------------------|------------------------------------------                        |--------------------|
| Bridge Exploit       | Can't prove state came from source chain | Cryptographic proof of origin chain                                                                        |
| Admin Key Compromise | Can't distinguish admin vs user actions  | Origin classes                                                                                |
| Governance Attack    | Can't bind proposal to execution         | Governance origin tracking                                                                               |
| Malicious Upgrade    | Can't prove upgrade was authorized       | Admin origin with threshold sigs                                                                         |

---

##  **Features**

```
┌─────────────────────────────────────────────────────────────────┐
│                    CORE FEATURES                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ORIGIN CLASSES                                                │
│  ├── User: Normal transactions                                  │
│  ├── Admin: Privileged operations                               │
│  ├── Bridge: Cross-chain imports                                │
│  ├── Governance: DAO actions                                    │
│  └── System: Automated operations                               │
│                                                                 │
│   POLICY ENFORCEMENT                                            │
│  ├── Allowed transition matrix                                  │
│  ├── Rate limits per class                                      │
│  ├── Merkle tree verification                                   │
│  └── Zero-knowledge compliance proofs                           │
│                                                                 │
│   RECURSIVE PROOFS                                              │
│  ├── Nova folding scheme                                        │
│  ├── Constant 32-byte proofs                                    │
│  ├── Constant-time verification (16µs)                          │
│  └── Scale to millions of steps                                 │
│                                                                 │
│    LINEAGE COMMITMENTS                                          │
│  ├── Recursive hash chain                                       │
│  ├── Binding to genesis                                         │
│  ├── Tamper-proof history                                       │
│  └── Privacy-preserving                                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

##  **Architecture**

```
┌─────────────────────────────────────────────────────────────────┐
│                    SYSTEM ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    LAYER 1: ORIGIN                      │    │
│  │                    Classification                       │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐               │    │
│  │  │  User    │  │  Admin   │  │  Bridge  │               │    │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘               │    │ 
│  └───────┼─────────────┼──────────────┼─────────────────────┘   │
│          │             │              │                         │
│          ▼             ▼              ▼                         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              LAYER 2: POLICY ENGINE                      │   │
│  │  ┌─────────────────────────────────────────────────────┐ │   │
│  │  │  Allowed Transitions:                               │ │   │
│  │  │  • User → User:                                     │ │   │
│  │  │  • User → Admin:                                    │ │   │
│  │  │  • Admin → Bridge:                                  │ │   │
│  │  │  • Bridge → User:                                   │ │   │
│  │  │  • Rate Limits: Admin (10/day), Bridge (100/day)    │ │   │
│  │  └─────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────┘    │
│                          │                                      │
│                          ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              LAYER 3: LINEAGE COMMITMENT                │    │
│  │  C₀ = Hash(genesis_state, 0, 0)                         │    │
│  │  C₁ = Hash(C₀, transition₁, 1)                          │    │
│  │  C₂ = Hash(C₁, transition₂, 2)                          │    │
│  │  Cₙ = Hash(Cₙ₋₁, transitionₙ, n) → 32 bytes!            │    │
│  └─────────────────────────────────────────────────────────┘    │
│                          │                                      │
│                          ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              LAYER 4: NOVA RECURSION                     │   │
│  │  ┌─────────────────────────────────────────────────────┐ │   │
│  │  │  Step 1 → Fold                                      │ │   │
│  │  │  Step 2 → Fold → Final Verification → 32-byte proof │ │   │
│  │  │  Step n → Fold                                      │ │   │
│  │  └─────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────┘    │
│                          │                                      │
│                          ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              LAYER 5: RUST PROVER                        │   │
│  │  • 31,130 TPS throughput                                 │   │
│  │  • 885µs for 1000 steps                                  │   │
│  │  • 32-byte proofs                                        │   │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

##  **Benchmarks**

### **vs Industry Standards**

```
┌─────────────────────────────────────────────────────────────────┐
│                    PERFORMANCE COMPARISON                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  METRIC              │ ZK-ORIGIN    │ Industry     │ IMPROVEMENT│
│──────────────────────┼──────────────┼──────────────┼────────────│
│  Proof Size (10)     │ 32 bytes     │ 200-300b     │ 6-10x      │
│  Proof Size (100)    │ 32 bytes     │ 2-3KB        │ 62-94x     │
│  Proof Size (1000)   │ 32 bytes     │ 30-300KB     │ 937-9375x  │
│──────────────────────┼──────────────┼──────────────┼────────────│
│  Proving Time (10)   │ 54µs         │ 10ms         │ 185x       │
│  Proving Time (100)  │ 173µs        │ 100ms        │ 578x       │
│  Proving Time (500)  │ 328µs        │ 500ms        │ 1,524x     │
│  Proving Time (1000) │ 885µs        │ 1s+          │ 1,130x     │
│──────────────────────┼──────────────┼──────────────┼────────────│
│  Verification Time   │ 16-35µs      │ 10-50ms      │ 285-3125x  │
│──────────────────────┼──────────────┼──────────────┼────────────│
│  Throughput          │ 31,130 TPS   │ 100-2K TPS   │ 15-311x    │
│──────────────────────┼──────────────┼──────────────┼────────────│
│  Proves Validity     │ YES          │ YES          │ -          │
│  Proves Origin       │ YES          │ NO           │ NEW!       │
│  Constant Proofs     │ YES          │ NO           │ NEW!       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### **Detailed Benchmarks (Your Actual Results)**

```
┌─────────────────────────────────────────────────────────────────┐
│                    RAW BENCHMARK DATA                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  $ cargo run --bin zk-origin-cli -- benchmark                   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Benchmark: Prover Initialization                       │    │
│  │    100 initializations: 16.485ms                        │    │
│  │    Average: 164.859µs                                   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Benchmark: Add Transitions                             │    │
│  │    1000 transitions: 32.123ms                           │    │
│  │    Average per transition: 32.123µs                     │    │
│  │    Throughput: 31,130 TPS!                              │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Benchmark: Proof Generation                            │    │
│  │  Depth   10:   54.756µs  (proof size: 32 bytes)         │    │
│  │  Depth  100:  173.078µs  (proof size: 32 bytes)         │    │
│  │  Depth  500:  328.294µs  (proof size: 32 bytes)         │    │
│  │  Depth 1000:  885.224µs  (proof size: 32 bytes)         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Benchmark: Proof Verification                          │    │
│  │  Depth   10:   27.038µs (1000 verifications)            │    │
│  │  Depth  100:   21.578µs (1000 verifications)            │    │
│  │  Depth  500:   18.497µs (1000 verifications)            │    │
│  │  Depth 1000:   35.123µs (1000 verifications)            │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

##  **Installation**

### **Prerequisites**

```
┌─────────────────────────────────────────────────────────────────┐
│                    REQUIREMENTS                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Rust 1.70+                                                    │
│     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs   |
| | sh                                                            |
│                                                                 │
│   Circom 2.1+                                                   │
│     git clone https://github.com/iden3/circom.git               │
│     cd circom && cargo build --release                          │
│     cargo install --path circom                                 │
│                                                                 │
│   Node.js 18+ (for testing)                                     │
│     curl -fsSL https://deb.nodesource.com/setup_18.x            | 
|    sudo -E bash -                                               |
│     sudo apt-get install -y nodejs                              │
│                                                                 │
│   Git                                                           │
│     sudo apt-get install git                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### **Step 1: Clone Repository**

```bash
git clone https://github.com/ZKChainForge/zk-origin.git
cd zk-origin
```

### **Step 2: Build the Prover**

```bash
cd prover
cargo build --release
```

**Expected Output:**
```
   Compiling zk-origin-prover v0.1.0
    Finished release profile [optimized] target(s) in 2m 30s
```

### **Step 3: Verify Installation**

```bash
cargo run --bin zk-origin-cli -- --help
```

**Expected Output:**
```
---------------------------------------------------------------
                      ZK-ORIGIN CLI                            
         Zero-Knowledge State Lineage Verification             
---------------------------------------------------------------

USAGE:
    zk-origin-cli <COMMAND> [OPTIONS]

COMMANDS:
    demo        Run a demonstration of ZK-ORIGIN
    prove       Generate a lineage proof
    verify      Verify a lineage proof
    benchmark   Run performance benchmarks
    help        Show this help message
    version     Show version information
```

---

##  **Quick Start**

### **Run the Demo (5 Minutes)**

```bash
cargo run --bin zk-origin-cli -- demo
```

**Expected Output:**
```
---------------------------------------------------------------
                    ZK-ORIGIN DEMO                             
---------------------------------------------------------------

 Step 1: Creating Origin Policy 
   Policy created with 16 allowed transitions
   Epoch duration: 86400 seconds (24 hours)

 Step 2: Initializing Lineage Prover 
   Prover created successfully
   Genesis commitment: 17b0761f87b081d5...

 Step 3: Adding State Transitions 
   Transition 1: Genesis → User
   Transition 2: User → User
   Transition 3: User → User
  Current lineage depth: 3

 Step 4: Generating Lineage Proof 
   Proof generated successfully!
  Proof Details:
  Lineage depth: 3 transitions
  Proof size: 32 bytes
  Generation time: 25.475µs

 Step 5: Verifying Lineage Proof
   PROOF IS VALID!
  Verification Details:
  Genesis check:  PASSED
  Policy check:   PASSED
  Depth check:    PASSED
  Proof check:    PASSED
  Verification time: 158ns

 Step 6: Testing Policy Enforcement
   Valid: Genesis → User (allowed)
   Invalid: User → Admin (correctly rejected)
  Policy enforcement is working correctly!
```

---

##  **Usage Guide**

### **1. Generate a Proof**

```bash
# Generate proof for 100 transitions
cargo run --bin zk-origin-cli -- prove --output my_proof.json --steps 100
```

**Output:**
```
 Generating Lineage Proof
  Output: my_proof.json
  Transitions: 100

  Adding transitions... 100/100
 Proof Generated
   Saved to: my_proof.json
   Depth: 100 transitions
   Size: 32 bytes
   Time: 3.622416ms
```

### **2. Verify a Proof**

```bash
cargo run --bin zk-origin-cli -- verify --proof my_proof.json
```

**Output:**
```
 Verifying Lineage Proof
  Input: my_proof.json

  Proof loaded:
  Depth: 100 transitions
  Size: 32 bytes
  Lineage: d5ef709c0da63b17...

 Verification Result
   PROOF IS VALID!
   Verification time: 248ns
```

### **3. Run Benchmarks**

```bash
cargo run --bin zk-origin-cli -- benchmark
```

**Output:**
```
--------------------------------------------------------------------
                  ZK-ORIGIN BENCHMARKS                              
--------------------------------------------------------------------
 Benchmark: Prover Initialization 
  100 initializations: 16.485ms
  Average: 164.859µs

 Benchmark: Add Transitions 
  1000 transitions: 32.123ms
  Average per transition: 32.123µs
  Throughput: 31130 transitions/sec

 Benchmark: Proof Generation 
  Depth   10:   54.756µs  (proof size: 32 bytes)
  Depth  100:  173.078µs  (proof size: 32 bytes)
  Depth  500:  328.294µs  (proof size: 32 bytes)
  Depth 1000:  885.224µs  (proof size: 32 bytes)

 Benchmark: Proof Verification 
  Depth   10:   27.038µs (1000 verifications)
  Depth  100:   21.578µs (1000 verifications)
  Depth  500:   18.497µs (1000 verifications)
  Depth 1000:   35.123µs (1000 verifications)
```

---

##  **API Reference**

### **Rust API**

```rust
use zk_origin_prover::prelude::*;

// Create a prover
let policy = OriginPolicy::default();
let mut prover = LineageProver::new(policy)?;

// Add transitions
prover.add_transition(Transition {
    prev_state: [0u8; 32],
    new_state: [1u8; 32],
    origin: OriginClass::User,
    timestamp: 1234567890,
})?;

// Generate proof
let proof = prover.finalize()?;
assert_eq!(proof.size(), 32);

// Verify proof
assert!(proof.verify(&prover.verifier_key())?);
```

### **Core Types**

```rust
/// Origin classes for state transitions
#[derive(Clone, Copy, Debug)]
pub enum OriginClass {
    User = 0,      // Normal user transactions
    Admin = 1,     // Privileged operations  
    Bridge = 2,    // Cross-chain imports
    Governance = 3, // DAO-approved actions
    System = 4,    // Automated operations
    Emergency = 5, // Crisis interventions
}

/// A state transition with origin tracking
#[derive(Clone, Debug)]
pub struct Transition {
    pub prev_state: [u8; 32],
    pub new_state: [u8; 32],
    pub origin: OriginClass,
    pub timestamp: u64,
}

/// The final lineage proof (always 32 bytes!)
#[derive(Clone, Debug)]
pub struct LineageProof {
    proof: [u8; 32],
    depth: u64,
    lineage_commitment: [u8; 32],
    genesis: [u8; 32],
}
```

---

##  **Project Structure**

```
zk-origin/
│
├── README.md                    # This file
├── LICENSE                      # MIT License
│
├── prover/                      # Rust prover (main implementation)
│   ├── Cargo.toml               # Dependencies
│   ├── src/
│   │   ├── lib.rs               # Library root
│   │   ├── types/               # Core type definitions
│   │   │   ├── origin.rs        # OriginClass enum
│   │   │   ├── transition.rs    # Transition struct
│   │   │   ├── proof.rs         # LineageProof struct
│   │   │   └── error.rs         # Error types
│   │   │
│   │   ├── circuit/             # ZK circuit definitions
│   │   │   ├── step.rs          # Nova step circuit
│   │   │   ├── gadgets.rs       # Poseidon, Merkle gadgets
│   │   │   └── constraints.rs   # Constraint helpers
│   │   │
│   │   ├── prover/              # Prover implementation
│   │   │   ├── lineage_prover.rs # Main prover logic
│   │   │   ├── recursive.rs     # Nova recursion
│   │   │   └── compress.rs      # Proof compression
│   │   │
│   │   └── bin/                  # CLI binaries
│   │       └── zk-origin-cli.rs  # Main CLI
│   │
│   └── benches/                  # Benchmarks
│       └── performance.rs
│
├── circuits/                     # Circom circuits
│   ├── src/
│   │   ├── lineage_step.circom  # Main circuit
│   │   ├── poseidon.circom      # Poseidon hash
│   │   └── merkle.circom        # Merkle verification
│   └── test/                     # Circuit tests
│
├── contracts/                    # Solidity contracts (coming soon)
│   ├── Groth16Verifier.sol
│   └── LineageVerifier.sol
│
└── docs/                         # Documentation
    ├── architecture.md
    └── benchmarks.md
```

---

##  **Testing**

```bash
# Run all tests
cd prover
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_proof_generation

# Run benchmarks
cargo bench
```

---

##  **Contributing**

We welcome contributions! Here's how:

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Commit your changes**
   ```bash
   git commit -m 'Add amazing feature'
   ```
4. **Push to the branch**
   ```bash
   git push origin feature/amazing-feature
   ```
5. **Open a Pull Request**

### **Development Guidelines**

- Write tests for new features
- Update documentation
- Follow Rust style guide
- Run `cargo fmt` before committing
- Ensure all tests pass: `cargo test`

---

##  **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

##  **Acknowledgments**

- **Nova Team** for the incredible folding scheme
- **Circom** for the circuit compiler
- **Mina Protocol** for recursive proof inspiration
- **Zcash** for pioneering ZK technology
- All ZK researchers pushing the field forward

---

##  **Contact**

- Twitter: [@zkorigin](https://x.com/zkchain_z41420)
- LinkedIn: [ZK-ORIGIN](https://linkedin.com/in/vikram-a-a6a252395)
- Email: [zkchainforge](mailto:zkchainforge@gmail.com)
- GitHub: [ZKChainForge/zk-origin](https://github.com/ZKChainForge/zk-origin)

---

##  **Star History**

If you find this project useful, please consider giving it a star on GitHub! It helps others discover it.

---

<div align="center">
  <sub>Built with  by [VIKRAM A]</sub>
  <br>
  <sub>Copyright © 2026 ZK-ORIGIN Contributors</sub>
</div>
```
