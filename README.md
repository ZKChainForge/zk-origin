# ZK-ORIGIN: Zero-Knowledge State Lineage Verification Protocol

![ZK-ORIGIN Banner](https://img.shields.io/badge/ZK--ORIGIN-Zero--Knowledge_State_Lineage-blue)
![GitHub License](https://img.shields.io/badge/license-MIT-green)
![Status-Development](https://img.shields.io/badge/Status-In%20Development-orange)
![WIP](https://img.shields.io/badge/WIP-Weeks%201--2%20(Foundations)-yellow)

##  Project Overview

**ZK-ORIGIN** is a novel zero-knowledge primitive under active development that addresses a critical security gap: while current ZK systems prove **state validity**, they cannot prove **state legitimacy**. This project aims to implement cryptographic lineage tracking to prevent $2B+ in bridge exploits, governance attacks, and admin key compromises.

> **Current Status**: Week 1-2 of 6-week development plan. Building foundational circuits in Circom.

### The Core Problem
```
Current ZK Proof says:
"Is this state mathematically valid?"

Current ZK Proof CANNOT say:
"Did this state come from legitimate origins?"

This gap = $2B+ in real exploits
```

##  Development Timeline (6 Weeks)

###  **WEEK 1-2: CIRCUIT FOUNDATION** (IN PROGRESS)
**Goal**: Build Circom circuits for lineage verification
- [ ] Poseidon hash implementation (ZK-friendly hashing)
- [ ] Merkle tree verification for policy enforcement
- [ ] Origin class logic (User/Admin/Bridge/Governance)
- [ ] Lineage commitment chaining
- [ ] Rate limit counter system

###  **WEEK 3-4: RECURSIVE PROVING**
**Goal**: Implement Nova folding scheme for constant-size proofs
- [ ] Nova prover in Rust
- [ ] Recursive proof compression
- [ ] Performance benchmarking
- [ ] Memory optimization

### **WEEK 5-6: ON-CHAIN & DEPLOYMENT**
**Goal**: Deploy to Sepolia testnet with complete demo
- [ ] Solidity verifier contracts
- [ ] Sepolia testnet deployment
- [ ] Interactive CLI demo
- [ ] Full documentation

##  Project Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    ZK-ORIGIN ARCHITECTURE (PLANNED)             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐      │
│  │   Circom    │    │     Rust    │    │    Solidity     │      │
│  │  Circuits   │────▶│  Nova Prover│────▶│   Verifier    │      │
│  │ (Week 1-2)  │    │ (Week 3-4)  │    │ (Week 5-6)      │      │
│  └─────────────┘    └─────────────┘    └─────────────────┘      │
│         │                    │                    │             │
│         ▼                    ▼                    ▼             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐      │
│  │ Policy      │    │ Lineage     │    │ Sepolia         │      │
│  │ Enforcement │    │ Compression │    │ Testnet         │      │
│  │ (Merkle)    │    │ (O(1) size) │    │ (Target)        │      |
│  └─────────────┘    └─────────────┘    └─────────────────┘      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

##  Key Innovations

### 1. **Origin Class System**
```rust
enum OriginClass {
    User,      // Normal transactions
    Admin,     // Privileged operations
    Bridge,    // Cross-chain imports
    Governance // DAO-approved actions
}
```

### 2. **Policy Enforcement Matrix**
```
Allowed Transitions:
Genesis →  User ✓    User → User ✓     Admin → User ✓
Genesis →  Admin ✓   User → Admin ✗    Admin → Bridge ✓
Bridge  →  User ✓    Bridge → Admin ✗
```

### 3. **Constant-Size Proofs via Nova**
- **Without Nova**: 1000 steps = 300KB proof
- **With Nova**: 1000 steps = 312 byte proof
- **Folding scheme**: 4x more efficient than traditional recursion

## Project Structure (Current)

```
zk-origin/
├── README.md                 # This file
├── ARCHITECTURE.md           # Technical design document
├── ROADMAP.md                # 6-week development plan
├── circuits/                 # Circom circuits (Week 1-2)
│   ├── package.json
│   ├── src/
│   │   ├── main/
│   │   │   └── lineage_step.circom    # Main step circuit
│   │   ├── lib/
│   │   │   ├── poseidon.circom        # Poseidon hash
│   │   │   ├── merkle.circom          # Merkle verification
│   │   │   └── comparators.circom     # Range checks
│   │   └── utils/
│   │       └── constants.circom       # Circuit constants
│   └── test/                # Circuit tests
│
├── docs/                    # Documentation
│   ├── Week1-2-Plan.md      # Detailed week 1-2 tasks
│   └── Theory/              # Mathematical foundations
│
└── scripts/                 # Build and test scripts
    └── setup.sh             # Initial setup
```

##  Getting Started (Development)

### Prerequisites
```bash
# Install Circom 2.1+
curl -o- https://raw.githubusercontent.com/iden3/circom/master/install.sh | bash

# Install Node.js (v18+)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18

# Install Rust (for later weeks)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Installation (Current Setup)
```bash
# Clone repository
git clone https://github.com/ZKChainForge/zk-origin.git
cd zk-origin

# Setup circuits
cd circuits
npm install
```

### Build & Test Circuits (Week 1-2 Focus)
```bash
# Compile main circuit
npx circom src/main/lineage_step.circom --r1cs --wasm --sym

# Generate witness
node generate_witness.js lineage_step.wasm input.json witness.wtns

# Setup and generate proof
snarkjs groth16 setup lineage_step.r1cs pot12_final.ptau circuit_0000.zkey
snarkjs groth16 prove circuit_0000.zkey witness.wtns proof.json public.json

# Verify proof
snarkjs groth16 verify verification_key.json public.json proof.json
```

##  Technical Details (Week 1-2)

### Circuit Design
```circom
template LineageVerificationCircuit(MERKLE_DEPTH, NUM_ORIGIN_CLASSES) {
    // Public inputs
    signal input prev_state_hash;
    signal input new_state_hash;
    signal input policy_root;
    
    // Private inputs
    signal private input prev_origin_class;
    signal private input new_origin_class;
    signal private input policy_proof[MERKLE_DEPTH];
    
    // Constraints
    // 1. Origin class validity
    // 2. Policy Merkle proof verification
    // 3. Rate limit checks
    // 4. Lineage commitment update
}
```

### Constraint Estimates
| Component | Estimated Constraints |
|-----------|---------------------- |
| Poseidon Hash (width 3)| ~300     |
| Merkle Proof (depth 4) | ~1,280   |
| Origin Validation      | ~20      |
| Total (Step Circuit)   | ~3,000   |

## 📖 Learning Resources

### For Week 1-2 (Circuit Development)
1. **[Circom 2 Documentation](https://docs.circom.io/)** - Official Circom guide
2. **[circomlib](https://github.com/iden3/circomlib)** - Standard circuit library
3. **[Poseidon Paper](https://eprint.iacr.org/2019/458)** - ZK-friendly hash function
4. **[0xPARC Circom Tutorials](https://github.com/0xPARC/circom-workshop)** - Excellent learning materials

### For Future Weeks
5. **[Nova Paper](https://eprint.iacr.org/2021/370)** - Recursive folding scheme
6. **[SNARKs for C Programmers](https://vitalik.ca/general/2021/01/26/snarks.html)** - Vitalik's ZK explanation
7. **[Proofs, Arguments, and Zero-Knowledge](https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.html)** - Formal foundations

##  Project Goals & Applications

### Primary Goal: Job-Ready Portfolio Project
This project is designed to demonstrate:
- **Deep ZK understanding** beyond tutorials
- **Full-stack implementation** (circuits → prover → verifier)
- **Production thinking** (testnet deployment, gas optimization)
- **Problem-solving** (novel solution to real security gap)
- **Communication skills** (documentation, demos, social proof)

### Target Applications
1. **Bridge Security**: Prove cross-chain states came from legitimate sources
2. **Governance Integrity**: Ensure proposal execution matches approval
3. **Admin Key Protection**: Rate-limit and audit privileged operations
4. **Regulatory Compliance**: Prove fund origins without revealing details

## Testing Strategy

### Week 1-2 Testing Plan
```javascript
// Sample test structure for circuits
describe('LineageStepCircuit', () => {
    it('should accept valid User → User transition', async () => {
        // Test valid policy path
    });
    
    it('should reject User → Admin transition', async () => {
        // Test policy violation
    });
    
    it('should enforce rate limits', async () => {
        // Test counter system
    });
});
```

##  Contributing (Future)

Once the foundation is complete, contributions will be welcome! Planned contribution areas:
- Additional proof systems (PLONK, Halo2 support)
- Multi-chain deployments
- Formal verification
- Performance optimizations

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

##  Developer

**VIKRAM A** - 3rd Year Cyber Security Student
- **Current Focus**: ZK Protocol Development
- **Goal**: ZK Engineer / Cryptography Engineer position (part-time or Full-time remote)
- **Timeline**: Building this project over 6 weeks to demonstrate skills
- **Contact**: [GitHub](https://github.com/ZKChainForge/zk-origin) | [Twitter](https://x.com/zkchain_z41420) | [LinkedIn](https://www.linkedin.com/in/vikram-a-a6a252395)

##  Follow the Journey

### Weekly Updates
- **LinkedIn**: Technical deep-dives and progress updates
- **Twitter**: Daily development insights and learnings
- **GitHub**: Code commits and project milestones

### Week 1-2 Focus (Current)
-  Day 1-2: Circom fundamentals and environment setup
-  Day 3-4: Poseidon hash implementation
-  Day 5-6: Merkle tree circuit design
-  Day 7: Integration planning and testing

##  Success Metrics

### Technical Goals
- [ ] Week 1-2: Working lineage step circuit with policy enforcement
- [ ] Week 3-4: Nova recursive prover with constant-size proofs
- [ ] Week 5-6: Deployed on Sepolia with demo CLI
- [ ] Final: Job offers in ZK/cryptography space

### Learning Outcomes
- Master Circom circuit design
- Understand Nova folding and recursive proofs
- Gain production ZK deployment experience
- Build comprehensive portfolio project

---

<p align="center">
  <strong>Building ZK-ORIGIN: A 6-week journey from concept to testnet</strong><br>
  <sub>Week 1-2: Circuit Foundations | Follow along as we build!</sub>
</p>

<div align="center">
  
  **Follow Development**: 
  [![X](https://x.com/zkchain_z41420)]
  [![GitHub](https://img.shields.io/github/stars/ZKChainForge/zk-origin?style=social)](https://github.com/ZKChainForge/zk-origin)

</div>