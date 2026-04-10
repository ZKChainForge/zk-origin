const hre = require("hardhat");
const { getContractInstance, getDeploymentMetadata, getAllContractAddresses } = require("./helpers/deployment");

async function checkStats() {
    console.log("╔════════════════════════════════════════════════════════╗");
    console.log("║       ZK-ORIGIN Contract Statistics Report             ║");
    console.log("╚════════════════════════════════════════════════════════╝\n");

    try {
        // Load deployment info
        const metadata = getDeploymentMetadata();
        const addresses = getAllContractAddresses();

        console.log(" Deployment Information:");
        console.log("   Network:", metadata.network);
        console.log("   Chain ID:", metadata.chainId);
        console.log("   Deployer:", metadata.deployer);
        console.log("   Timestamp:", metadata.timestamp, "\n");

        console.log(" Contract Addresses:");
        console.log("   LineageVerifier:", addresses.LineageVerifier);
        console.log("   Groth16Verifier:", addresses.Groth16Verifier);
        console.log("   EpochManager:", addresses.EpochManager);
        console.log("   RateLimiter:", addresses.RateLimiter);
        console.log("   AuthorizationVerifier:", addresses.AuthorizationVerifier);
        console.log("   PolicyRegistry:", addresses.PolicyRegistry);
        console.log("   BatchVerifier:", addresses.BatchVerifier, "\n");

        // Get contract instances
        const lineageVerifier = await getContractInstance(hre, "LineageVerifier");
        const epochManager = await getContractInstance(hre, "EpochManager");
        const policyRegistry = await getContractInstance(hre, "PolicyRegistry");

        console.log(" Contract Statistics:");
        
        // LineageVerifier stats
        const maxDepth = await lineageVerifier.MAX_DEPTH();
        const version = await lineageVerifier.VERSION();
        const genesisInit = await lineageVerifier.genesisInitialized();
        
        console.log("\n   LineageVerifier:");
        console.log("      MAX_DEPTH:", maxDepth.toString());
        console.log("      VERSION:", version.toString());
        console.log("      Genesis Initialized:", genesisInit);
        console.log("      Genesis State Hash:", metadata.genesisStateHash);
        console.log("      Genesis Lineage Commitment:", metadata.genesisLineageCommitment);

        // EpochManager stats
        const currentEpoch = await epochManager.getCurrentEpoch();
        const epochDuration = await epochManager.EPOCH_DURATION();
        
        console.log("\n   EpochManager:");
        console.log("      Current Epoch:", currentEpoch.toString());
        console.log("      Epoch Duration:", epochDuration.toString(), "seconds");
        console.log("      Hours per epoch:", (epochDuration.toNumber() / 3600).toFixed(2));

        // PolicyRegistry stats
        const currentPolicyRoot = await policyRegistry.getCurrentPolicyRoot();
        
        console.log("\n   PolicyRegistry:");
        console.log("      Current Policy Root:", currentPolicyRoot);
        console.log("      Policy Root (stored):", metadata.policyRoot);
        console.log("      Transition Count:", metadata.transitionCount);

        // System Configuration
        console.log("\n  System Configuration:");
        console.log("   Policy Root:", metadata.policyRoot);
        console.log("   Transition Count:", metadata.transitionCount);
        console.log("   Allow Duplicate States:", "false (default)");

        

    } catch (error) {
        console.error("\n Error: Invalid deployment structure");
        console.error("   Details:", error.message);
  
    }
}

checkStats()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("\n Fatal error:", error.message);
        process.exit(1);
    });