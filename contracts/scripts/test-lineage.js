/**
 * @title Test Lineage Script
 * @notice Verify basic lineage functionality
 */

const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
    console.log(" Testing Lineage Verification...\n");
    
    const [deployer] = await ethers.getSigners();
    console.log("Deployer:", deployer.address);
    
    // Read deployment
    const deploymentPath = path.join(__dirname, "../deployment-hardhat.json");
    const deployment = JSON.parse(fs.readFileSync(deploymentPath, "utf8"));
    const lineageVerifierAddress = deployment.contracts.LineageVerifier;
    
    const LineageVerifier = await ethers.getContractFactory("LineageVerifier");
    const lineageVerifier = LineageVerifier.attach(lineageVerifierAddress);
    
    try {
        console.log("\n Contract Status:");
        console.log("-".repeat(50));
        
        const stats = await lineageVerifier.getStats();
        console.log("Genesis Initialized:", stats.initialized);
        console.log("Total Transitions:", stats.transitions.toString());
        console.log("Max Depth Reached:", stats.maxDepth.toString());
        console.log("Contract Paused:", stats.paused);
        console.log("Current Epoch:", stats.currentEpoch.toString());
        console.log("Last Processed Epoch:", stats.lastProcessedEpoch.toString());
        
        console.log("\n Contract is ready for verification!");
        console.log("\nNext: Generate and verify ZK proofs");
        
    } catch (error) {
        console.error("\n Error:", error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch(error => {
        console.error("Fatal error:", error);
        process.exit(1);
    });