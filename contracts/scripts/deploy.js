const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

// Configuration
const GENESIS_COMMITMENT = BigInt("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
const POLICY_ROOT = BigInt("0x00abcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678");
const ALLOW_DUPLICATES = false;

async function main() {
  console.log("╔════════════════════════════════════════════════════════════════╗");
  console.log("║                    ZK-ORIGIN DEPLOYMENT                        ║");
  console.log("╚════════════════════════════════════════════════════════════════╝\n");

  const [deployer] = await hre.ethers.getSigners();
  const network = hre.network.name;
  
  // Get chainId - compatible with both ethers v5 and v6
  const networkData = await hre.ethers.provider.getNetwork();
  const chainId = networkData.chainId;

  console.log(`Network: ${network} (chainId: ${chainId})`);
  console.log(`Deployer: ${deployer.address}`);
  
  const balance = await hre.ethers.provider.getBalance(deployer.address);
  
  // Compatible with both ethers v5 and v6
  const formatEther = hre.ethers.formatEther || hre.ethers.utils.formatEther;
  const parseEther = hre.ethers.parseEther || hre.ethers.utils.parseEther;
  
  console.log(`Balance: ${formatEther(balance)} ETH\n`);

  // Check minimum balance
  const minBalance = parseEther("0.1");
  if (balance < minBalance) {
    console.error(" Insufficient balance. Need at least 0.1 ETH for deployment.");
    process.exit(1);
  }

  // ============ Deploy Groth16Verifier ============
  console.log("═ Step 1: Deploying Groth16Verifier...");
  
  const Groth16Verifier = await hre.ethers.getContractFactory("Groth16Verifier");
  const groth16Verifier = await Groth16Verifier.deploy();
  
  // Compatible with both ethers v5 and v6
  if (groth16Verifier.waitForDeployment) {
    await groth16Verifier.waitForDeployment();
  } else {
    await groth16Verifier.deployed();
  }
  
  // Get address - compatible with both versions
  const groth16Address = groth16Verifier.address || await groth16Verifier.getAddress();
  
  console.log(`    Groth16Verifier deployed at: ${groth16Address}`);

  // Wait for confirmations on testnet
  if (network !== "hardhat" && network !== "localhost") {
    console.log("    Waiting for confirmations...");
    const deployTx = groth16Verifier.deployTransaction || groth16Verifier.deploymentTransaction?.();
    if (deployTx) {
      await deployTx.wait(2);
    }
    console.log("    Confirmed");
  }

  // ============ Deploy LineageVerifier ============
  console.log("\n═ Step 2: Deploying LineageVerifier...");
  console.log(`  Genesis: 0x${GENESIS_COMMITMENT.toString(16).padStart(64, '0')}`);
  console.log(`  Policy:  0x${POLICY_ROOT.toString(16).padStart(64, '0')}`);
  console.log(`  Verifier: ${groth16Address}`);
  console.log(`  Allow Duplicates: ${ALLOW_DUPLICATES}`);

  const LineageVerifier = await hre.ethers.getContractFactory("LineageVerifier");
  const lineageVerifier = await LineageVerifier.deploy(
    GENESIS_COMMITMENT,
    POLICY_ROOT,
    groth16Address,
    ALLOW_DUPLICATES
  );
  
  // Compatible with both ethers v5 and v6
  if (lineageVerifier.waitForDeployment) {
    await lineageVerifier.waitForDeployment();
  } else {
    await lineageVerifier.deployed();
  }
  
  const lineageAddress = lineageVerifier.address || await lineageVerifier.getAddress();

  console.log(`    LineageVerifier deployed at: ${lineageAddress}`);

  // Wait for confirmations on testnet
  if (network !== "hardhat" && network !== "localhost") {
    console.log("    Waiting for confirmations...");
    const deployTx = lineageVerifier.deployTransaction || lineageVerifier.deploymentTransaction?.();
    if (deployTx) {
      await deployTx.wait(2);
    }
    console.log("    Confirmed");
  }

  // ============ Verify Deployment ============
  console.log("\n═ Step 3: Verifying deployment...");
  
  const storedGenesis = await lineageVerifier.getGenesisCommitment();
  const storedPolicy = await lineageVerifier.getPolicyRoot();
  const storedVerifier = await lineageVerifier.getVerifierAddress();

  // Convert to BigInt for comparison
  const storedGenesisBigInt = BigInt(storedGenesis.toString());
  const storedPolicyBigInt = BigInt(storedPolicy.toString());

  const genesisMatch = storedGenesisBigInt === GENESIS_COMMITMENT;
  const policyMatch = storedPolicyBigInt === POLICY_ROOT;
  const verifierMatch = storedVerifier.toLowerCase() === groth16Address.toLowerCase();

  console.log(`  Genesis commitment: ${genesisMatch ? "✓" : "✗"}`);
  console.log(`  Policy root: ${policyMatch ? "✓" : "✗"}`);
  console.log(`  Verifier address: ${verifierMatch ? "✓" : "✗"}`);

  if (!genesisMatch || !policyMatch || !verifierMatch) {
    console.error("\n Deployment verification failed!");
    console.log(`  Expected genesis: ${GENESIS_COMMITMENT}, got: ${storedGenesisBigInt}`);
    console.log(`  Expected policy: ${POLICY_ROOT}, got: ${storedPolicyBigInt}`);
    console.log(`  Expected verifier: ${groth16Address}, got: ${storedVerifier}`);
    process.exit(1);
  }

  // ============ Save Deployment Info ============
  console.log("\n═ Step 4: Saving deployment info...");

  const deployTxHash1 = (groth16Verifier.deployTransaction?.hash) || 
                        (groth16Verifier.deploymentTransaction?.()?.hash) || 
                        "N/A";
  const deployTxHash2 = (lineageVerifier.deployTransaction?.hash) || 
                        (lineageVerifier.deploymentTransaction?.()?.hash) || 
                        "N/A";

  const deploymentInfo = {
    network: network,
    chainId: Number(chainId),
    deployer: deployer.address,
    deployedAt: new Date().toISOString(),
    contracts: {
      Groth16Verifier: groth16Address,
      LineageVerifier: lineageAddress
    },
    config: {
      genesisCommitment: "0x" + GENESIS_COMMITMENT.toString(16).padStart(64, '0'),
      policyRoot: "0x" + POLICY_ROOT.toString(16).padStart(64, '0'),
      allowDuplicates: ALLOW_DUPLICATES
    },
    transactions: {
      Groth16Verifier: deployTxHash1,
      LineageVerifier: deployTxHash2
    }
  };

  // Create deployments directory if needed
  const deploymentsDir = path.join(__dirname, "..", "deployments", network);
  if (!fs.existsSync(deploymentsDir)) {
    fs.mkdirSync(deploymentsDir, { recursive: true });
  }

  // Save addresses
  const addressesPath = path.join(deploymentsDir, "addresses.json");
  fs.writeFileSync(addressesPath, JSON.stringify(deploymentInfo, null, 2));
  console.log(`    Saved to ${addressesPath}`);

  console.log("\n╔════════════════════════════════════════════════════════════════╗");
  console.log("║                    DEPLOYMENT COMPLETE                           ║");
  console.log("╠════════════════════════════════════════════════════════════════  ╣");
  console.log(`║  Network:          ${network.padEnd(43)}                         ║`);
  console.log(`║  Groth16Verifier:  ${groth16Address}                             ║`);
  console.log(`║  LineageVerifier:  ${lineageAddress}                             ║`);
  console.log("╚════════════════════════════════════════════════════════════════╝\n");

  return deploymentInfo;
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });

module.exports = { main };