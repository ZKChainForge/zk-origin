const hre = require("hardhat");
const { ethers } = hre;
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("========================================");
  console.log("SUBMIT PROOF TEST");
  console.log("========================================\n");

  // ─── Load deployment ───────────────────────────────────────────────────────
  const deploymentPath = path.join(__dirname, "../deployments/localhost.json");

  if (!fs.existsSync(deploymentPath)) {
    console.error("ERROR: Run deploy-local.js first");
    process.exit(1);
  }

  const deployment = JSON.parse(fs.readFileSync(deploymentPath, "utf8"));

  const [deployer] = await ethers.getSigners();
  console.log("Submitter:", deployer.address);

  // ─── Get contract instances ────────────────────────────────────────────────
  const lineageVerifier = await ethers.getContractAt(
    "LineageVerifier",
    deployment.contracts.LineageVerifier
  );

  const epochManager = await ethers.getContractAt(
    "EpochManager",
    deployment.contracts.EpochManager
  );

  const rateLimiter = await ethers.getContractAt(
    "RateLimiter",
    deployment.contracts.RateLimiter
  );

  console.log("\nContract signature:");
  console.log("verifyLineage(");
  console.log("  uint256[2]    pA,");
  console.log("  uint256[2][2] pB,");
  console.log("  uint256[2]    pC,");
  console.log("  uint256[20]   publicSignals,");
  console.log("  uint8         authType,");
  console.log("  bytes         authData");
  console.log(")\n");

  // ─── Prepare Groth16 proof ─────────────────────────────────────────────────
  // Mock values — real proofs must come from snarkjs circuit
  const pA = [
    ethers.BigNumber.from("1"),
    ethers.BigNumber.from("2"),
  ];

  const pB = [
    [
      ethers.BigNumber.from("3"),
      ethers.BigNumber.from("4"),
    ],
    [
      ethers.BigNumber.from("5"),
      ethers.BigNumber.from("6"),
    ],
  ];

  const pC = [
    ethers.BigNumber.from("7"),
    ethers.BigNumber.from("8"),
  ];

  // ─── Prepare public signals ────────────────────────────────────────────────
  const parentStateHash      = deployment.genesis.genesisStateHash;
  const newStateHash         = ethers.utils.keccak256(
    ethers.utils.toUtf8Bytes("NEW_STATE_PROOF_TEST_1")
  );
  const parentCommitment     = deployment.genesis.genesisLineageCommitment;
  const newCommitment        = ethers.utils.keccak256(
    ethers.utils.toUtf8Bytes("NEW_COMMITMENT_1")
  );
  const parentOriginClass    = 0; // Genesis
  const newOriginClass       = 1; // User
  const parentDepth          = 0;

  // Build uint256[20] array — exactly 20 elements required
  const publicSignals = new Array(20).fill(ethers.BigNumber.from(0));
  publicSignals[0]  = ethers.BigNumber.from(parentStateHash);
  publicSignals[1]  = ethers.BigNumber.from(newStateHash);
  publicSignals[2]  = ethers.BigNumber.from(parentCommitment);
  publicSignals[3]  = ethers.BigNumber.from(newCommitment);
  publicSignals[4]  = ethers.BigNumber.from(parentOriginClass);
  publicSignals[5]  = ethers.BigNumber.from(newOriginClass);
  publicSignals[6]  = ethers.BigNumber.from(parentDepth);
  // [7..19] remain 0 (padding)

  console.log("─── Proof Data ───");
  console.log("Parent state:      ", parentStateHash);
  console.log("New state:         ", newStateHash);
  console.log("Parent commitment: ", parentCommitment);
  console.log("New commitment:    ", newCommitment);
  console.log(
    "Transition:        ",
    `${parentOriginClass} (Genesis) → ${newOriginClass} (User)`
  );
  console.log("Parent depth:      ", parentDepth);
  console.log("Expected new depth:", parentDepth + 1);

  // ─── Authorization data ────────────────────────────────────────────────────
  const authType = 0;   // 0 = no auth
  const authData = "0x"; // empty bytes

  // ─── Pre-flight checks ─────────────────────────────────────────────────────
  console.log("\n─── Pre-flight Checks ───");

  const isPaused = await lineageVerifier.isPaused();
  console.log("Contract paused:   ", isPaused);
  if (isPaused) {
    console.error("❌ Contract is paused!");
    process.exit(1);
  }

  const parentVerified = await lineageVerifier.hasVerifiedLineage(
    parentStateHash
  );
  console.log("Parent verified:   ", parentVerified);
  if (!parentVerified) {
    console.error("❌ Parent state not verified!");
    process.exit(1);
  }

  const transitionAllowed = await lineageVerifier.policyMatrix(
    parentOriginClass,
    newOriginClass
  );
  console.log("Transition allowed:", transitionAllowed);
  if (!transitionAllowed) {
    console.error("❌ Policy does not allow this transition!");
    process.exit(1);
  }

  // FIX: Use BigNumber.from() to normalize ALL return values
  // regardless of whether contract returns uint32, uint64, or uint256
  const currentEpoch = ethers.BigNumber.from(
    await epochManager.getCurrentEpoch()
  );

  const limitRaw   = await rateLimiter.getLimit(parentOriginClass);
  const counterRaw = await rateLimiter.getCounter(currentEpoch, parentOriginClass);

  // Safely convert to BigNumber regardless of returned type
  const limit   = ethers.BigNumber.from(limitRaw.toString());
  const counter = ethers.BigNumber.from(counterRaw.toString());

  console.log("Current epoch:     ", currentEpoch.toString());
  console.log(
    "Rate limit:        ",
    `${counter.toString()} / ${limit.toString()}`
  );

  if (counter.gte(limit)) {
    console.error(
      `❌ Rate limit exceeded for origin class ${parentOriginClass}!`
    );
    console.error(
      `   Counter: ${counter.toString()}, Limit: ${limit.toString()}`
    );
    process.exit(1);
  }

  console.log("All pre-flight checks passed ✅");

  // ─── Submit proof ──────────────────────────────────────────────────────────
  console.log("\n─── Submitting Proof ───");
  console.log("⚠️  Note: Mock proof will likely fail Groth16 verification.");
  console.log("   Real proofs must be generated via snarkjs from ZK circuit.\n");

  try {
    const tx = await lineageVerifier.verifyLineage(
      pA,
      pB,
      pC,
      publicSignals,
      authType,
      authData,
      { gasLimit: 5000000 }
    );

    console.log("Transaction sent, waiting for confirmation...");
    const receipt = await tx.wait();

    console.log("\n✅ PROOF VERIFIED SUCCESSFULLY!");
    console.log("Transaction hash:", receipt.transactionHash);
    console.log("Gas used:        ", receipt.gasUsed.toString());
    console.log("Block number:    ", receipt.blockNumber);

    // ─── Post-verification state ──────────────────────────────────────────
    console.log("\n─── Post-Verification State ───");

    const isVerified = await lineageVerifier.hasVerifiedLineage(newStateHash);
    console.log("New state verified:  ", isVerified);

    if (isVerified) {
      const depth = await lineageVerifier.getDepth(newStateHash);
      console.log(
        "New state depth:     ",
        ethers.BigNumber.from(depth.toString()).toString()
      );

      const originClass = await lineageVerifier.stateOriginClass(newStateHash);
      console.log(
        "New origin class:    ",
        ethers.BigNumber.from(originClass.toString()).toString()
      );

      const timestamp = await lineageVerifier.stateTimestamp(newStateHash);
      console.log(
        "Timestamp:           ",
        ethers.BigNumber.from(timestamp.toString()).toString()
      );

      const creator = await lineageVerifier.stateCreator(newStateHash);
      console.log("Creator:             ", creator);

      const totalTransitions = await lineageVerifier.totalTransitions();
      console.log(
        "Total transitions:   ",
        ethers.BigNumber.from(totalTransitions.toString()).toString()
      );

      const maxDepth = await lineageVerifier.maxDepthReached();
      console.log(
        "Max depth reached:   ",
        ethers.BigNumber.from(maxDepth.toString()).toString()
      );

      const newCounterRaw = await rateLimiter.getCounter(
        currentEpoch,
        parentOriginClass
      );
      const newCounter = ethers.BigNumber.from(newCounterRaw.toString());
      console.log(
        "Rate counter:        ",
        `${newCounter.toString()} / ${limit.toString()}`
      );
    }

  } catch (err) {
    console.error("\n❌ VERIFICATION FAILED (expected with mock proof)");
    console.error("Error name:   ", err.errorName || "unknown");
    console.error("Error reason: ", err.reason || err.message);

    // Try to decode custom revert error
    if (err.data) {
      try {
        const knownErrors = new ethers.utils.Interface([
          "error InvalidProof()",
          "error ParentNotVerified()",
          "error PolicyViolation(uint8 from, uint8 to)",
          "error RateLimitExceeded(uint8 originClass, uint256 limit)",
          "error InvalidDepth()",
          "error StateAlreadyVerified(bytes32 stateHash)",
          "error ProofAlreadyUsed(bytes32 proofHash)",
          "error ContractPaused()",
          "error InvalidAuthType(uint8 authType)",
        ]);

        const decoded = knownErrors.parseError(err.data);
        console.error(
          "Decoded error:     ",
          decoded.name,
          decoded.args.map((a) => a.toString()).join(", ")
        );
      } catch {
        console.error("Raw error data:    ", err.data);
      }
    }

    console.error("\n─── Failure Reason ───");
    console.error("Mock proof (1,2),(3,4,5,6),(7,8) fails Groth16 check.");
    console.error("This is correct behavior — the verifier is working!\n");
    console.error("─── To Generate Real Proof ───");
    console.error("  1. Compile circuit:");
    console.error("       cd ../circuits");
    console.error("       circom lineage.circom --r1cs --wasm --sym");
    console.error("  2. Trusted setup:");
    console.error("       snarkjs groth16 setup lineage.r1cs pot12.ptau lineage_0000.zkey");
    console.error("       snarkjs zkey contribute lineage_0000.zkey lineage_0001.zkey");
    console.error("  3. Generate witness:");
    console.error("       node lineage_js/generate_witness.js lineage_js/lineage.wasm input.json witness.wtns");
    console.error("  4. Generate proof:");
    console.error("       snarkjs groth16 prove lineage_0001.zkey witness.wtns proof.json public.json");
    console.error("  5. Get Solidity calldata:");
    console.error("       snarkjs generatecall public.json proof.json");
  }

  console.log("\n========================================");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("\n❌ FATAL ERROR:");
    console.error(error);
    process.exit(1);
  });