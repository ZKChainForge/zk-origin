/**
 * @title Set Genesis Script
 * @notice Initialize the genesis state for LineageVerifier
 */

const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
    console.log(" Setting Genesis State...\n");
    
    const [deployer] = await ethers.getSigners();
    console.log("Deployer:", deployer.address);
    
    // Read deployment addresses
    const deploymentPath = path.join(__dirname, "../deployment-hardhat.json");
    if (!fs.existsSync(deploymentPath)) {
        console.error(" Deployment file not found. Run deploy.js first.");
        process.exit(1);
    }
    
    const deployment = JSON.parse(fs.readFileSync(deploymentPath, "utf8"));
    const lineageVerifierAddress = deployment.contracts.LineageVerifier;
    
    // Genesis parameters
    const genesisStateHash = "0x0000000000000000000000000000000000000000000000000000000000000001";
    const genesisLineageCommitment = "0x0000000000000000000000000000000000000000000000000000000000000001";
    
    // Get LineageVerifier contract
    const LineageVerifier = await ethers.getContractFactory("LineageVerifier");
    const lineageVerifier = LineageVerifier.attach(lineageVerifierAddress);
    
    console.log("\n Genesis Parameters:");
    console.log("Genesis State Hash:", genesisStateHash);
    console.log("Genesis Lineage Commitment:", genesisLineageCommitment);
    
    try {
        console.log("\n Setting genesis...");
        const tx = await lineageVerifier.setGenesis(
            genesisStateHash,
            genesisLineageCommitment
        );
        
        const receipt = await tx.wait();
        console.log(" Genesis set successfully!");
        console.log("Transaction Hash:", receipt.transactionHash);
        console.log("Block Number:", receipt.blockNumber);
        console.log("Gas Used:", receipt.gasUsed.toString());
        
        // Verify genesis was set
        const isInitialized = await lineageVerifier.genesisInitialized();
        const storedGenesisHash = await lineageVerifier.genesisStateHash();
        const storedLineageCommitment = await lineageVerifier.genesisLineageCommitment();
        
        console.log("\n Genesis Verification:");
        console.log("Initialized:", isInitialized);
        console.log("Stored Genesis Hash:", storedGenesisHash);
        console.log("Stored Lineage Commitment:", storedLineageCommitment);
        
        if (storedGenesisHash === genesisStateHash && isInitialized) {
            console.log("\n Genesis successfully initialized!");
        }
        
    } catch (error) {
        console.error("\n Error setting genesis:", error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch(error => {
        console.error("Fatal error:", error);
        process.exit(1);
    });