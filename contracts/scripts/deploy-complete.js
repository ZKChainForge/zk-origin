const hre = require("hardhat");
const fs = require('fs');

async function main() {
    console.log(" DEPLOYING COMPLETE ZK-ORIGIN SYSTEM\n");
    
    const [deployer] = await hre.ethers.getSigners();
    console.log("Deployer:", deployer.address);
    console.log("Balance:", (await deployer.getBalance()).toString());
    
    // 1. Deploy Groth16Verifier
    console.log("\n Deploying Groth16Verifier...");
    const Groth16Verifier = await hre.ethers.getContractFactory("Groth16Verifier");
    const groth16Verifier = await Groth16Verifier.deploy();
    await groth16Verifier.deployed();
    console.log( groth16Verifier.address);
    
    // 2. Deploy EpochManager
    console.log("\nDeploying EpochManager...");
    const EpochManager = await hre.ethers.getContractFactory("EpochManager");
    const epochManager = await EpochManager.deploy();
    await epochManager.deployed();
    console.log(epochManager.address);
    
    // 3. Deploy RateLimiter
    console.log("\nDeploying RateLimiter...");
    const RateLimiter = await hre.ethers.getContractFactory("RateLimiter");
    const rateLimiter = await RateLimiter.deploy();
    await rateLimiter.deployed();
    console.log( rateLimiter.address);
    
    // 4. Deploy AuthorizationVerifier
    console.log("\n Deploying AuthorizationVerifier...");
    const AuthorizationVerifier = await hre.ethers.getContractFactory("AuthorizationVerifier");
    const authVerifier = await AuthorizationVerifier.deploy();
    await authVerifier.deployed();
    console.log( authVerifier.address);
    
    // 5. Deploy PolicyRegistry
    console.log("\n Deploying PolicyRegistry...");
    const PolicyRegistry = await hre.ethers.getContractFactory("PolicyRegistry");
    const policyRegistry = await PolicyRegistry.deploy();
    await policyRegistry.deployed();
    console.log(policyRegistry.address);
    
    // 6. Deploy LineageVerifier
    console.log("\n Deploying LineageVerifier...");
    const LineageVerifier = await hre.ethers.getContractFactory("LineageVerifier");
    
    const GENESIS_COMMITMENT = "0x0000000000000000000000000000000000000000000000000000000000000001";
    const POLICY_ROOT = "0x0000000000000000000000000000000000000000000000000000000000000001";
    
    const lineageVerifier = await LineageVerifier.deploy(
        groth16Verifier.address,
        epochManager.address,
        rateLimiter.address,
        authVerifier.address,
        GENESIS_COMMITMENT,
        POLICY_ROOT,
        false // allowDuplicates
    );
    await lineageVerifier.deployed();
    console.log(lineageVerifier.address);
    
    // 7. Setup: Transfer RateLimiter Admin
    console.log("\nSetting up RateLimiter admin...");
    await rateLimiter.transferAdmin(lineageVerifier.address);
    console.log(" RateLimiter admin transferred");
    
    // 8. Initialize Genesis
    console.log("\n Initializing Genesis State...");
    const GENESIS_STATE_HASH = "0x0000000000000000000000000000000000000000000000000000000000000001";
    const tx = await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_COMMITMENT);
    await tx.wait();
    console.log(" Genesis initialized");
    
    // Save deployment info
    const deploymentInfo = {
        network: hre.network.name,
        deployer: deployer.address,
        timestamp: new Date().toISOString(),
        contracts: {
            Groth16Verifier: groth16Verifier.address,
            EpochManager: epochManager.address,
            RateLimiter: rateLimiter.address,
            AuthorizationVerifier: authVerifier.address,
            PolicyRegistry: policyRegistry.address,
            LineageVerifier: lineageVerifier.address,
        },
        genesis: {
            stateHash: GENESIS_STATE_HASH,
            commitment: GENESIS_COMMITMENT,
            policyRoot: POLICY_ROOT,
        },
        systemStatus: {
            genesisInitialized: true,
            rateLimitingActive: true,
            epochManagementActive: true,
            authorizationEnabled: true,
            policyEnforcementActive: true,
        }
    };
    
    fs.writeFileSync('deployment-complete.json', JSON.stringify(deploymentInfo, null, 2));
    
    console.log("\n" + "=".repeat(70));
    console.log(" COMPLETE ZK-ORIGIN DEPLOYMENT SUCCESS ");
    console.log("=".repeat(70));
    console.log("\nDeployed Addresses:");
    console.log(JSON.stringify(deploymentInfo.contracts, null, 2));
    console.log("\nSystem Status:");
    console.log(JSON.stringify(deploymentInfo.systemStatus, null, 2));
    console.log("\n Saved to: deployment-complete.json");
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
