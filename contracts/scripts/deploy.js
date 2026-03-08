const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

// Configuration - Replace with actual values from your prover
const GENESIS_COMMITMENT = BigInt("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
const POLICY_ROOT = BigInt("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678");
const ALLOW_DUPLICATES = false;

async function main() {
  
  console.log("║                    ZK-ORIGIN DEPLOYMENT                        ║");
  

  const [deployer] = await hre.ethers.getSigners();
  const network = hre.network.name;
  const chainId = (await hre.ethers.provider.getNetwork()).chainId;

  console.log(`Network: ${network} (chainId: ${chainId})`);
  console.log(`Deployer: ${deployer.address}`);
  
  const balance = await hre.ethers.provider.getBalance(deployer.address);
  console.log(`Balance: ${hre.ethers.utils.formatEther(balance)} ETH\n`);

  // Check minimum balance
  const minBalance = hre.ethers.utils.parseEther("0.1");
  if (balance < minBalance) {
    console.error(" Insufficient balance. Need at least 0.1 ETH for deployment.");
    process.exit(1);
  }

  // ============ Deploy Groth16Verifier ============
  console.log("═ Step 1: Deploying Groth16Verifier...");
  
  const Groth16Verifier = await hre.ethers.getContractFactory("Groth16Verifier");
  const groth16Verifier = await Groth16Verifier.deploy();
  await groth16Verifier.deployed();
  const groth16Address = groth16Verifier.address;
  
  console.log(`   Groth16Verifier deployed at: ${groth16Address}`);

  // Wait for confirmations on testnet
  if (network !== "hardhat" && network !== "localhost") {
    console.log("  Waiting for confirmations...");
    await groth16Verifier.deploymentTransaction().wait(2);
    console.log("   Confirmed");
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
  await lineageVerifier.deployed();
  const lineageAddress = lineageVerifier.address;

  console.log(`   LineageVerifier deployed at: ${lineageAddress}`);

  // Wait for confirmations on testnet
  if (network !== "hardhat" && network !== "localhost") {
    console.log("  Waiting for confirmations...");
    await lineageVerifier.deploymentTransaction().wait(2);
    console.log("   Confirmed");
  }

  // ============ Verify Deployment ============
  console.log("\n═ Step 3: Verifying deployment...");
  
  const storedGenesis = await lineageVerifier.getGenesisCommitment();
  const storedPolicy = await lineageVerifier.getPolicyRoot();
  const storedVerifier = await lineageVerifier.getVerifierAddress();

  const genesisMatch = storedGenesis === GENESIS_COMMITMENT;
  const policyMatch = storedPolicy === POLICY_ROOT;
  const verifierMatch = storedVerifier === groth16Address;

  console.log(`  Genesis commitment: ${genesisMatch ? "✓" : "✗"}`);
  console.log(`  Policy root: ${policyMatch ? "✓" : "✗"}`);
  console.log(`  Verifier address: ${verifierMatch ? "✓" : "✗"}`);

  if (!genesisMatch || !policyMatch || !verifierMatch) {
    console.error("\n Deployment verification failed!");
    process.exit(1);
  }

  // ============ Save Deployment Info ============
  console.log("\n═ Step 4: Saving deployment info...");

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
      Groth16Verifier: groth16Verifier.deploymentTransaction().hash,
      LineageVerifier: lineageVerifier.deploymentTransaction().hash
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
  console.log(`   Saved to ${addressesPath}`);

  // Save deployment log
  const logPath = path.join(deploymentsDir, "deployment-log.txt");
  const logContent = `
ZK-ORIGIN Deployment Log
========================
Network: ${network}
Chain ID: ${chainId}
Deployer: ${deployer.address}
Timestamp: ${deploymentInfo.deployedAt}

Contracts:
- Groth16Verifier: ${groth16Address}
- LineageVerifier: ${lineageAddress}

Transactions:
- Groth16Verifier: ${deploymentInfo.transactions.Groth16Verifier}
- LineageVerifier: ${deploymentInfo.transactions.LineageVerifier}

Configuration:
- Genesis: ${deploymentInfo.config.genesisCommitment}
- Policy: ${deploymentInfo.config.policyRoot}
- Allow Duplicates: ${ALLOW_DUPLICATES}
`;
  fs.appendFileSync(logPath, logContent);
  console.log(`   Appended to ${logPath}`);

  
  console.log(`  Network:          ${network.padEnd(43)}║`);
  console.log(`  Groth16Verifier:  ${groth16Address}  ║`);
  console.log(`  LineageVerifier:  ${lineageAddress}  ║`);
  

  // ============ Etherscan Verification Reminder ============
  if (network === "sepolia") {
    console.log(" To verify on Etherscan, run:");
    console.log(`   npx hardhat verify --network sepolia ${groth16Address}`);
    console.log(`   npx hardhat verify --network sepolia ${lineageAddress} "${GENESIS_COMMITMENT}" "${POLICY_ROOT}" "${groth16Address}" ${ALLOW_DUPLICATES}\n`);
  }

  return deploymentInfo;
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });

module.exports = { main };