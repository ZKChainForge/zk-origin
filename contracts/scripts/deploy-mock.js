const hre = require("hardhat");
const { ethers } = hre;
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("========================================");
  console.log("ZK-ORIGIN MOCK DEPLOYMENT (TESTING ONLY)");
  console.log("========================================\n");

  const [deployer] = await ethers.getSigners();
  console.log("Deployer:", deployer.address);

  // 1. Deploy MOCK Groth16Verifier
  console.log("\n🚨 Step 1: Deploying MockGroth16Verifier (always returns true)...");
  
  const MockGroth16Verifier = await ethers.getContractFactory("MockGroth16Verifier");
  const mockVerifier = await MockGroth16Verifier.deploy();
  await mockVerifier.deployed();
  
  console.log("MockGroth16Verifier deployed to:", mockVerifier.address);
  console.log("⚠️  WARNING: This verifier accepts ANY proof - for testing only!");

  // 2. Deploy EpochManager
  console.log("\nStep 2: Deploying EpochManager...");
  const EpochManager = await ethers.getContractFactory("EpochManager");
  const epochManager = await EpochManager.deploy();
  await epochManager.deployed();
  console.log("EpochManager deployed to:", epochManager.address);

  // 3. Deploy RateLimiter
  console.log("\nStep 3: Deploying RateLimiter...");
  const genesisTime = Math.floor(Date.now() / 1000);
  const RateLimiter = await ethers.getContractFactory("RateLimiter");
  const rateLimiter = await RateLimiter.deploy(genesisTime);
  await rateLimiter.deployed();
  console.log("RateLimiter deployed to:", rateLimiter.address);

  // 4. Deploy AuthorizationVerifier
  console.log("\nStep 4: Deploying AuthorizationVerifier...");
  const AuthorizationVerifier = await ethers.getContractFactory("AuthorizationVerifier");
  const authVerifier = await AuthorizationVerifier.deploy();
  await authVerifier.deployed();
  console.log("AuthorizationVerifier deployed to:", authVerifier.address);

  // 5. Deploy PolicyRegistry
  console.log("\nStep 5: Deploying PolicyRegistry...");
  const PolicyRegistry = await ethers.getContractFactory("PolicyRegistry");
  const policyRegistry = await PolicyRegistry.deploy();
  await policyRegistry.deployed();
  console.log("PolicyRegistry deployed to:", policyRegistry.address);

  // 6. Create and activate policy
  console.log("\nStep 6: Creating default policy...");
  const transitions = [
    [0, 1], [0, 2], [0, 5],
    [1, 1], [2, 1], [2, 2], [2, 3], [2, 5],
    [3, 1], [4, 0], [4, 1], [4, 2], [4, 3], [4, 4], [4, 5], [4, 6],
    [5, 1], [5, 5], [6, 1], [6, 2], [6, 5],
  ];

  const encodedTransitions = ethers.utils.defaultAbiCoder.encode(
    ["uint8[2][]"],
    [transitions]
  );
  const policyMerkleRoot = ethers.utils.keccak256(encodedTransitions);

  const createPolicyTx = await policyRegistry.createPolicy(policyMerkleRoot, transitions);
  await createPolicyTx.wait();
  console.log("Policy created");

  const proposeTx = await policyRegistry.proposePolicyActivation(0);
  await proposeTx.wait();

  await hre.network.provider.send("evm_increaseTime", [2 * 24 * 60 * 60]);
  await hre.network.provider.send("evm_mine");

  const activateTx = await policyRegistry.activatePolicy(0);
  await activateTx.wait();

  const policyRoot = await policyRegistry.getCurrentPolicyRoot();
  console.log("Policy activated:", policyRoot);

  // 7. Deploy LineageVerifier with MOCK verifier
  console.log("\nStep 7: Deploying LineageVerifier with mock verifier...");
  
  const genesisLineageCommitment = ethers.utils.keccak256(
    ethers.utils.toUtf8Bytes("GENESIS_LINEAGE_COMMITMENT")
  );

  const LineageVerifier = await ethers.getContractFactory("LineageVerifier");
  const lineageVerifier = await LineageVerifier.deploy(
    mockVerifier.address,  // ← MOCK VERIFIER HERE
    epochManager.address,
    rateLimiter.address,
    authVerifier.address,
    genesisLineageCommitment,
    policyRoot
  );
  await lineageVerifier.deployed();
  console.log("LineageVerifier deployed to:", lineageVerifier.address);

  // 8. Connect RateLimiter
  console.log("\nStep 8: Connecting RateLimiter...");
  const setVerifierTx = await rateLimiter.setLineageVerifier(lineageVerifier.address);
  await setVerifierTx.wait();

  // 9. Deploy StateRegistry
  console.log("\nStep 9: Deploying StateRegistry...");
  const StateRegistry = await ethers.getContractFactory("StateRegistry");
  const stateRegistry = await StateRegistry.deploy(lineageVerifier.address);
  await stateRegistry.deployed();
  console.log("StateRegistry deployed to:", stateRegistry.address);

  // 10. Deploy BatchVerifier
  console.log("\nStep 10: Deploying BatchVerifier...");
  const BatchVerifier = await ethers.getContractFactory("BatchVerifier");
  const batchVerifier = await BatchVerifier.deploy(
    lineageVerifier.address,
    authVerifier.address
  );
  await batchVerifier.deployed();
  console.log("BatchVerifier deployed to:", batchVerifier.address);

  // 11. Set Genesis
  console.log("\nStep 11: Setting genesis...");
  const genesisStateHash = ethers.utils.keccak256(
    ethers.utils.toUtf8Bytes("GENESIS_STATE")
  );

  const setGenesisTx = await lineageVerifier.setGenesis(
    genesisStateHash,
    genesisLineageCommitment
  );
  await setGenesisTx.wait();
  console.log("Genesis set:", genesisStateHash);

  // Save deployment
  const deployment = {
    network: "localhost-mock",
    chainId: (await ethers.provider.getNetwork()).chainId,
    deployedAt: new Date().toISOString(),
    deployer: deployer.address,
    isMockVerifier: true,
    warning: "MOCK VERIFIER - DO NOT USE IN PRODUCTION",
    contracts: {
      MockGroth16Verifier: mockVerifier.address,
      EpochManager: epochManager.address,
      RateLimiter: rateLimiter.address,
      AuthorizationVerifier: authVerifier.address,
      PolicyRegistry: policyRegistry.address,
      LineageVerifier: lineageVerifier.address,
      StateRegistry: stateRegistry.address,
      BatchVerifier: batchVerifier.address,
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

  fs.writeFileSync(
    path.join(deploymentsDir, "localhost-mock.json"),
    JSON.stringify(deployment, null, 2)
  );

  console.log("\n========================================");
  console.log("🚨 MOCK DEPLOYMENT COMPLETE");
  console.log("========================================");
  console.log("Deployment saved to: deployments/localhost-mock.json");
  console.log("\n⚠️  WARNING: Mock verifier accepts ANY proof!");
  console.log("   Use only for testing contract logic.");
  console.log("   Deploy with real Groth16Verifier for production.\n");
}

main()
  .then(() => process.exit(0))
  .catch(console.error);