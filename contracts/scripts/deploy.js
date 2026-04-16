const hre = require("hardhat");
const fs = require("fs");
const crypto = require("crypto");

// Helper function to generate hashes
const hash = (str) => {
    const hashObj = crypto.createHash('sha256');
    hashObj.update(str);
    return '0x' + hashObj.digest('hex');
};

async function main() {
    console.log(" Deploying ZK-ORIGIN contracts to", hre.network.name);
   
    
    const [deployer] = await ethers.getSigners();
    console.log(" Deploying from account:", deployer.address);
    
    const balance = await deployer.provider.getBalance(deployer.address);
    console.log(" Account balance:", ethers.utils.formatEther(balance), "ETH\n");
    
    // Generate genesis values
    const GENESIS_STATE_HASH = hash("genesis");
    const GENESIS_LINEAGE_COMMITMENT = hash("genesis_lineage");
    const POLICY_ROOT = hash("policy_root");
    
    console.log(" Genesis Configuration:");
    console.log("   Genesis State Hash:", GENESIS_STATE_HASH);
    console.log("   Genesis Lineage:", GENESIS_LINEAGE_COMMITMENT);
    console.log("   Policy Root:", POLICY_ROOT);
    console.log("");
    
    // Track deployment info
    const deployments = {
        network: hre.network.name,
        deployer: deployer.address,
        timestamp: new Date().toISOString(),
        contracts: {}
    };
    
    // 1. Deploy MockGroth16Verifier
    console.log(" Deploying MockGroth16Verifier...");
    const MockGroth16Verifier = await ethers.getContractFactory("MockGroth16Verifier");
    const groth16Verifier = await MockGroth16Verifier.deploy();
    await groth16Verifier.deployed();
    console.log("    MockGroth16Verifier:", groth16Verifier.address);
    deployments.contracts.groth16Verifier = groth16Verifier.address;
    
    // 2. Deploy EpochManager
    console.log("\n Deploying EpochManager...");
    const EpochManager = await ethers.getContractFactory("EpochManager");
    const epochManager = await EpochManager.deploy();
    await epochManager.deployed();
    console.log("    EpochManager:", epochManager.address);
    deployments.contracts.epochManager = epochManager.address;
    
    const currentEpoch = await epochManager.getCurrentEpoch();
    console.log("    Current Epoch:", currentEpoch.toString());
    
    // 3. Deploy RateLimiter
    console.log("\n Deploying RateLimiter...");
    const RateLimiter = await ethers.getContractFactory("RateLimiter");
    const rateLimiter = await RateLimiter.deploy();
    await rateLimiter.deployed();
    console.log("    RateLimiter:", rateLimiter.address);
    deployments.contracts.rateLimiter = rateLimiter.address;
    
    // Show rate limits
    console.log("    Rate Limits:");
    const originClasses = ["Genesis", "User", "Admin", "Bridge", "Governance", "System", "Emergency"];
    for (let i = 0; i < 7; i++) {
        const limit = await rateLimiter.getLimit(i);
        const limitStr = limit.toString().length > 20 ? "Unlimited" : limit.toString();
        console.log(`      ${originClasses[i]}: ${limitStr}`);
    }
    
    // 4. Deploy AuthorizationVerifier
    console.log("\n  Deploying AuthorizationVerifier...");
    const AuthorizationVerifier = await ethers.getContractFactory("AuthorizationVerifier");
    const authVerifier = await AuthorizationVerifier.deploy();
    await authVerifier.deployed();
    console.log("    AuthorizationVerifier:", authVerifier.address);
    deployments.contracts.authVerifier = authVerifier.address;
    
    // 5. Deploy LineageVerifier
    console.log("\n  Deploying LineageVerifier...");
    const LineageVerifier = await ethers.getContractFactory("LineageVerifier");
    const lineageVerifier = await LineageVerifier.deploy(
        groth16Verifier.address,
        epochManager.address,
        rateLimiter.address,
        authVerifier.address,
        GENESIS_LINEAGE_COMMITMENT,
        POLICY_ROOT
    );
    await lineageVerifier.deployed();
    console.log("    LineageVerifier:", lineageVerifier.address);
    deployments.contracts.lineageVerifier = lineageVerifier.address;
    
    // Transfer RateLimiter admin to LineageVerifier
    console.log("    Transferring RateLimiter admin to LineageVerifier...");
    const transferTx = await rateLimiter.transferAdmin(lineageVerifier.address);
    await transferTx.wait();
    console.log("    Admin transferred");
    
    // 6. Deploy StateRegistry
    console.log("\n Deploying StateRegistry...");
    const StateRegistry = await ethers.getContractFactory("StateRegistry");
    const stateRegistry = await StateRegistry.deploy();
    await stateRegistry.deployed();
    console.log("   StateRegistry:", stateRegistry.address);
    deployments.contracts.stateRegistry = stateRegistry.address;
    
    // 7. Deploy PolicyRegistry
    console.log("\n Deploying PolicyRegistry...");
    const PolicyRegistry = await ethers.getContractFactory("PolicyRegistry");
    const policyRegistry = await PolicyRegistry.deploy();
    await policyRegistry.deployed();
    console.log("    PolicyRegistry:", policyRegistry.address);
    deployments.contracts.policyRegistry = policyRegistry.address;
    
    // 8. Deploy BatchVerifier
    console.log("\n Deploying BatchVerifier...");
    const BatchVerifier = await ethers.getContractFactory("BatchVerifier");
    const batchVerifier = await BatchVerifier.deploy(
        lineageVerifier.address,
        authVerifier.address
    );
    await batchVerifier.deployed();
    console.log("    BatchVerifier:", batchVerifier.address);
    deployments.contracts.batchVerifier = batchVerifier.address;
    
    const maxBatch = await batchVerifier.getMaxProofsPerBatch();
    console.log("    Max Batch Size:", maxBatch.toString());
    
    // Initialize Genesis
    console.log("\n9 Initializing Genesis State...");
    const setGenesisTx = await lineageVerifier.setGenesis(
        GENESIS_STATE_HASH,
        GENESIS_LINEAGE_COMMITMENT
    );
    await setGenesisTx.wait();
    console.log("    Genesis initialized");
    
    const isInitialized = await lineageVerifier.genesisInitialized();
    console.log("    Genesis Initialized:", isInitialized);
    
    // Get final stats
    console.log("\n Final Statistics:");
    const stats = await lineageVerifier.getStats();
    console.log("   Transitions:", stats.transitions.toString());
    console.log("   Max Depth:", stats.maxDepth.toString());
    console.log("   Initialized:", stats.initialized);
    console.log("   Paused:", stats.isPaused);
    console.log("   Current Epoch:", stats.currentEpoch.toString());
    
    // Save deployment info
    const deploymentFile = `deployment-${hre.network.name}.json`;
    fs.writeFileSync(
        deploymentFile,
        JSON.stringify(deployments, null, 2)
    );
    console.log("\n Deployment info saved to:", deploymentFile);
    
   
    console.log(" Contract Addresses:");
    
    Object.entries(deployments.contracts).forEach(([name, address]) => {
        console.log(`${name.padEnd(25)} ${address}`);
    });
  
    
    console.log(" Network:", hre.network.name);
    console.log(" Deployer:", deployer.address);
    console.log(" Timestamp:", deployments.timestamp);
    console.log("");
    
    // Verification commands (for testnet/mainnet)
    if (hre.network.name !== "localhost" && hre.network.name !== "hardhat") {
        console.log(" Verify contracts with:");
        
        Object.entries(deployments.contracts).forEach(([name, address]) => {
            console.log(`npx hardhat verify --network ${hre.network.name} ${address}`);
        });
        console.log("");
    }
    
   
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(" Deployment failed:");
        console.error(error);
        process.exit(1);
    });