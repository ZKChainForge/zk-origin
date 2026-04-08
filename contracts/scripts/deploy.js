const hre = require("hardhat");

async function main() {
    console.log(" Deploying ZK-ORIGIN Contracts...\n");
    
    // 1. Deploy Groth16Verifier (use generated contract)
    console.log("1. Deploying Groth16Verifier...");
    const Groth16Verifier = await hre.ethers.getContractFactory("Groth16Verifier");
    const groth16Verifier = await Groth16Verifier.deploy();
    await groth16Verifier.deployed();
    console.log("    Groth16Verifier:", groth16Verifier.address);
    
    // 2. Deploy EpochManager
    console.log("2. Deploying EpochManager...");
    const EpochManager = await hre.ethers.getContractFactory("EpochManager");
    const epochManager = await EpochManager.deploy();
    await epochManager.deployed();
    console.log("    EpochManager:", epochManager.address);
    
    // 3. Deploy RateLimiter
    console.log("3. Deploying RateLimiter...");
    const RateLimiter = await hre.ethers.getContractFactory("RateLimiter");
    const rateLimiter = await RateLimiter.deploy();
    await rateLimiter.deployed();
    console.log("    RateLimiter:", rateLimiter.address);
    
    // 4. Deploy PolicyRegistry
    console.log("4. Deploying PolicyRegistry...");
    const PolicyRegistry = await hre.ethers.getContractFactory("PolicyRegistry");
    const policyRegistry = await PolicyRegistry.deploy();
    await policyRegistry.deployed();
    console.log("    PolicyRegistry:", policyRegistry.address);
    
    // 5. Deploy LineageVerifier
    console.log("5. Deploying LineageVerifier...");
    const LineageVerifier = await hre.ethers.getContractFactory("LineageVerifier");
    
    // Use your circuit's genesis commitment and policy root
    const GENESIS_COMMITMENT = "0x0000000000000000000000000000000000000000000000000000000000000000";
    const POLICY_ROOT = "0x0000000000000000000000000000000000000000000000000000000000000000";
    
    const lineageVerifier = await LineageVerifier.deploy(
        groth16Verifier.address,
        epochManager.address,
        rateLimiter.address,
        GENESIS_COMMITMENT,
        POLICY_ROOT,
        false // allowDuplicates
    );
    await lineageVerifier.deployed();
    console.log("    LineageVerifier:", lineageVerifier.address);
    
    // 6. Deploy BatchVerifier
    console.log("6. Deploying BatchVerifier...");
    const BatchVerifier = await hre.ethers.getContractFactory("BatchVerifier");
    const batchVerifier = await BatchVerifier.deploy(lineageVerifier.address);
    await batchVerifier.deployed();
    console.log("    BatchVerifier:", batchVerifier.address);
    
    // 7. Set Genesis
    console.log("\n7. Setting Genesis State...");
    const GENESIS_STATE_HASH = "0x0000000000000000000000000000000000000000000000000000000000000001";
    await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_COMMITMENT);
    console.log("    Genesis initialized");
    
    // Save addresses
    const addresses = {
        groth16Verifier: groth16Verifier.address,
        epochManager: epochManager.address,
        rateLimiter: rateLimiter.address,
        policyRegistry: policyRegistry.address,
        lineageVerifier: lineageVerifier.address,
        batchVerifier: batchVerifier.address,
    };
    
    const fs = require('fs');
    fs.writeFileSync('deployment.json', JSON.stringify(addresses, null, 2));
    
    console.log("\n DEPLOYMENT COMPLETE!");
    console.log("\nDeployed Addresses:");
    console.log(JSON.stringify(addresses, null, 2));
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});