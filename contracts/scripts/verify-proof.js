const { ethers, network } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
    console.log(" ZK-ORIGIN Proof Verification");
    
    
    // Load deployment info
    const deploymentsPath = path.join(__dirname, "../deployments", network.name, "addresses.json");
    if (!fs.existsSync(deploymentsPath)) {
        console.error("❌ Deployment not found. Run deploy.js first.");
        process.exit(1);
    }
    
    const deployment = JSON.parse(fs.readFileSync(deploymentsPath));
    console.log("Network:", network.name);
    console.log("LineageVerifier:", deployment.contracts.LineageVerifier);
    
    // Load proofs
    const proofsPath = path.join(__dirname, "../../circuits/build/proofs.json");
    if (!fs.existsSync(proofsPath)) {
        console.error("❌ Proofs not found. Run circuits/scripts/prove.js first.");
        process.exit(1);
    }
    
    const proofs = JSON.parse(fs.readFileSync(proofsPath));
    console.log(`\nFound ${proofs.length} proofs to verify\n`);
    
    // Get contract instance
    const lineageVerifier = await ethers.getContractAt(
        "LineageVerifier",
        deployment.contracts.LineageVerifier
    );
    
    // Check if genesis is set
    const genesisInitialized = await lineageVerifier.genesisInitialized();
    if (!genesisInitialized) {
        console.log("Setting genesis state...");
        
        // Use the first proof's output as genesis
        const genesisHash = ethers.randomBytes(32);
        const genesisLineage = ethers.randomBytes(32);
        
        const tx = await lineageVerifier.setGenesis(genesisHash, genesisLineage);
        await tx.wait();
        console.log(" Genesis set\n");
    }
    
    // Verify each proof
    for (let i = 0; i < proofs.length; i++) {
        const { proof, publicSignals } = proofs[i];
        
        console.log(`Verifying proof ${i + 1}/${proofs.length}...`);
        
        // Format proof for contract
        const pA = [BigInt(proof.pi_a[0]), BigInt(proof.pi_a[1])];
        const pB = [
            [BigInt(proof.pi_b[0][1]), BigInt(proof.pi_b[0][0])],
            [BigInt(proof.pi_b[1][1]), BigInt(proof.pi_b[1][0])]
        ];
        const pC = [BigInt(proof.pi_c[0]), BigInt(proof.pi_c[1])];
        const signals = [BigInt(publicSignals[0]), BigInt(publicSignals[1])];
        
        try {
            const tx = await lineageVerifier.verifyLineage(pA, pB, pC, signals);
            const receipt = await tx.wait();
            
            console.log(`   Verified! Gas used: ${receipt.gasUsed.toString()}`);
        } catch (error) {
            console.log(`    Failed: ${error.message}`);
        }
    }
    
    console.log("\n Verification complete!");
    console.log("Total transitions:", (await lineageVerifier.totalTransitions()).toString());
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });