const hre = require("hardhat");
const { ethers } = hre;
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("========================================");
  console.log("ZK-ORIGIN LOCAL DEPLOYMENT");
  console.log("========================================\n");

  const [deployer] = await ethers.getSigners();

  console.log("Deployer address:", deployer.address);

  const balance = await ethers.provider.getBalance(deployer.address);

  console.log(
    "Deployer balance:",
    ethers.utils.formatEther(balance),
    "ETH\n"
  );

  // ─── Step 1: Deploy Groth16Verifier ───────────────────────────────────────
  console.log("Step 1: Deploying Groth16Verifier...");

  const Groth16Verifier = await ethers.getContractFactory("Groth16Verifier");
  const groth16Verifier = await Groth16Verifier.deploy();
  await groth16Verifier.deployed();

  const groth16Address = groth16Verifier.address;
  console.log("Groth16Verifier deployed to:", groth16Address);

  // ─── Step 2: Deploy EpochManager ──────────────────────────────────────────
  console.log("\nStep 2: Deploying EpochManager...");

  const EpochManager = await ethers.getContractFactory("EpochManager");
  const epochManager = await EpochManager.deploy();
  await epochManager.deployed();

  const epochManagerAddress = epochManager.address;
  console.log("EpochManager deployed to:", epochManagerAddress);

  // ─── Step 3: Deploy RateLimiter ───────────────────────────────────────────
  console.log("\nStep 3: Deploying RateLimiter...");

  const genesisTime = Math.floor(Date.now() / 1000);

  const RateLimiter = await ethers.getContractFactory("RateLimiter");
  const rateLimiter = await RateLimiter.deploy(genesisTime);
  await rateLimiter.deployed();

  const rateLimiterAddress = rateLimiter.address;
  console.log("RateLimiter deployed to:", rateLimiterAddress);
  console.log("Genesis time:", genesisTime);

  // ─── Step 4: Deploy AuthorizationVerifier ─────────────────────────────────
  console.log("\nStep 4: Deploying AuthorizationVerifier...");

  const AuthorizationVerifier = await ethers.getContractFactory(
    "AuthorizationVerifier"
  );
  const authVerifier = await AuthorizationVerifier.deploy();
  await authVerifier.deployed();

  const authVerifierAddress = authVerifier.address;
  console.log("AuthorizationVerifier deployed to:", authVerifierAddress);

  // ─── Step 5: Deploy PolicyRegistry ────────────────────────────────────────
  console.log("\nStep 5: Deploying PolicyRegistry...");

  const PolicyRegistry = await ethers.getContractFactory("PolicyRegistry");
  const policyRegistry = await PolicyRegistry.deploy();
  await policyRegistry.deployed();

  const policyRegistryAddress = policyRegistry.address;
  console.log("PolicyRegistry deployed to:", policyRegistryAddress);

  // ─── Step 6: Create default policy ────────────────────────────────────────
  console.log("\nStep 6: Creating default policy...");

  // Define transitions as plain JS arrays (uint8[2][])
  // Each inner array is [fromState, toState] as numbers 0-255
  const transitions = [
    [0, 1],
    [0, 2],
    [0, 5],
    [1, 1],
    [2, 1],
    [2, 2],
    [2, 3],
    [2, 5],
    [3, 1],
    [4, 0],
    [4, 1],
    [4, 2],
    [4, 3],
    [4, 4],
    [4, 5],
    [4, 6],
    [5, 1],
    [5, 5],
    [6, 1],
    [6, 2],
    [6, 5],
  ];

  // Encode transitions to compute deterministic merkle root
  // NOTE: We encode OFF-CHAIN only for the hash — the array itself
  //       is passed directly to the contract as uint8[2][]
  const encodedTransitions = ethers.utils.defaultAbiCoder.encode(
    ["uint8[2][]"],
    [transitions]
  );

  const policyMerkleRoot = ethers.utils.keccak256(encodedTransitions);

  console.log("Computed policyMerkleRoot:", policyMerkleRoot);
  console.log(
    "Transitions count:",
    transitions.length
  );

  // Verify ABI signature before calling
  const createPolicyFragment =
    policyRegistry.interface.getFunction("createPolicy");

  console.log(
    "createPolicy signature:",
    createPolicyFragment.inputs
      .map((i) => `${i.type} ${i.name}`)
      .join(", ")
  );

  // Contract expects: createPolicy(bytes32 merkleRoot, uint8[2][] transitions)
  // Pass the raw JS array — ethers.js handles the ABI encoding internally
  const createPolicyTx = await policyRegistry.createPolicy(
    policyMerkleRoot,
    transitions   // ← raw array, NOT encodedTransitions hex string
  );

  const createReceipt = await createPolicyTx.wait();
  console.log(
    "Policy created | tx:",
    createReceipt.transactionHash
  );
  console.log("Policy Merkle root:", policyMerkleRoot);

  // ─── Step 7: Propose & activate policy ────────────────────────────────────
  console.log("\nStep 7: Proposing policy activation...");

  const proposeTx = await policyRegistry.proposePolicyActivation(0);
  await proposeTx.wait();
  console.log("Policy activation proposed.");

  console.log("Fast-forwarding 2 days for timelock...");

  await hre.network.provider.send("evm_increaseTime", [2 * 24 * 60 * 60]);
  await hre.network.provider.send("evm_mine");

  console.log("Activating policy...");

  const activateTx = await policyRegistry.activatePolicy(0);
  await activateTx.wait();

  const policyRoot = await policyRegistry.getCurrentPolicyRoot();
  console.log("Policy activated | root:", policyRoot);

  // ─── Step 8: Deploy LineageVerifier ───────────────────────────────────────
  console.log("\nStep 8: Deploying LineageVerifier...");

  const genesisLineageCommitment = ethers.utils.keccak256(
    ethers.utils.toUtf8Bytes("GENESIS_LINEAGE_COMMITMENT")
  );

  const LineageVerifier = await ethers.getContractFactory("LineageVerifier");
  const lineageVerifier = await LineageVerifier.deploy(
    groth16Address,
    epochManagerAddress,
    rateLimiterAddress,
    authVerifierAddress,
    genesisLineageCommitment,
    policyRoot
  );
  await lineageVerifier.deployed();

  const lineageVerifierAddress = lineageVerifier.address;
  console.log("LineageVerifier deployed to:", lineageVerifierAddress);

  // ─── Step 9: Connect RateLimiter ──────────────────────────────────────────
  console.log("\nStep 9: Connecting RateLimiter to LineageVerifier...");

  const setVerifierTx = await rateLimiter.setLineageVerifier(
    lineageVerifierAddress
  );
  await setVerifierTx.wait();
  console.log("RateLimiter connected to LineageVerifier.");

  // ─── Step 10: Deploy StateRegistry ────────────────────────────────────────
  console.log("\nStep 10: Deploying StateRegistry...");

  const StateRegistry = await ethers.getContractFactory("StateRegistry");
  const stateRegistry = await StateRegistry.deploy(lineageVerifierAddress);
  await stateRegistry.deployed();

  const stateRegistryAddress = stateRegistry.address;
  console.log("StateRegistry deployed to:", stateRegistryAddress);

  // ─── Step 11: Deploy BatchVerifier ────────────────────────────────────────
  console.log("\nStep 11: Deploying BatchVerifier...");

  const BatchVerifier = await ethers.getContractFactory("BatchVerifier");
  const batchVerifier = await BatchVerifier.deploy(
    lineageVerifierAddress,
    authVerifierAddress
  );
  await batchVerifier.deployed();

  const batchVerifierAddress = batchVerifier.address;
  console.log("BatchVerifier deployed to:", batchVerifierAddress);

  // ─── Step 12: Set Genesis ──────────────────────────────────────────────────
  console.log("\nStep 12: Setting genesis state...");

  const genesisStateHash = ethers.utils.keccak256(
    ethers.utils.toUtf8Bytes("GENESIS_STATE")
  );

  const setGenesisTx = await lineageVerifier.setGenesis(
    genesisStateHash,
    genesisLineageCommitment
  );
  await setGenesisTx.wait();

  console.log("Genesis state set:", genesisStateHash);

  const isGenesisVerified = await lineageVerifier.hasVerifiedLineage(
    genesisStateHash
  );
  console.log("Genesis verified:", isGenesisVerified);

  // ─── Save deployment info ──────────────────────────────────────────────────
  const network = await ethers.provider.getNetwork();

  const deploymentInfo = {
    network: hre.network.name,
    chainId: network.chainId,
    deployedAt: new Date().toISOString(),
    deployer: deployer.address,
    contracts: {
      Groth16Verifier:       groth16Address,
      EpochManager:          epochManagerAddress,
      RateLimiter:           rateLimiterAddress,
      AuthorizationVerifier: authVerifierAddress,
      PolicyRegistry:        policyRegistryAddress,
      LineageVerifier:       lineageVerifierAddress,
      StateRegistry:         stateRegistryAddress,
      BatchVerifier:         batchVerifierAddress,
    },
    genesis: {
      genesisTime,
      genesisStateHash,
      genesisLineageCommitment,
      policyMerkleRoot,
      policyRoot,
    },
  };

  const deploymentsDir = path.join(__dirname, "../deployments");

  if (!fs.existsSync(deploymentsDir)) {
    fs.mkdirSync(deploymentsDir, { recursive: true });
  }

  const deploymentFile = path.join(
    deploymentsDir,
    `${hre.network.name}.json`
  );

  fs.writeFileSync(deploymentFile, JSON.stringify(deploymentInfo, null, 2));

  console.log("\nDeployment info saved to:", deploymentFile);

  // ─── Print summary ─────────────────────────────────────────────────────────
  console.log("\n========================================");
  console.log("DEPLOYMENT COMPLETED SUCCESSFULLY");
  console.log("========================================");
  console.log("\nContract Addresses:");
  console.log("──────────────────────────────────────────────────────");

  Object.entries(deploymentInfo.contracts).forEach(([name, addr]) => {
    console.log(`  ${name.padEnd(26)} ${addr}`);
  });

  console.log("──────────────────────────────────────────────────────");
  console.log("\nGenesis Info:");
  console.log("  genesisTime              ", genesisTime);
  console.log("  genesisStateHash         ", genesisStateHash);
  console.log("  genesisLineageCommitment ", genesisLineageCommitment);
  console.log("  policyMerkleRoot         ", policyMerkleRoot);
  console.log("  policyRoot               ", policyRoot);
  console.log();
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });