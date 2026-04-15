/**
 * Verify State: Query contract and verify state is in lineage
 */

const { ethers } = require("hardhat");

async function main() {
    console.log(`\n${'═'.repeat(60)}`);
    console.log(" STATE VERIFICATION");
    console.log(`${'═'.repeat(60)}\n`);
    
    // Get contract
    const lineageVerifier = await ethers.getContractAt(
        "LineageVerifier",
        process.env.LINEAGE_VERIFIER || "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9"
    );
    
    // Get latest verified state
    console.log(" Querying verified states...\n");
    
    const stats = await lineageVerifier.getStats();
    console.log(` Contract Statistics:`);
    console.log(`   Total transitions verified: ${stats.transitions}`);
    console.log(`   Max depth reached: ${stats.maxDepth}`);
    console.log(`   Genesis initialized: ${stats.initialized}`);
    console.log(`   Contract paused: ${stats.isPaused}`);
    console.log(`   Current epoch: ${stats.currentEpoch}`);
    
    // Get genesis state info
    if (stats.initialized) {
        const genesisHash = await lineageVerifier.genesisStateHash();
        const genesisLineage = await lineageVerifier.getLineage(genesisHash);
        
        console.log(`\n Genesis State:`);
        console.log(`   Hash: ${genesisHash.slice(0, 10)}...`);
        console.log(`   Lineage: ${genesisLineage.slice(0, 10)}...`);
        console.log(`   Verified: ${await lineageVerifier.hasVerifiedLineage(genesisHash)}`);
    }
    
    console.log(`\n${'═'.repeat(60)}`);
    console.log(" VERIFICATION COMPLETE");
    console.log(`${'═'.repeat(60)}\n`);
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });