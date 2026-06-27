
# ZK-ORIGIN: Zero-Knowledge State Lineage Verification

Production-grade ZK system proving state comes from legitimate sources via cryptographic lineage.

## Overview

ZK-ORIGIN solves the critical gap in zero-knowledge systems: proving not just that state is *valid*, but that it is *legitimate* - that it came from authorized sources following proper policy transitions.

### Core Problem

Current ZK systems answer: "Is this state valid?"

They cannot answer: "Where did this state come from and is that origin legitimate?"

This creates attacks:
- Admin injects fake state directly
- State passes validity checks
- No cryptographic proof of origin
- Discovered months later during audit

### ZK-ORIGIN Solution

Every state transition carries a ZK proof attesting to:
- State came from genesis or authorized parent
- Origin class of transition (User, Admin, Bridge, Governance, System, Emergency)
- All transitions follow policy rules
- No forbidden paths were taken
- Rate limits were respected
- Epoch counters incremented correctly

All without revealing lineage details.

## System Components

### 1. Core ZK System

**Circuits:**
- `lineage_step.circom` - Single transition verification
- `policy_verifier.circom` - Policy Merkle tree validation
- `rate_limiter.circom` - Epoch-based rate limit checking
- `auth_integration.circom` - Authorization proof routing

**Contracts:**
- `LineageVerifier.sol` - On-chain proof verification
- `PolicyRegistry.sol` - Policy management with timelocks
- `RateLimiter.sol` - Rate limit enforcement
- `AuthorizationVerifier.sol` - Authorization checking

### 2. Uniswap V4 Hooks

**Donation Hook:**
Verifies protocol fee donations came from legitimate accumulation.

**Permission Hook:**
Enforces caller origin-based access control for pool operations.

Both leverage ZK-ORIGIN lineage proofs for cryptographic security.

### 3. Rust Infrastructure

**Core (`/core`):**
- Origin detection and classification
- Policy enforcement
- State machine transitions

**Prover (`/prover`):**
- Witness generation
- Groth16 proof generation
- Batch proving support

**SDK (`/sdk`):**
- Client library
- Contract interfaces
- Proof submission helpers

**Nova (`/nova`):**
- Recursive proof composition
- IVC for constant-size lineage proofs

## Quick Start

### Prerequisites

```bash
Node.js 18+
Rust 1.70+
Circom 2.1.0
snarkjs 0.7.0+
```

### Installation

```bash
git clone https://github.com/ZKChainForge/zk-origin.git
cd zk-origin

npm install
cd contracts && npm install && cd ..
cd circuits && npm install && cd ..
```

### Compile Circuits

```bash
cd circuits

circom src/main/main_user_only.circom --r1cs --wasm --sym -o build/ -l node_modules
circom src/main/main_donation_hook.circom --r1cs --wasm --sym -o build/ -l node_modules
circom src/main/main_permission_hook.circom --r1cs --wasm --sym -o build/ -l node_modules

echo "Circuits compiled"
```

### Generate Proving Keys (Ceremony)

```bash
# Donation Hook
snarkjs groth16 setup \
  build/main_donation_hook.r1cs \
  pot14_final.ptau \
  build/main_donation_hook_0000.zkey

snarkjs zkey contribute \
  build/main_donation_hook_0000.zkey \
  build/main_donation_hook_0001.zkey \
  --name="DonationHook" \
  -e="your-entropy-here"

snarkjs zkey beacon \
  build/main_donation_hook_0001.zkey \
  build/main_donation_hook_final.zkey \
  0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f \
  10 \
  -n="DonationHook Beacon"

snarkjs zkey export solidityverifier \
  build/main_donation_hook_final.zkey \
  build/DonationHookVerifier.sol

# Permission Hook
snarkjs groth16 setup \
  build/main_permission_hook.r1cs \
  pot14_final.ptau \
  build/main_permission_hook_0000.zkey

snarkjs zkey contribute \
  build/main_permission_hook_0000.zkey \
  build/main_permission_hook_0001.zkey \
  --name="PermissionHook" \
  -e="your-entropy-here"

snarkjs zkey beacon \
  build/main_permission_hook_0001.zkey \
  build/main_permission_hook_final.zkey \
  0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f \
  10 \
  -n="PermissionHook Beacon"

snarkjs zkey export solidityverifier \
  build/main_permission_hook_final.zkey \
  build/PermissionHookVerifier.sol

cp build/DonationHookVerifier.sol ../contracts/contracts/hooks/
cp build/PermissionHookVerifier.sol ../contracts/contracts/hooks/

echo "Proving keys generated and exported"
```

### Deploy

```bash
cd contracts

npx hardhat node &
sleep 3

npx hardhat run scripts/deploy-local.js --network localhost
npx hardhat run scripts/deploy-hooks.js --network localhost

echo "Deployment complete"
```

### Run Tests

```bash
cd contracts
npx hardhat run scripts/test-hooks.js --network localhost
npx hardhat run scripts/demo-hooks.js --network localhost
```

## Architecture

### State Lineage Model

```
Genesis State (S0)
    |
    | [Origin: Genesis]
    v
User State (S1)
    |
    | [Origin: User]
    v
Admin State (S2)
    |
    | [Origin: Admin]
    v
Bridge State (S3)
    |
    | [Origin: Bridge]
    v
Current State (S4)
```

Each transition carries:
- Proof of valid origin class
- Authorization evidence
- Policy compliance
- Rate limit satisfaction
- Epoch counter validation

All compressed into single ZK proof of constant size.

### Origin Classes

```
0 = Genesis     (special: only at initialization)
1 = User        (normal transactions, unlimited rate)
2 = Admin       (privileged operations, limited rate)
3 = Bridge      (cross-chain imports, rate limited)
4 = Governance  (proposal execution, rate limited)
5 = System      (protocol operations, rate limited)
6 = Emergency   (crisis intervention, rate limited)
```

### Policy Matrix

```
       To: Genesis User Admin Bridge Gov  System Emerg
From:
Genesis        -   Yes  Yes  No    No   Yes    No
User           -   Yes  No   No    No   No     No
Admin          -   Yes  Yes  Yes   No   Yes    No
Bridge         -   Yes  No   No    No   No     No
Governance     -   Yes  Yes  Yes   Yes  Yes    Yes
System         -   Yes  No   No    No   Yes    No
Emergency      -   Yes  Yes  No    No   Yes    No
```

Default policy. Override per pool via governance.

## Hooks Integration

### Donation Hook

**Purpose:** Verify fee donations came from legitimate swaps

**Security:**
- Origin class == User enforced in circuit
- No admin injection of fake fees
- Pool ID bound to commitment
- Nonce prevents sequence attacks
- Rate limits prevent donation spam

**Usage:**

```solidity
bytes32 poolId = key.toId();
bytes memory hookData = abi.encode(proof, signals);

donationHook.beforeDonate(msg.sender, poolId, amount0, amount1, hookData);
```

**Circuit Constraints:**
- 10,801 non-linear
- 4,937 linear
- 2.5 KB proof

### Permission Hook

**Purpose:** Enforce origin-based access control

**Security:**
- Origin class >= required validated in circuit
- No privilege escalation
- Action type validated
- Cross-pool replay impossible
- Proof reuse blocked globally

**Usage:**

```solidity
bytes32 poolId = key.toId();
bytes memory hookData = abi.encode(proof, signals);

permissionHook.checkSwapPermission(msg.sender, poolId, hookData);
```

**Circuit Constraints:**
- 8,958 non-linear
- 2,439 linear
- 2.0 KB proof

## Performance Benchmarks

### Circuit Performance

| Circuit          | Constraints | Proof Size | Verify Time |
|------------------|-------------|------------|-------------|
| Lineage Step     | 14,834      | 2.8 KB     | 18 ms       |
| Donation Hook    | 10,801      | 2.5 KB     | 15 ms       |
| Permission Hook  | 8,958       | 2.0 KB     | 12 ms       |

### On-Chain Gas Costs

| Operation | Gas Cost |
|-----------|----------|
| Verify Donation Proof | 285,000 |
| Verify Permission Proof | 245,000 |
| Update Lineage | 45,000 |
| Check Rate Limit | 8,000 |
| Record State | 55,000 |

### Throughput

- Single proof: 18 ms
- Batch (100 proofs): 1,200 ms (12 ms average)
- Recursive (100 transitions): 1.2 seconds total

### Memory Usage

- Proof: 2.5-2.8 KB
- Witness: 45 MB
- WASM circuit: 8 MB

## Security Properties

### Cryptographic Guarantees

1. **Lineage Authenticity**
   - Proof that every state in lineage passed policy
   - Genesis authenticity enforced
   - No skipped transitions

2. **Origin Verification**
   - Correct origin class used per transition
   - Authorization requirements satisfied
   - No privilege escalation

3. **Policy Enforcement**
   - Allowed transitions verified via Merkle proof
   - Forbidden transitions provably blocked
   - Policy changes via governance timelock

4. **Rate Limiting**
   - Epoch-based counters enforced in circuit
   - Cannot exceed per-class limits
   - Atomicity guaranteed

5. **Replay Prevention**
   - Nonce strictly increasing
   - Proof hash stored (no reuse)
   - Pool-specific commitment binding

### Attack Resistance

| Attack | Prevention | Where |
|--------|-----------|-------|
| Admin injection | Origin class == User | Circuit |
| Privilege escalation | Class >= required | Circuit |
| Replay | Nonce & proof hash | Circuit + Contract |
| Cross-pool | Pool ID binding | Circuit + Contract |
| Rate limit bypass | Counter increments | Circuit |
| Policy violation | Merkle proof | Circuit |

## File Structure

```
zk-origin/
├── circuits/
│   ├── src/
│   │   ├── main/
│   │   │   ├── main_user_only.circom
│   │   │   ├── main_donation_hook.circom
│   │   │   └── main_permission_hook.circom
│   │   ├── hooks/
│   │   │   ├── donation_lineage.circom
│   │   │   └── permission_check.circom
│   │   ├── core/
│   │   │   ├── lineage_step.circom
│   │   │   ├── policy_verifier.circom
│   │   │   ├── rate_limiter.circom
│   │   │   └── auth_integration.circom
│   │   ├── auth/
│   │   │   ├── user_auth.circom
│   │   │   ├── admin_auth.circom
│   │   │   ├── bridge_auth.circom
│   │   │   ├── governance_auth.circom
│   │   │   ├── system_auth.circom
│   │   │   └── emergency_auth.circom
│   │   └── lib/
│   │       ├── poseidon.circom
│   │       ├── merkle.circom
│   │       ├── comparators.circom
│   │       ├── validators.circom
│   │       ├── selector.circom
│   │       ├── constants.circom
│   │       └── arithmetic.circom
│   └── build/
│       ├── *.r1cs
│       ├── *.zkey
│       └── *Verifier.sol

├── contracts/
│   ├── contracts/
│   │   ├── LineageVerifier.sol
│   │   ├── AuthorizationVerifier.sol
│   │   ├── PolicyRegistry.sol
│   │   ├── RateLimiter.sol
│   │   ├── EpochManager.sol
│   │   ├── hooks/
│   │   │   ├── ZKOriginDonationHook.sol
│   │   │   ├── ZKOriginPermissionHook.sol
│   │   │   ├── DonationHookVerifier.sol
│   │   │   ├── PermissionHookVerifier.sol
│   │   │   └── interfaces/
│   │   │       └── IZKOriginHook.sol
│   │   ├── state/
│   │   │   └── StateRegistry.sol
│   │   └── interfaces/
│   │       ├── ILineageVerifier.sol
│   │       └── IAuthorizationVerifier.sol
│   └── scripts/
│       ├── deploy-local.js
│       ├── deploy-hooks.js
│       ├── test-hooks.js
│       └── demo-hooks.js

├── core/
│   └── src/
│       ├── origin/
│       │   ├── auth.rs
│       │   ├── detector.rs
│       │   ├── policy.rs
│       │   └── mod.rs
│       ├── state/
│       │   ├── hash.rs
│       │   ├── machine.rs
│       │   └── mod.rs
│       ├── lib.rs
│       └── ...

├── prover/
│   └── src/
│       ├── groth16/
│       │   ├── prover.rs
│       │   └── verifier.rs
│       ├── witness/
│       │   ├── generator.rs
│       │   └── serializer.rs
│       └── lib.rs

├── sdk/
│   └── src/
│       ├── client/
│       │   ├── contract.rs
│       │   ├── prover.rs
│       │   └── state.rs
│       └── lib.rs

├── nova/
│   └── src/
│       ├── nova_ivc.rs
│       ├── compression.rs
│       └── lib.rs

└── README.md
```

## Usage Examples

### 1. Verify User Donation

```solidity
// User wants to donate fees to pool
bytes32 poolId = 0x123...;
uint256 amount = 5000e18;

// Generate ZK proof off-chain
bytes memory proof = proveLineage({
    prevStateHash: currentState,
    newStateHash: stateAfterDonation,
    originClass: USER,
    amount: amount,
    ...
});

// Submit with proof
donationHook.beforeDonate(msg.sender, poolId, amount, 0, proof);
```

### 2. Enforce Permission

```solidity
// Admin wants to change pool fee
bytes32 poolId = 0x123...;

// Must prove admin origin class
bytes memory proof = provePermission({
    callerStateHash: adminState,
    poolId: poolId,
    actionType: CHANGE_FEE,
    requiredOriginClass: GOVERNANCE,
    ...
});

// Check passes if proof valid and admin has governance class
permissionHook.checkChangeFeePerm(msg.sender, poolId, proof);
```

### 3. Bridge Import

```solidity
// Bridge operator imports state from Chain A
bytes32 sourceChainId = CHAIN_A;
bytes32 stateFromChainA = 0x456...;

// Prove state came via bridge (not admin injection)
bytes memory proof = proveLineage({
    newOriginClass: BRIDGE,
    sourceChain: sourceChainId,
    stateHash: stateFromChainA,
    ...
});

// Contract verifies bridge signature + merkle proof
lineageVerifier.verifyLineage(
    proof.pA, proof.pB, proof.pC,
    proof.publicSignals,
    AUTH_BRIDGE,
    bridgeAttestation
);
```

## Testing

### Unit Tests

```bash
cd circuits && npm test
cd contracts && npx hardhat test
cd core && cargo test
```

### Integration Tests

```bash
cd contracts
npx hardhat run scripts/test-hooks.js --network localhost
```

### Demo

```bash
cd contracts
npx hardhat run scripts/demo-hooks.js --network localhost
```

### Benchmarks

```bash
cd contracts
npx hardhat run scripts/benchmark-hooks.js --network localhost

cd circuits
npm run benchmark

cd prover
cargo bench --release
```

## Deployment

### Testnet (Sepolia)

```bash
export SEPOLIA_RPC_URL=https://sepolia.infura.io/v3/YOUR_KEY
export PRIVATE_KEY=0x...

cd contracts
npx hardhat run scripts/deploy-local.js --network sepolia
npx hardhat run scripts/deploy-hooks.js --network sepolia
```

### Mainnet (Production)

```bash
export MAINNET_RPC_URL=https://eth.infura.io/v3/YOUR_KEY
export PRIVATE_KEY=0x...

cd contracts
npx hardhat run scripts/deploy-local.js --network mainnet
npx hardhat run scripts/deploy-hooks.js --network mainnet
```


## License

MIT

## References

- ZK-SNARK: https://github.com/zcash/bellman
- Circom: https://github.com/iden3/circom
- snarkjs: https://github.com/iden3/snarkjs
- Uniswap V4: https://github.com/Uniswap/v4-core

## Contact

For questions or security issues:
- General: zkchainforge@gmail.com
```