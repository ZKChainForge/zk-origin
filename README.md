
**Zero-Knowledge State Lineage Protocol**  
*Prove where your state came from, not just that it's valid — with real Nova recursion.*

![GitHub](https://img.shields.io/badge/Rust-1.70%2B-orange) ![GitHub](https://img.shields.io/badge/Nova-IVC-blueviolet) ![GitHub](https://img.shields.io/badge/license-MIT-green)

---

##  **Why I Built This**

Every ZK system today answers one question: *"Is this state valid?"*

But none can answer: *"Where did this state come from?"*

This gap has caused:
- **$2B+** in bridge exploits
- **$500M+** in governance attacks
- **$1B+** in admin key compromises

I built ZK-ORIGIN to solve this — proving state lineage using real Nova recursive proofs.

---

##  **What It Does**

- **Origin Classes** — Tag every transition as User, Admin, Bridge, or Governance  
- **Policy Enforcement** — User→User allowed, User→Admin blocked (in-circuit)  
- **Real Nova Recursion** — Microsoft's folding scheme, implemented from scratch  
- **Lineage Commitments** — Each state carries its entire ancestry in one hash  
- **Two Modes** — Fast dev mode + real ZK mode for production  

---

##  **How It Works**

```
Layer 1: Origin Classes
├── User
├── Admin
├── Bridge
└── Governance

Layer 2: Policy Matrix (Merkle tree enforced)
├── User → User: ✓
├── User → Admin: ✗
└── Admin → Bridge: ✓

Layer 3: Recursive Commitments
├── C₀ = Hash(genesis)
└── Cₙ = Hash(Cₙ₋₁, transition, depth)

Layer 4: Nova Folding
├── Step 1 → RecursiveSNARK::new()
├── Step 2 → prove_step() (cached)
├── Step 3 → prove_step()
└── Final → compressed proof (constant size)
```

---

##  **Benchmarks**

These are real numbers from my laptop, running actual Nova proofs:

```
Nova Setup (one-time)
├── Parameter generation: 1.26s
├── Primary circuit: ~9,831 constraints
└── Secondary circuit: ~10,357 constraints

Proving Performance
├── Step 1 (Genesis→User): 123.9ms
├── Step 2 (User→User): 437ns (cached!)
├── Step 3 (User→User): 68.5ms
├── Step 4 (User→User): 76.3ms
└── Step 5 (User→User): 81.4ms

Proof Generation
├── Compression time: 1.87–1.98s
└── Final proof size: 10,072 bytes (constant)

Verification
├── Structural: 2.8µs
├── Full ZK verification: 108–124ms
└── Total with deserialization: ~500ms

Throughput
└── ~14 transitions/second (real ZK mode)
```

**Key takeaway:** Proof size stays the same no matter how many steps. That's the magic of recursion.

---

##  **Tech Stack**

- **Rust** — High-performance prover  
- **Nova** — Microsoft Research folding scheme  
- **Circom** — Circuit compiler (1.4k constraints for policy)  
- **Solidity** — On-chain verifiers  
- **Hardhat** — Local deployment  

---

##  **Quick Start**

```bash
# Clone the repo
git clone https://github.com/ZKChainForge/zk-origin.git
cd zk-origin/prover

# Build (release mode for real performance)
cargo build --release

# Run the demo
./target/release/zk-origin-cli demo

# Run benchmarks
./target/release/zk-origin-cli benchmark
```

---

##  **Demo Output**

Here's what you'll see when you run the demo:

```
╔═══════════════════════════════════════════════════════════════╗
║                    ZK-ORIGIN DEMO                             ║
║               Mode: Nova IVC (Real ZK)                      ║
╚═══════════════════════════════════════════════════════════════╝

Step 1: Creating Origin Policy
   Policy created with 16 allowed transitions

Step 2: Initializing Lineage Prover
   Nova setup complete in 1.26s
   Prover initialized in 1.33s

Step 3: Adding Transitions
   Step 1: Genesis → User (123.9ms)
   Step 2: User → User (437ns)
   Step 3: User → User (68.5ms)
   Current depth: 3

Step 4: Generating Proof
   Proof generated in 1.94s
   Proof size: 10072 bytes (9.84 KB)
   Is real ZK: true

Step 5: Verifying Proof
   ✓ ZK Verification PASSED in 108ms
   Proof size: 10072 bytes
   Depth: 3 steps

Step 6: Testing Policy Enforcement
   Genesis → User: ALLOWED
   User → Admin: BLOCKED


```

---

##  **Project Structure**

```
zk-origin/
│
├── README.md
├── LICENSE
│
├── prover/                # Rust prover
│   ├── src/
│   │   ├── types/         # Core types (OriginClass, Transition, etc.)
│   │   ├── circuit/       # Nova step circuit
│   │   ├── prover/        # Prover implementation
│   │   └── bin/           # CLI
│   └── benches/           # Benchmarks
│
├── circuits/               # Circom circuits
│   └── src/
│       ├── lineage_step.circom
│       └── poseidon.circom
│
└── contracts/              # Solidity contracts
    ├── Groth16Verifier.sol
    └── LineageVerifier.sol
```

---

##  **Testing**

```bash
cd prover
cargo test                 # Run all tests
cargo test -- --nocapture  # Show output
cargo bench                # Run benchmarks
```

---

##  **Why This Matters**

- **Bridge Security** — Prove state came from a legitimate source chain  
- **Governance Integrity** — Bind proposals to execution  
- **Admin Key Protection** — Rate limits + origin tracking  
- **Regulatory Compliance** — Cryptographically verifiable provenance  

---

##  **Contributing**

I welcome contributions. Here's how:

1. Fork the repo  
2. Create a feature branch  
3. Commit your changes  
4. Push and open a PR  

---

##  **License**

Apache-2.0 license — use it, build on it, share it.

---

##  **Acknowledgments**

- **Microsoft Nova Team** — For the folding scheme  
- **Mina Protocol** — Recursive proof inspiration  
- **Zcash** — ZK pioneers  
- Everyone who followed this journey — your feedback kept me going  

---

##  **Contact**

- GitHub: [@ZKChainForge](https://github.com/ZKChainForge)  
- Project repo: [github.com/ZKChainForge/zk-origin](https://github.com/ZKChainForge/zk-origin)  

---

*Built with  by Vikram*  
*March 2026*