const hre = require("hardhat");
const { getContractInstance, getDeploymentMetadata, getAllContractAddresses } = require("./helpers/deployment");

async function monitorLineage() {
    console.log("╔════════════════════════════════════════════════════════╗");
    console.log("║       ZK-ORIGIN Real-Time Lineage Monitor              ║");
    console.log("╚════════════════════════════════════════════════════════╝\n");

    const [signer] = await hre.ethers.getSigners();

    try {
        // Load deployment metadata
        const metadata = getDeploymentMetadata();
        const addresses = getAllContractAddresses();

        console.log(" Monitor Address:", signer.address);
        console.log(" Network:", metadata.network);
        console.log(" Started at:", metadata.timestamp);
        console.log(" Chain ID:", metadata.chainId, "\n");

        // Get contract instances
        const lineageVerifier = await getContractInstance(hre, "LineageVerifier");
        const epochManager = await getContractInstance(hre, "EpochManager");
        const rateLimiter = await getContractInstance(hre, "RateLimiter");
        const policyRegistry = await getContractInstance(hre, "PolicyRegistry");

        console.log(" Contract Status:");
        console.log("   LineageVerifier:", addresses.LineageVerifier);
        console.log("   EpochManager:", addresses.EpochManager);
        console.log("   RateLimiter:", addresses.RateLimiter);
        console.log("   PolicyRegistry:", addresses.PolicyRegistry, "\n");

        // Get current state
        const genesisInit = await lineageVerifier.genesisInitialized();
        const currentEpoch = await epochManager.getCurrentEpoch();
        const currentPolicyRoot = await policyRegistry.getCurrentPolicyRoot();

        console.log(" Current State:");
        console.log("   Genesis Initialized:", genesisInit);
        console.log("   Current Epoch:", currentEpoch.toString());
        console.log("   Current Policy Root:", currentPolicyRoot);
        console.log("   Policy Root (stored):", metadata.policyRoot);
        console.log("   Transition Count:", metadata.transitionCount, "\n");



    } catch (error) {
        console.error("\n Error: Invalid deployment structure");
        console.error("   Details:", error.message);
    }
}

monitorLineage()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("\n Fatal error:", error.message);
        process.exit(1);
    });