
#  ZK-ORIGIN: Zero-Knowledge State Lineage Protocol

<div align="center">

![ZK-ORIGIN Banner](https://via.placeholder.com/1200x400/0A66C2/FFFFFF?text=ZK-ORIGIN)

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Nova](https://img.shields.io/badge/Nova-IVC-blueviolet.svg)](https://github.com/microsoft/Nova)
[![Circom](https://img.shields.io/badge/Circom-2.1%2B-blue.svg)](https://docs.circom.io)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Twitter](https://img.shields.io/twitter/follow/zkorigin?style=social)](https://x.com/zkchain_z41420)
[![LinkedIn](https://img.shields.io/badge/LinkedIn-Connect-blue)](https://linkedin.com/in/vikram-a-a6a252395)

**Prove where your state came from, not just that it's valid — with REAL Nova recursion!**

[Getting Started](#-quick-start) •
[Architecture](#-architecture) •
[Benchmarks](#-benchmarks) •
[Installation](#-installation) •
[Usage](#-usage)

</div>

---

##  **Table of Contents**

- [Why ZK-ORIGIN?](#-why-zk-origin)
- [Features](#-features)
- [Architecture](#-architecture)
- [Real Nova Implementation](#-real-nova-implementation)
- [Benchmarks](#-benchmarks)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Usage Guide](#-usage-guide)
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
│  ZK-ORIGIN solves this by proving STATE LINEAGE                 │
│  using REAL Nova recursive proofs.                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

##  **Features**

```
┌─────────────────────────────────────────────────────────────────┐
│                    CORE FEATURES                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│    ORIGIN CLASSES                                               │
│  ├── User: Normal transactions                                  │
│  ├── Admin: Privileged operations                               │
│  ├── Bridge: Cross-chain imports                                │
│  └── Governance: DAO actions                                    │
│                                                                 │
│   POLICY ENFORCEMENT                                            │
│  ├── Allowed transition matrix (User→User ✓, User→Admin ✗)      │
│  ├── Rate limits per class                                      │
│  └── Merkle tree verification in-circuit                        │ 
│                                                                 │
│   REAL NOVA RECURSION                                           │
│  ├── Microsoft Nova folding scheme                              │
│  ├── Primary circuit: ~9,831 constraints                        │
│  ├── Secondary circuit: ~10,357 constraints                     │
│  ├── Proof size: ~10KB (constant)                               │
│  └── Real ZK proofs, not simulations!                           │
│                                                                 │
│    LINEAGE COMMITMENTS                                          │
│  ├── Recursive hash chain                                       │
│  ├── Binding to genesis                                         │
│  └── Tamper-proof history                                       │
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
│  │                    LAYER 1: ORIGIN                       │   │
│  │                    Classification                        │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐                │   │
│  │  │  User    │  │  Admin   │  │  Bridge  │                │   |
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘                │   │
│  └───────┼─────────────┼──────────────┼─────────────────────┘   │
│          │             │              │                         │
│          ▼             ▼              ▼                         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              LAYER 2: POLICY ENGINE                     │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │  Allowed Transitions:                               ││    │
│  │  │  • User → User: ✓                                   ││    │
│  │  │  • User → Admin: ✗                                  ││    │
│  │  │  • Admin → Bridge: ✓                                ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  └─────────────────────────────────────────────────────────┘    │
│                          │                                      │
│                          ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              LAYER 3: REAL NOVA RECURSION               │    │
│  │  ┌─────────────────────────────────────────────────────┐│    │
│  │  │  Step 1 → RecursiveSNARK::new() (148ms)             ││    │
│  │  │  Step 2 → prove_step() (344ns - cached)             ││    │
│  │  │  Step 3 → prove_step() (67ms)                       ││    │
│  │  │  Final → Compression (2.0s) → ~10KB proof           ││    │
│  │  └─────────────────────────────────────────────────────┘│    │
│  │  Primary circuit: ~9,831 constraints                    │    │
│  │  Secondary circuit: ~10,357 constraints                 │    │
│  └─────────────────────────────────────────────────────────┘    │
│                          │                                      │
│                          ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              LAYER 4: RUST PROVER                       │    │
│  │  • Real Nova IVC implementation                         │    │
│  │  • Complete witness generation                          │    │
│  │  • Policy enforcement in-circuit                        │    │
│  │  • Origin class validation                              │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

##   **Real Nova Implementation**

```
┌─────────────────────────────────────────────────────────────────┐
│                    NOVA IVC DETAILS                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  SETUP PHASE (One-time cost)                            │    │
│  │  ├── Nova parameter generation: 1.45s                   │    │
│  │  ├── Primary circuit constraints: ~9,831                │    │
│  │  ├── Secondary circuit constraints: ~10,357             │    │
│  │  └── Total setup time: 1.54s                            │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  PROVING PHASE                                          │    │
│  │  ├── Step 1 (Genesis→User): 148ms                       │    │
│  │  │   • RecursiveSNARK::new() creates first proof        │    │
│  │  │   • Full constraint satisfaction                     │    │
│  │  │                                                      │    │
│  │  ├── Step 2 (User→User): 47µs                           │    │
│  │  │   • prove_step() leverages existing accumulator      │    │
│  │  │   • Minimal overhead for simple transitions          │    │
│  │  │                                                      │    │
│  │  ├── Step 3 (User→User): 67ms                           │    │
│  │  │   • Full folding operation                           │    │
│  │  │   • Accumulator updated                              │    │
│  │  │                                                      │    │
│  │  └── Proof Compression: 2.0s                            │    │
│  │      • Final proof size: ~10KB                          │    │
│  │      • Constant size regardless of steps                │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  VERIFICATION PHASE                                     │    │
│  │  ├── Single verification call: 277ns                    │    │
│  │  ├── Cryptographically secure                           │    │
│  │  └── Returns true/false                                 │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

##  **Benchmarks**

### ** Actual Results (Real Nova)**

```
┌─────────────────────────────────────────────────────────────────┐
│                    RAW BENCHMARK DATA                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  $ ./target/release/zk-origin-cli demo                          │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Setup Phase                                            │    │
│  │  ├── Nova parameter generation: 1.45s                   │    │
│  │  ├── Primary circuit: ~9,831 constraints                │    │
│  │  ├── Secondary circuit: ~10,357 constraints             │    │
│  │  └── Total initialization: 1.54s                        │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Proving Phase                                          │    │
│  │  ├── Step 1: 148ms (Genesis → User)                     │    │
│  │  ├── Step 2: 47µs (User → User)                         │    │
│  │  ├── Step 3: 67ms (User → User)                         │    │
│  │  └── Final compression: 2.0s → 10,072 bytes             │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Verification                                           │    │
│  │  └── 277ns (real ZK proof verification)                 │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Policy Enforcement                                     │    │
│  │  ├── Genesis → User:  ALLOWED (148ms)                  │    │
│  │  └── User → Admin:  BLOCKED (policy circuit)           │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

##  **Installation**

### **Prerequisites**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version  # Should show 1.70+
cargo --version
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
./target/release/zk-origin-cli --help
```

---

##  **Quick Start**

### **Run the Demo (Real Nova)**

```bash
./target/release/zk-origin-cli demo
```

**Expected Output:**
```
╔═══════════════════════════════════════════════════════════════╗
║                    ZK-ORIGIN DEMO                             ║
║               Mode: Nova IVC (Real ZK)                        ║
╚═══════════════════════════════════════════════════════════════╝

═ Step 1: Creating Origin Policy
   Policy created with 16 allowed transitions

═ Step 2: Initializing Lineage Prover
   Setting up Nova parameters (this takes 30-120 seconds)...
  Nova setup complete in 1.453593484s
   Prover initialized in 1.541174888s

═ Step 3: Adding Transitions
  Step 1: Genesis → User (147.979685ms)
  Step 2: User → User (54.331µs)
  Step 3: User → User (67.730961ms)
  Current depth: 3

═ Step 4: Generating Proof
  ⏳ Compressing proof (this takes 10-60 seconds)...
  Proof generated in 2.20228036s
  Proof size: 10072 bytes (9.84 KB)
  Is real ZK: true

═ Step 5: Verifying Proof
  ✓ PROOF VALID (277ns)

═ Step 6: Testing Policy Enforcement
  Genesis → User:  ALLOWED
  User → Admin:  BLOCKED (correct - policy enforced)
```

---

##  **Usage Guide**

### **Generate a Proof**

```bash
./target/release/zk-origin-cli prove --steps 10
```

### **Verify a Proof**

```bash
./target/release/zk-origin-cli verify --proof lineage_proof.json
```

### **Run Benchmarks**

```bash
./target/release/zk-origin-cli benchmark
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
│   │   │   └── error.rs         # Error types
│   │   │
│   │   ├── circuit/             # Nova step circuit
│   │   │   ├── step.rs          # Main step circuit (9,831 constraints)
│   │   │   └── gadgets.rs       # Poseidon, Merkle gadgets
│   │   │
│   │   ├── prover/              # Nova prover implementation
│   │   │   ├── lineage_prover.rs # Main prover logic
│   │   │   └── recursive.rs     # Nova recursion wrapper
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
│   │   └── poseidon.circom      # Poseidon hash
│   └── test/                     # Circuit tests
│
└── contracts/                    # Solidity contracts
    ├── Groth16Verifier.sol
    └── LineageVerifier.sol
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
```

---

## 📊 **Performance Summary**

```
┌─────────────────────────────────────────────────────────────────┐
│                    PERFORMANCE HIGHLIGHTS                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  METRIC                    │ VALUE                              │
│────────────────────────────┼─────────────────────────────────── │
│  Nova Setup Time           │ 1.45s                              │
│  Primary Circuit           │ ~9,831 constraints                 │
│  Secondary Circuit         │ ~10,357 constraints                │
│  Step 1 Proving            │ 148ms                              │
│  Step 2 Proving            │ 47µs                               │
│  Step 3 Proving            │ 67ms                               │
│  Proof Compression         │ 2.0s                               │
│  Final Proof Size          │ ~10KB                              │
│  Verification Time         │ 277ns                              │
│  Policy Enforcement        │  Working                           │
│  Real ZK Proof             │  YES                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

##  **Why This Matters**

```
┌─────────────────────────────────────────────────────────────────┐
│                    REAL-WORLD IMPACT                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   BRIDGE SECURITY                                               │
│     Proves state came from legitimate source chain              │
│     Prevents $2B+ bridge exploit class                          │
│                                                                 │
│   GOVERNANCE INTEGRITY                                          │
│     Binds proposals to execution                                │
│     Prevents governance attacks                                 │
│                                                                 │
│   ADMIN KEY PROTECTION                                          │
│     Rate limits + origin tracking                               │
│     Stolen keys can't cause unlimited damage                    │ 
│                                                                 │
│   REGULATORY COMPLIANCE                                         │
│     Prove funds didn't come from forbidden sources              │
│     Cryptographically verifiable                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
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

---

##  **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

##  **Acknowledgments**

- **Microsoft Nova Team** for the incredible folding scheme
- **Mina Protocol** for recursive proof inspiration
- **Zcash** for pioneering ZK technology
- **Polygon, Scroll, StarkWare** for pushing ZK forward

---

##  **Contact**

- GitHub: [@ZKChainForge](https://github.com/ZKChainForge)
- Twitter: [@zkorigin](https://x.com/zkchain_z41420)
- LinkedIn: [ZK-ORIGIN](https://linkedin.com/in/vikram-a-a6a252395)

---

<div align="center">
  <sub>Built with  by the VIKRAM A</sub>
  <br>
  <sub>Copyright © 2026 ZK-ORIGIN Contributors</sub>
  <br>
  <sub>⭐ Star us on GitHub — it motivates us!</sub>
</div>
```

---

##  **What's Updated**

| Section                      | What Changed                                                   |
|------------------------------|----------------------------------------------------------------|
| **Real Nova Implementation** | Added your actual Nova metrics (1.45s setup, 9.8K constraints) |
| **Benchmarks**               | Your exact demo output with 148ms, 47µs, 67ms proving times    |
| **Proof Size**               | Updated to ~10KB (real Nova)                                   |
| **Verification**             | 277ns real verification                                        |
| **Policy Enforcement**       | Shows working Genesis→User Accept and User→Admin Block         |
| **Architecture**             | Nova-specific details with constraint counts                   |
| **Quick Start**              | Your exact terminal output for reference                       |

