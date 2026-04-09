const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
    console.log(" ZK-ORIGIN Contract Statistics\n");

    // Load deployment
    const deploymentFile = path.join(__dirname, "../deployment-complete.json");
    if (!fs.existsSync(deploymentFile)) {
        throw new Error("No deployment found. Run deploy-complete.js first.");
    }

    const deployment = JSON.parse(fs.readFileSync(deploymentFile, "utf8"));

    // Get contracts
    const lineageVerifier = await hre.ethers.getContractAt(
        "LineageVerifier",
        deployment.lineageVerifier
    );

    const epochManager = await hre.ethers.getContractAt(
        "EpochManager",
        deployment.epochManager
    );

    const policyRegistry = await hre.ethers.getContractAt(
        "PolicyRegistry",
        deployment.policyRegistry
    );

    // Get stats
    const stats = await lineageVerifier.getStats();
    const currentEpoch = await epochManager.getCurrentEpoch();
    const currentPolicyRoot = await policyRegistry.getCurrentPolicyRoot();


    console.log("   Network:", deployment.network);
    console.log("   Deployed:", deployment.timestamp);
    console.log("   Deployer:", deployment.deployer, "\n");


    console.log("   Total transitions:", stats.transitions.toString());
    console.log("   Max depth reached:", stats.maxDepth.toString());
    console.log("   Genesis initialized:", stats.initialized);
    console.log("   Contract paused:", stats.isPaused);
    console.log("   Current epoch:", stats.currentEpoch.toString(), "\n");


    console.log("   Current epoch:", currentEpoch.toString());
    const timeToNext = await epochManager.timeUntilNextEpoch();
    console.log("   Time to next epoch:", timeToNext.toString(), "seconds\n");

    
    console.log("   Active policy root:", currentPolicyRoot);
    const policyCount = await policyRegistry.policyCount();
    console.log("   Total policies:", policyCount.toString(), "\n");

    
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });