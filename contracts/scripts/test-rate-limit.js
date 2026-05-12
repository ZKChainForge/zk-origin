const hre = require("hardhat");
const { ethers } = hre;
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("========================================");
  console.log("SUBMIT PROOF TEST");
  console.log("========================================\n");

  // Load deployment
  const deploymentPath = path.join(
    __dirname,
    "../deployments/localhost.json"
  );

  if (!fs.existsSync(deploymentPath)) {
    console.error("ERROR: Run deploy-local.js first");
    process.exit(1);
  }

  const deployment = JSON.parse(
    fs.readFileSync(deploymentPath, "utf8")
  );

  const [deployer] = await ethers.getSigners();
  console.log("Submitter:", deployer.address);

  // Get LineageVerifier instance
  const lineageVerifier = await ethers.getContractAt(
    "LineageVerifier",
    deployment.contracts.LineageVerifier
  );

  // Get the function signature to check expected parameters
  const verifyLineageFragment = lineageVerifier.interface.getFunction(
    "verifyLineage"
  );

  console.log("\nverifyLineage signature:");
  console.log(
    verifyLineageFragment.inputs
      .map((input, i) => `  [${i}] ${input.type} ${input.name}`)
      .join("\n")
  );

  // ─── Prepare proof data ────────────────────────────────────────────────────
  // Mock Groth16 proof (in production, generate from snarkjs)
  
  // Groth16 proof format for Solidity:
  // - a: [uint256, uint256]
  // - b: [[uint256, uint256], [uint256, uint256]]
  // - c: [uint256, uint256]
  
  const proof = {
    a: [
      "0x0000000000000000000000000000000000000000000000000000000000000001",
      "0x0000000000000000000000000000000000000000000000000000000000000002"
    ],
    b: [
      [
        "0x0000000000000000000000000000000000000000000000000000000000000003",
        "0x0000000000000000000000000000000000000000000000000000000000000004"
      ],
      [
        "0x0000000000000000000000000000000000000000000000000000000000000005",
        "0x0000000000000000000000000000000000000000000000000000000000000006"
      ]
    ],
    c: [
      "0x0000000000000000000000000000000000000000000000000000000000000007",
      "0x0000000000000000000000000000000000000000000000000000000000000008"
    ]
  };

  // Public signals for lineage verification
  const parentStateHash = deployment.genesis.genesisStateHash;
  const newStateHash = ethers.utils.keccak256(
    ethers.utils.toUtf8Bytes("NEW_STATE_1")
  );
  const parentCommitment = deployment.genesis.genesisLineageCommitment;
  const newCommitment = ethers.utils.keccak256(
    ethers.utils.toUtf8Bytes("NEW_COMMITMENT_1")
  );

  const parentOriginClass = 0; // Genesis
  const newOriginClass = 1;    // User
  const parentDepth = 0;       // Genesis has depth 0

  const publicSignals = [
    parentStateHash,
    newStateHash,
    parentCommitment,
    newCommitment,
    parentOriginClass,
    newOriginClass,
    parentDepth
  ];

  console.log("\n─── Proof Details ───");
  console.log("Parent state:  ", parentStateHash);
  console.log("New state:     ", newStateHash);
  console.log("Transition:    ", `${parentOriginClass} (Genesis) → ${newOriginClass} (User)`);
  console.log("Parent depth:  ", parentDepth);
  console.log("New depth:     ", parentDepth + 1);

  // Authorization proof hash (optional, use zero hash for now)
  const authProofHash = ethers.constants.HashZero;

  // Metadata (optional)
  const metadata = [];

  console.log("\n─── Submitting Proof ───");

  try {
    // Check what parameters verifyLineage actually expects
    const paramCount = verifyLineageFragment.inputs.length;
    
    console.log(`Function expects ${paramCount} parameters`);

    let tx;

    // Try different call formats based on contract signature
    if (paramCount === 5) {
      // Format: verifyLineage(proof, publicSignals, authProofHash, metadata, extraParam)
      tx = await lineageVerifier.verifyLineage(
        proof,
        publicSignals,
        authProofHash,
        metadata,
        ethers.constants.AddressZero, // possible extra param
        { gasLimit: 3000000 }
      );
    } else if (paramCount === 4) {
      // Format: verifyLineage(proof, publicSignals, authProofHash, metadata)
      tx = await lineageVerifier.verifyLineage(
        proof,
        publicSignals,
        authProofHash,
        metadata,
        { gasLimit: 3000000 }
      );
    } else if (paramCount === 3) {
      // Format: verifyLineage(proof, publicSignals, metadata)
      tx = await lineageVerifier.verifyLineage(
        proof,
        publicSignals,
        metadata,
        { gasLimit: 3000000 }
      );
    } else if (paramCount === 2) {
      // Format: verifyLineage(proof, publicSignals)
      tx = await lineageVerifier.verifyLineage(
        proof,
        publicSignals,
        { gasLimit: 3000000 }
      );
    } else if (paramCount === 7) {
      // Flattened format: verifyLineage(a[2], b[2][2], c[2], publicSignals)
      tx = await lineageVerifier.verifyLineage(
        proof.a,
        proof.b,
        proof.c,
        publicSignals,
        authProofHash,
        metadata,
        ethers.constants.AddressZero,
        { gasLimit: 3000000 }
      );
    } else {
      throw new Error(
        `Unexpected parameter count: ${paramCount}. ` +
        `Check LineageVerifier.sol verifyLineage() signature.`
      );
    }

    console.log("Transaction sent, waiting for confirmation...");
    const receipt = await tx.wait();

    console.log("\n✅ PROOF VERIFIED SUCCESSFULLY!");
    console.log("Transaction hash:", receipt.transactionHash);
    console.log("Gas used:        ", receipt.gasUsed.toString());

    // Verify the new state was registered
    console.log("\n─── Verification Check ───");
    
    const isVerified = await lineageVerifier.hasVerifiedLineage(newStateHash);
    console.log("New state verified:       ", isVerified);

    if (isVerified) {
      const depth = await lineageVerifier.getDepth(newStateHash);
      console.log("New state depth:          ", depth.toString());

      const originClass = await lineageVerifier.stateOriginClass(newStateHash);
      console.log("New state origin class:   ", originClass.toString());

      const totalTransitions = await lineageVerifier.totalTransitions();
      console.log("Total transitions so far: ", totalTransitions.toString());
    }

  } catch (err) {
    console.error("\n❌ VERIFICATION FAILED");
    console.error("Error:", err.reason || err.message);
    
    if (err.error && err.error.data) {
      console.error("Error data:", err.error.data);
    }

    // Try to decode revert reason
    if (err.data) {
      try {
        const decodedError = lineageVerifier.interface.parseError(err.data);
        console.error("Decoded error:", decodedError);
      } catch (decodeErr) {
        console.error("Could not decode error");
      }
    }

    // Common errors and solutions
    console.error("\nCommon causes:");
    console.error("  • Mock proof fails Groth16 verification (expected with dummy data)");
    console.error("  • Parent state not verified");
    console.error("  • Policy transition not allowed");
    console.error("  • Rate limit exceeded");
    console.error("  • Invalid proof format");
    
    console.error("\nTo fix:");
    console.error("  • Generate real proof with snarkjs");
    console.error("  • Or check contract's verifyLineage() signature");
  }

  console.log("\n========================================");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });