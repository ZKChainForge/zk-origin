const hre = require("hardhat");
const fs = require("fs");

async function main() {
    console.log("Testing ZK-ORIGIN proof submission...\n");

    // Load deployment addresses
    const deployment = JSON.parse(fs.readFileSync("./deployment-complete.json", "utf8"));
    console.log("Loaded deployment addresses\n");

    // Load proof data
    const proof = JSON.parse(fs.readFileSync("../circuits/proof.json", "utf8"));
    const publicSignals = JSON.parse(fs.readFileSync("../circuits/public.json", "utf8"));

    console.log("Proof data:");
    console.log("- Public signals count:", publicSignals.length);

    // Get signer
    const [signer] = await hre.ethers.getSigners();
    console.log("\nUsing signer:", signer.address);

    // Get LineageVerifier contract
    const LineageVerifier = await hre.ethers.getContractFactory("LineageVerifier");
    const lineageVerifier = LineageVerifier.attach(deployment.contracts.LineageVerifier);

    console.log("LineageVerifier at:", deployment.contracts.LineageVerifier);

    // Try to read genesis state directly
    try {
        const genesisHash = await lineageVerifier.genesisStateHash();
        console.log("\nGenesis state hash:", genesisHash);
        
        if (genesisHash === "0x" + "0".repeat(64)) {
            console.log("Genesis is zero - need to initialize");
            
            console.log("\n Setting genesis...");
            const genesisStateHash = "0x" + "0".repeat(64);
            const genesisLineageCommitment = deployment.config.genesisLineage;
            
            const tx = await lineageVerifier.setGenesis(genesisStateHash, genesisLineageCommitment);
            console.log("Transaction sent:", tx.hash);
            await tx.wait();
            console.log(" Genesis set");
        } else {
            console.log("Genesis already initialized");
        }
    } catch (error) {
        console.log("\nCannot read genesis, assuming not initialized");
        console.log("\n Setting genesis...");
        
        try {
            const genesisStateHash = "0x" + "0".repeat(64);
            const genesisLineageCommitment = deployment.config.genesisLineage;
            
            const tx = await lineageVerifier.setGenesis(genesisStateHash, genesisLineageCommitment);
            console.log("Transaction sent:", tx.hash);
            await tx.wait();
            console.log(" Genesis set");
        } catch (setError) {
            console.error("Failed to set genesis:", setError.message);
        }
    }

    // Prepare proof for Solidity
    const pA = [proof.pi_a[0], proof.pi_a[1]];
    const pB = [
        [proof.pi_b[0][1], proof.pi_b[0][0]],
        [proof.pi_b[1][1], proof.pi_b[1][0]]
    ];
    const pC = [proof.pi_c[0], proof.pi_c[1]];

    console.log("\n Public signals:");
    publicSignals.forEach((sig, i) => {
        console.log(`  [${i}] ${sig}`);
    });

    console.log("\n Submitting proof to contract...");
    
    try {
        const tx = await lineageVerifier.verifyLineage(pA, pB, pC, publicSignals, {
            gasLimit: 5000000
        });
        console.log("Transaction sent:", tx.hash);
        
        const receipt = await tx.wait();
        console.log("\n Proof verified on-chain!");
        console.log("Gas used:", receipt.gasUsed.toString());
        console.log("Block:", receipt.blockNumber);
        
        // Parse events
        if (receipt.events && receipt.events.length > 0) {
            console.log("\n Events emitted:");
            receipt.events.forEach((event, i) => {
                console.log(`  [${i}] ${event.event || 'Unknown'}`);
            });
        }
        
        // Check the new state
        const newStateHash = "0x" + BigInt(publicSignals[4]).toString(16).padStart(64, "0");
        
        try {
            const stateLineage = await lineageVerifier.stateLineage(newStateHash);
            const stateDepth = await lineageVerifier.stateDepth(newStateHash);
            const verifiedStates = await lineageVerifier.verifiedStates(newStateHash);
            const stateOriginClass = await lineageVerifier.stateOriginClass(newStateHash);
            
            console.log("\n New state details:");
            console.log("  State hash:", newStateHash);
            console.log("  Lineage:", stateLineage);
            console.log("  Depth:", stateDepth.toString());
            console.log("  Verified:", verifiedStates);
            console.log("  Origin class:", stateOriginClass);
        } catch (stateError) {
            console.log("\nCould not read state details:", stateError.message);
        }
        
        // Get stats
        try {
            const totalTransitions = await lineageVerifier.totalTransitions();
            const maxDepthReached = await lineageVerifier.maxDepthReached();
            
            console.log("\n Contract stats:");
            console.log("  Total transitions:", totalTransitions.toString());
            console.log("  Max depth reached:", maxDepthReached.toString());
        } catch (statsError) {
            console.log("\nCould not read stats:", statsError.message);
        }
        
    } catch (error) {
        console.error("\n Proof verification failed:");
        console.error("Error:", error.message);
        
        if (error.data) {
            console.error("Data:", error.data);
        }
        
        if (error.transaction) {
            console.error("\nTransaction that failed:");
            console.error("  To:", error.transaction.to);
            console.error("  Data:", error.transaction.data.substring(0, 66) + "...");
        }
        
        process.exit(1);
    }


}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });