
# ZK-ORIGIN: Zero-Knowledge State Provenance Protocol

<div align="center">

![ZK-ORIGIN Banner](https://img.shields.io/badge/ZK--ORIGIN-State%20Provenance-blue?style=for-the-badge)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solidity](https://img.shields.io/badge/Solidity-0.8.19-363636?logo=solidity)](https://soliditylang.org/)
[![Circom](https://img.shields.io/badge/Circom-2.1.0-orange)](https://docs.circom.io/)
[![Hardhat](https://img.shields.io/badge/Hardhat-2.19-yellow)](https://hardhat.org/)

**Cryptographic proof of where your blockchain state came from.**

[Demo](#-demo) • [Installation](#-installation) • [How It Works](#-how-it-works) • [Benchmarks](#-benchmarks) • [Documentation](#-documentation)

</div>

---

##  The Problem

Current ZK systems prove **state validity** but not **state legitimacy**.

```
 ZK Rollups prove:    "This state transition is mathematically valid"
 ZK Rollups DON'T prove: "This state came from an authorized source"
```

### Real Attack Scenarios
ZK-ORIGIN Security Theory

ZK-ORIGIN operates on the principle that every critical blockchain action must have a provable and traceable origin. Instead of reacting to exploits, it enforces cryptographic accountability at the protocol level.

1. Bridge Exploit Prevention Theory

Bridge exploits have caused over $2B+ in losses.
ZK-ORIGIN prevents these attacks by requiring proof of cross-chain origin, ensuring that assets and messages are cryptographically verified before execution.

2. Privileged Access Control Theory

Admin key compromises account for $1B+ in losses.
ZK-ORIGIN introduces verifiable tracking of privileged operations, ensuring that sensitive actions are provably authorized and auditable.

3. Governance Integrity Theory

Governance attacks have led to $500M+ in losses.
ZK-ORIGIN binds governance proposals directly to the resulting code changes, preventing malicious proposal substitution or hidden execution logic.

4. Upgrade Lineage Verification Theory

Upgrade backdoors present unknown but critical systemic risk.
ZK-ORIGIN verifies the entire upgrade lineage, ensuring that every contract update is cryptographically linked to its approved governance process.


---

##  The Solution

ZK-ORIGIN adds **origin class tracking** with **policy enforcement** in zero-knowledge:

```
┌─────────────┐     ┌──────────────┐     ┌────────────────┐
│   State     │────▶│   Origin     │────▶│    Policy      │
│ Transition  │     │Classification│     │  Enforcement   │
└─────────────┘     └──────────────┘     └────────────────┘
                           │                     │
                           ▼                     ▼
                    ┌──────────────┐     ┌────────────────┐
                    │   Lineage    │────▶│    ZK Proof    │
                    │  Commitment  │     │   Generation   │
                    └──────────────┘     └────────────────┘
```

### Origin Classes

| Origin      | Description           | Example                |
|-------------|-----------------------|------------------------|
| **Genesis** | System initialization | Protocol deployment    |
| **User**    | Normal transactions   | Token transfers, swaps |
| **Admin**   | Privileged operations | Parameter updates      |
| **Bridge**  | Cross-chain imports   | Asset bridging         |

### Policy Matrix

```
           To →    User    Admin    Bridge
From ↓           
Genesis           YES       YES       NO
User              YES       NO        NO
Admin             YES       YES       YES
Bridge            YES       NO        NO
```

**Key Security Property:** Users cannot escalate to Admin privileges. Bridges cannot inject Admin states.

---

##  Demo

```bash
./scripts/run-demo.sh
```

### Output

```
 ZK-ORIGIN Demo
Demonstrating zero-knowledge state provenance verification

Scenario: DeFi Protocol Lifecycle

 Protocol Deployment - Proof generated in 454ms
     New lineage: 0x1d42394b15f5620c...
     Depth: 1

 Open for Users - Proof generated in 223ms
     New lineage: 0x185ca4589009db45...
     Depth: 2

 User Activity - Proof generated in 221ms
     New lineage: 0x1f4d884324d19cc3...
     Depth: 3

 More Activity - Proof generated in 217ms
     New lineage: 0x0f2ca5a180b58500...
     Depth: 4

 All proofs verified successfully

Attack Simulation:
 Attack blocked by policy!
  User → Admin privilege escalation rejected

 ZK-ORIGIN successfully protects against unauthorized state origins!
```

---

##  Benchmarks

| Metric                    | Value             |
|---------------------------|-------------------|
| **Circuit Constraints**   | ~1,500            |
| **Proof Generation**      | 217-454ms         |
| **Proof Verification**    | <10ms (off-chain) |
| **On-chain Verification** | ~200k gas         |
| **Proof Size**            | ~256 bytes        |

### Performance Breakdown

| Component                 | Constraints | % of Total |
|---------------------------|-------------|------------|
| Policy Check              | ~400        | 27%        |
| Poseidon Hash (transition)| ~300        | 20%        |
| Poseidon Hash (lineage)   | ~300        | 20%        |
| Origin Validation         | ~200        | 13%        |
| Other                     | ~300        | 20%        |

---

##  Installation

### Prerequisites

- Node.js 18+
- npm or yarn
- Circom 2.1+ (`npm install -g circom`)

### Setup

```bash
# Clone the repository
git clone https://github.com/ZKChainForge/zk-origin.git
cd zk-origin

# Install dependencies
npm install

# Compile circuits
cd circuits
npm install
./scripts/compile.sh lineage_step_simple
./scripts/setup.sh lineage_step_simple

# Compile contracts
cd ../contracts
npm install
npx hardhat compile

# Run demo
cd ..
./scripts/run-demo.sh
```

---

##  Project Structure

```
zk-origin/
├── circuits/                    # Circom ZK circuits
│   ├── src/
│   │   ├── main/
│   │   │   └── lineage_step_simple.circom
│   │   └── lib/
│   │       ├── poseidon.circom
│   │       ├── merkle.circom
│   │       └── comparators.circom
│   ├── build/                   # Compiled outputs
│   │   ├── lineage_step_simple.zkey
│   │   └── verification_key.json
│   └── test/
│
├── contracts/                   # Solidity contracts
│   ├── contracts/
│   │   ├── LineageVerifier.sol
│   │   ├── Groth16Verifier.sol
│   │   └── PolicyRegistry.sol
│   └── test/
│
├── prover/                      # Rust recursive prover (Nova)
│   └── src/
│
├── demo/                        # Demo application
│   └── src/
│       └── demo.ts
│
└── scripts/                     # Automation scripts
```

---

##  How It Works

### 1. Origin Classification

Every state transition is tagged with an origin class:

```typescript
enum Origin {
    Genesis = 0,  // Initial deployment
    User = 1,     // Normal user transaction
    Admin = 2,    // Privileged operation
    Bridge = 3    // Cross-chain import
}
```

### 2. Lineage Commitment

States carry cryptographic commitments to their entire history:

```
C₀ = Hash(genesis_state, 0, 0)
Cₙ = Hash(Cₙ₋₁, transition_hash, depth)
```

**Property:** Constant size regardless of history length.

### 3. Policy Enforcement

The ZK circuit enforces transition rules:

```circom
// Users cannot escalate to Admin
signal userToAdmin;
userToAdmin <== isUser.out * toAdmin.out;
userToAdmin === 0;  // Must be zero (not allowed)
```

### 4. Zero-Knowledge Proof

The prover generates a proof showing:
-  Origin transition follows policy
-  Lineage commitment correctly updated
-  No revelation of actual origin classes

---

##  Security Analysis

### What ZK-ORIGIN Proves

| Property                                 | Proven in ZK |
|------------------------------------------|--------------|
| Origin transition is policy-compliant    | YES          |
| Lineage commitment is correctly computed | YES          |
| State has valid ancestry from genesis    | YES          |
| No privilege escalation occurred         | YES          |

### What ZK-ORIGIN Hides

| Information                  | Hidden       |
|------------------------------|--------------|
| Specific origin classes used | YES          |
| Intermediate states          | YES          |
| Transition timestamps        | YES          |
| Lineage depth (optional)     | Configurable |

### Trust Assumptions

1. **Circuit correctness** - Auditable, open source
2. **Trusted setup** - Uses Groth16 ceremony (can use existing ceremonies)
3. **Hash security** - Poseidon hash function security

---

##  Deployment

### Testnet (Sepolia)

```bash
cd contracts

# Set up environment
cp .env.example .env
# Edit .env with your PRIVATE_KEY and SEPOLIA_RPC_URL

# Deploy
npx hardhat run scripts/deploy.js --network sepolia


##  Testing

### Circuit Tests

```bash
cd circuits
npm test
```

### Contract Tests

```bash
cd contracts
npx hardhat test
```

### Full Integration Test

```bash
./scripts/test-all.sh
```

---

##  Future Work

- [ ] **Recursive Proofs (Nova)** - O(1) verification for any chain length
- [ ] **Merkle Policy Tree** - Configurable policies on-chain
- [ ] **Rate Limiting** - Epoch-based operation limits
- [ ] **Multi-chain Support** - Cross-chain lineage verification
- [ ] **Frontend Dashboard** - Visual lineage explorer

---

##  Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) first.

```bash
# Fork the repo
# Create your feature branch
git checkout -b feature/amazing-feature

# Commit your changes
git commit -m 'Add amazing feature'

# Push to the branch
git push origin feature/amazing-feature

# Open a Pull Request
```

---

##  License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

##  Acknowledgments

- [iden3/circom](https://github.com/iden3/circom) - Circuit compiler
- [iden3/snarkjs](https://github.com/iden3/snarkjs) - Proof generation
- [Poseidon Hash](https://eprint.iacr.org/2019/458) - ZK-friendly hash function
- [Hardhat](https://hardhat.org/) - Ethereum development environment

---

##  Contact

- **GitHub:** [@ZKChain](https://github.com/ZKChainForge)
- **X:** [@ZKchain](https://x.com/zkchain_z41420)
- **LinkedIn:** [Vikram](https://linkedin.com/in/vikram-a-a6a252395)

---

<div align="center">

**Built with ❤️ for the ZK community**

⭐ Star this repo if you find it useful!

</div>
```

