const hre = require("hardhat");
const { ethers } = hre;
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("========================================");
  console.log("ZK-ORIGIN LOCAL TESTING");
  console.log("========================================\n");

  // ─── Load deployment ───────────────────────────────────────────────────────
  const deploymentPath = path.join(
    __dirname,
    "..",
    "deployments",
    "localhost.json"
  );

  if (!fs.existsSync(deploymentPath)) {
    console.error(
      "ERROR: No deployment found. Run deploy-local.js first."
    );
    process.exit(1);
  }

  const deployment = JSON.parse(fs.readFileSync(deploymentPath, "utf8"));
  console.log("Loaded deployment from:", deploymentPath);

  // ── Destructure with safe fallbacks ─────────────────────────────────────
  const contracts = deployment.contracts || {};
  const genesis = deployment.genesis || {};

  const genesisStateHash = genesis.genesisStateHash;
  const genesisLineageCommitment = genesis.genesisLineageCommitment;
  const policyMerkleRoot = genesis.policyMerkleRoot;
  const policyRoot = genesis.policyRoot;
  const genesisTime = genesis.genesisTime;

  // Validate critical fields
  const requiredGenesis = {
    genesisStateHash,
    genesisLineageCommitment,
    policyMerkleRoot,
    policyRoot,
  };

  for (const [key, value] of Object.entries(requiredGenesis)) {
    if (!value) {
      console.error(
        `ERROR: Missing genesis field "${key}" in localhost.json`
      );
      console.error(
        "Re-run deploy-local.js to regenerate the deployment file."
      );
      process.exit(1);
    }
  }

  const requiredContracts = [
    "LineageVerifier",
    "RateLimiter",
    "EpochManager",
    "StateRegistry",
    "PolicyRegistry",
    "BatchVerifier",
  ];

  for (const name of requiredContracts) {
    if (!contracts[name]) {
      console.error(
        `ERROR: Missing contract address "${name}" in localhost.json`
      );
      process.exit(1);
    }
  }

  console.log("Genesis state hash:", genesisStateHash);
  console.log("Policy root:       ", policyRoot);

  // ─── Signers ───────────────────────────────────────────────────────────────
  const [deployer, user1, user2] = await ethers.getSigners();

  console.log("\nAccounts:");
  console.log("  Deployer:", deployer.address);
  console.log("  User 1:  ", user1.address);
  console.log("  User 2:  ", user2.address);

  // ─── Contract instances ────────────────────────────────────────────────────
  const lineageVerifier = await ethers.getContractAt(
    "LineageVerifier",
    contracts.LineageVerifier
  );
  const rateLimiter = await ethers.getContractAt(
    "RateLimiter",
    contracts.RateLimiter
  );
  const epochManager = await ethers.getContractAt(
    "EpochManager",
    contracts.EpochManager
  );
  const stateRegistry = await ethers.getContractAt(
    "StateRegistry",
    contracts.StateRegistry
  );
  const policyRegistry = await ethers.getContractAt(
    "PolicyRegistry",
    contracts.PolicyRegistry
  );

  // ─── TEST 1: Genesis State ─────────────────────────────────────────────────
  console.log("\n========================================");
  console.log("TEST 1: Verify Genesis State");
  console.log("========================================");

  const genesisVerified = await lineageVerifier.hasVerifiedLineage(
    genesisStateHash
  );
  console.log("Genesis verified:", genesisVerified);

  const genesisDepth = await lineageVerifier.getDepth(genesisStateHash);
  console.log("Genesis depth:", genesisDepth.toString());

  const genesisOriginClass = await lineageVerifier.stateOriginClass(
    genesisStateHash
  );
  console.log(
    "Genesis origin class:",
    genesisOriginClass.toString(),
    "(0 = Genesis)"
  );

  // ─── TEST 2: Epoch System ──────────────────────────────────────────────────
  console.log("\n========================================");
  console.log("TEST 2: Check Epoch System");
  console.log("========================================");

  const currentEpoch = await epochManager.getCurrentEpoch();
  const epochDuration = await epochManager.getEpochDuration();
  const epochGenesis = await epochManager.genesisTime();

  console.log("Current epoch:   ", currentEpoch.toString());
  console.log("Epoch duration:  ", epochDuration.toString(), "seconds");
  console.log("Genesis time:    ", epochGenesis.toString());

  // ─── TEST 3: Rate Limits ───────────────────────────────────────────────────
  console.log("\n========================================");
  console.log("TEST 3: Check Rate Limits");
  console.log("========================================");

  const originClasses = [
    "Genesis",
    "User",
    "Admin",
    "Bridge",
    "Governance",
    "System",
    "Emergency",
  ];

  for (let i = 0; i < 7; i++) {
    try {
      const limit = await rateLimiter.getLimit(i);
      const counter = await rateLimiter.getCounter(currentEpoch, i);
      console.log(
        `  ${originClasses[i].padEnd(12)}: ` +
          `${counter.toString().padStart(5)} / ` +
          `${limit.toString().padStart(10)}`
      );
    } catch (err) {
      console.log(
        `  ${originClasses[i].padEnd(12)}: [error - ${
          err.reason || err.message
        }]`
      );
    }
  }

  // ─── TEST 4: Policy Transitions ───────────────────────────────────────────
  console.log("\n========================================");
  console.log("TEST 4: Check Policy Transitions");
  console.log("========================================");

  const testTransitions = [
    [0, 1, "Genesis -> User"],
    [0, 2, "Genesis -> Admin"],
    [0, 5, "Genesis -> System"],
    [1, 1, "User    -> User"],
    [1, 2, "User    -> Admin (expect BLOCKED)"],
    [2, 1, "Admin   -> User"],
    [2, 2, "Admin   -> Admin"],
    [2, 3, "Admin   -> Bridge"],
    [3, 1, "Bridge  -> User"],
    [4, 0, "Gov     -> Genesis"],
    [4, 6, "Gov     -> Emergency"],
    [5, 1, "System  -> User"],
    [6, 1, "Emerg   -> User"],
  ];

  for (const [from, to, description] of testTransitions) {
    try {
      const allowed = await lineageVerifier.policyMatrix(from, to);
      const status = allowed ? "✅ ALLOWED" : "❌ BLOCKED";
      console.log(`  ${description.padEnd(35)}: ${status}`);
    } catch (err) {
      console.log(
        `  ${description.padEnd(35)}: [error - ${err.reason || err.message}]`
      );
    }
  }

  // ─── TEST 5: Contract Statistics ──────────────────────────────────────────
  console.log("\n========================================");
  console.log("TEST 5: Contract Statistics");
  console.log("========================================");

  const totalTransitions = await lineageVerifier.totalTransitions();
  const maxDepth = await lineageVerifier.maxDepthReached();
  const lastEpoch = await lineageVerifier.lastEpochProcessed();
  const totalStates = await stateRegistry.totalStates();

  console.log("  Total transitions:    ", totalTransitions.toString());
  console.log("  Max depth reached:    ", maxDepth.toString());
  console.log("  Last epoch processed: ", lastEpoch.toString());
  console.log("  Total states:         ", totalStates.toString());

  // ─── TEST 6: State Registry Queries ───────────────────────────────────────
  console.log("\n========================================");
  console.log("TEST 6: State Registry Queries");
  console.log("========================================");

  const isStateVerified = await stateRegistry.isStateVerified(genesisStateHash);
  console.log("  Genesis in state registry:", isStateVerified);

  if (!isStateVerified) {
    console.log(
      "    Genesis NOT in StateRegistry (this is expected if not registered separately)"
    );
    console.log(
      "     StateRegistry and LineageVerifier are separate contracts."
    );
    console.log(
      "     Genesis is verified in LineageVerifier but may need separate registration."
    );
  } else {
    // Only query these if state exists
    try {
      const stateDepth = await stateRegistry.getStateDepth(genesisStateHash);
      console.log(
        "  Genesis depth (from registry):",
        stateDepth.toString()
      );

      const stateOriginClass = await stateRegistry.getStateOriginClass(
        genesisStateHash
      );
      console.log(
        "  Genesis origin class (registry):",
        stateOriginClass.toString()
      );
    } catch (err) {
      console.log(
        "  [error querying state details -",
        err.errorName || err.reason || err.message,
        "]"
      );
    }
  }

  // ─── TEST 7: Policy Registry Checks ───────────────────────────────────────
  console.log("\n========================================");
  console.log("TEST 7: Policy Registry");
  console.log("========================================");

  try {
    const activePolicyRoot = await policyRegistry.getCurrentPolicyRoot();
    console.log("  Active policy root:", activePolicyRoot);
    console.log(
      "  Matches deployment:",
      activePolicyRoot === policyRoot ? " YES" : " NO"
    );
  } catch (err) {
    console.log(
      "  [error reading policy root -",
      err.reason || err.message,
      "]"
    );
  }

  // ─── TEST 8: Lineage Commitment Check ─────────────────────────────────────
  console.log("\n========================================");
  console.log("TEST 8: Lineage Commitment");
  console.log("========================================");

  try {
    const storedCommitment = await lineageVerifier.genesisLineageCommitment();
    console.log("  Stored commitment: ", storedCommitment);
    console.log("  Expected:          ", genesisLineageCommitment);
    console.log(
      "  Match:             ",
      storedCommitment === genesisLineageCommitment ? "✅ YES" : "❌ NO"
    );
  } catch (err) {
    console.log(
      "  [error reading commitment -",
      err.reason || err.message,
      "]"
    );
  }

  // ─── TEST 9: Check Contract Connections ───────────────────────────────────
  console.log("\n========================================");
  console.log("TEST 9: Contract Connections");
  console.log("========================================");

  try {
    const connectedLV = await rateLimiter.lineageVerifier();
    console.log("  RateLimiter -> LineageVerifier:");
    console.log("    Expected: ", contracts.LineageVerifier);
    console.log("    Actual:   ", connectedLV);
    console.log("    Match:    ", connectedLV === contracts.LineageVerifier ? "✅" : "❌");

    const stateRegLV = await stateRegistry.lineageVerifier();
    console.log("  StateRegistry -> LineageVerifier:");
    console.log("    Expected: ", contracts.LineageVerifier);
    console.log("    Actual:   ", stateRegLV);
    console.log("    Match:    ", stateRegLV === contracts.LineageVerifier ? "✅" : "❌");
  } catch (err) {
    console.log("  [error checking connections -", err.reason || err.message, "]");
  }

  // ─── Summary ──────────────────────────────────────────────────────────────
  console.log("\n========================================");
  console.log("ALL TESTS COMPLETED");
  console.log("========================================\n");

  // ─── Save results ──────────────────────────────────────────────────────────
  const results = {
    timestamp: new Date().toISOString(),
    network: "localhost",
    deployment: {
      genesisStateHash,
      genesisLineageCommitment,
      policyMerkleRoot,
      policyRoot,
      genesisTime,
    },
    tests: {
      genesisVerifiedInLineageVerifier: genesisVerified,
      genesisInStateRegistry: isStateVerified,
      currentEpoch: currentEpoch.toString(),
      epochDuration: epochDuration.toString(),
      totalTransitions: totalTransitions.toString(),
      maxDepth: maxDepth.toString(),
      lastEpochProcessed: lastEpoch.toString(),
      totalStates: totalStates.toString(),
      genesisDepth: genesisDepth.toString(),
      genesisOriginClass: genesisOriginClass.toString(),
    },
    accounts: {
      deployer: deployer.address,
      user1: user1.address,
      user2: user2.address,
    },
  };

  const resultsDir = path.join(__dirname, "..", "deployments");
  const resultsPath = path.join(resultsDir, "test-results.json");

  if (!fs.existsSync(resultsDir)) {
    fs.mkdirSync(resultsDir, { recursive: true });
  }

  fs.writeFileSync(resultsPath, JSON.stringify(results, null, 2));
  console.log("Test results saved to:", resultsPath);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("\n FATAL ERROR:");
    console.error(error);
    process.exit(1);
  });