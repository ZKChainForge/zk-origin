// scripts/deploy.js
require("dotenv").config();
const { ethers, network } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
  console.log("\n ZK-ORIGIN Contract Deployment\n");

  // Set deployer account based on network
  let deployer;
  if (network.name === "sepolia") {
    if (!process.env.PRIVATE_KEY) {
      throw new Error("PRIVATE_KEY not set in .env for Sepolia deployment");
    }
    deployer = new ethers.Wallet(process.env.PRIVATE_KEY, ethers.provider);
  } else {
    [deployer] = await ethers.getSigners();
  }

  console.log("Deploying with account:", deployer.address);
  console.log("Network:", network.name);

  const balance = await ethers.provider.getBalance(deployer.address);
  console.log("Balance:", ethers.utils.formatEther(balance), "ETH\n");

  // Deploy Groth16Verifier (or mock if not available)
  let groth16VerifierAddress;
  try {
    console.log("Deploying Groth16Verifier...");
    const Groth16Verifier = await ethers.getContractFactory("Groth16Verifier", deployer);
    const groth16Verifier = await Groth16Verifier.deploy();
    await groth16Verifier.deployed();
    groth16VerifierAddress = groth16Verifier.address;
    console.log(" Groth16Verifier deployed to:", groth16VerifierAddress);
  } catch (error) {
    console.log(" Real Groth16Verifier not found, deploying Mock...");
    
    // FIX: Use fully qualified name to avoid ambiguity
    const MockVerifier = await ethers.getContractFactory(
      "contracts/contracts/MockGroth16Verifier.sol:MockGroth16Verifier",
      deployer
    );
    
    const mockVerifier = await MockVerifier.deploy();
    await mockVerifier.deployed();
    groth16VerifierAddress = mockVerifier.address;
    console.log(" MockGroth16Verifier deployed to:", groth16VerifierAddress);
  }

  // Deploy LineageVerifier
  console.log("\nDeploying LineageVerifier...");
  const LineageVerifier = await ethers.getContractFactory("LineageVerifier", deployer);
  const lineageVerifier = await LineageVerifier.deploy(groth16VerifierAddress);
  await lineageVerifier.deployed();
  const lineageVerifierAddress = lineageVerifier.address;
  console.log(" LineageVerifier deployed to:", lineageVerifierAddress);

  // Deploy PolicyRegistry
  console.log("\nDeploying PolicyRegistry...");
  const PolicyRegistry = await ethers.getContractFactory("PolicyRegistry", deployer);
  const policyRegistry = await PolicyRegistry.deploy();
  await policyRegistry.deployed();
  const policyRegistryAddress = policyRegistry.address;
  console.log(" PolicyRegistry deployed to:", policyRegistryAddress);

  // Save deployment info
  const deploymentInfo = {
    network: network.name,
    deployer: deployer.address,
    timestamp: new Date().toISOString(),
    contracts: {
      Groth16Verifier: groth16VerifierAddress,
      LineageVerifier: lineageVerifierAddress,
      PolicyRegistry: policyRegistryAddress,
    },
  };

  const deploymentsDir = path.join(__dirname, "../deployments", network.name);
  if (!fs.existsSync(deploymentsDir)) {
    fs.mkdirSync(deploymentsDir, { recursive: true });
  }

  fs.writeFileSync(
    path.join(deploymentsDir, "addresses.json"),
    JSON.stringify(deploymentInfo, null, 2)
  );

  console.log("\n Deployment complete!");
  console.log(" Deployment info saved to:", path.join(deploymentsDir, "addresses.json"));

  console.log("\n Contract Addresses:");
  console.log("  Groth16Verifier:", groth16VerifierAddress);
  console.log("  LineageVerifier:", lineageVerifierAddress);
  console.log("  PolicyRegistry:", policyRegistryAddress);

  // Etherscan verification instructions
  if (network.name === "sepolia") {
    console.log("\n Verify on Etherscan:");
    console.log(
      `npx hardhat verify --network sepolia ${lineageVerifierAddress} ${groth16VerifierAddress}`
    );
    
    console.log("\n Live URLs:");
    console.log(`  https://sepolia.etherscan.io/address/${groth16VerifierAddress}`);
    console.log(`  https://sepolia.etherscan.io/address/${lineageVerifierAddress}`);
    console.log(`  https://sepolia.etherscan.io/address/${policyRegistryAddress}`);
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("\n Deployment failed:", error);
    process.exit(1);
  });