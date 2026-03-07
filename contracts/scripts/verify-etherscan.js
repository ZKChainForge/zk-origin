const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
  const network = hre.network.name;
  console.log(`\nVerifying contracts on ${network}...\n`);

  // Load deployment info
  const deploymentsPath = path.join(__dirname, "..", "deployments", network, "addresses.json");
  
  if (!fs.existsSync(deploymentsPath)) {
    console.error(`No deployment found for ${network}`);
    console.error(`Run 'npx hardhat run scripts/deploy.js --network ${network}' first`);
    process.exit(1);
  }

  const deployment = JSON.parse(fs.readFileSync(deploymentsPath, "utf8"));
  const { Groth16Verifier, LineageVerifier } = deployment.contracts;
  const config = deployment.config;

  // Parse config values
  const genesisCommitment = BigInt(config.genesisCommitment);
  const policyRoot = BigInt(config.policyRoot);

  console.log("═ Verifying Groth16Verifier...");
  try {
    await hre.run("verify:verify", {
      address: Groth16Verifier,
      constructorArguments: [],
    });
    console.log("   Groth16Verifier verified");
  } catch (error) {
    if (error.message.includes("Already Verified")) {
      console.log("   Groth16Verifier already verified");
    } else {
      console.error("   Failed:", error.message);
    }
  }

  console.log("\n═ Verifying LineageVerifier...");
  try {
    await hre.run("verify:verify", {
      address: LineageVerifier,
      constructorArguments: [
        genesisCommitment,
        policyRoot,
        Groth16Verifier,
        config.allowDuplicates
      ],
    });
    console.log("   LineageVerifier verified");
  } catch (error) {
    if (error.message.includes("Already Verified")) {
      console.log("  LineageVerifier already verified");
    } else {
      console.error("   Failed:", error.message);
    }
  }

  console.log("\n Verification complete!");
  console.log(`View on Etherscan:`);
  console.log(`  Groth16Verifier: https://${network}.etherscan.io/address/${Groth16Verifier}`);
  console.log(`  LineageVerifier: https://${network}.etherscan.io/address/${LineageVerifier}\n`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });