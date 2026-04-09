const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
    console.log("=============================================================");
    console.log("|          ZK-ORIGIN Complete Deployment Script              |");
    console.log("=============================================================\n");

    const [deployer] = await hre.ethers.getSigners();
    
    console.log(" Deployment Configuration:");
    console.log("   Deployer address:", deployer.address);
    console.log("   Network:", hre.network.name);
    console.log("   Chain ID:", (await hre.ethers.provider.getNetwork()).chainId);
    
    const balance = await deployer.getBalance();
    console.log("   Deployer balance:", hre.ethers.utils.formatEther(balance), "ETH\n");

    // Check if we have enough balance
    const minBalance = hre.ethers.utils.parseEther("0.1");
    if (balance.lt(minBalance)) {
        throw new Error(`Insufficient balance. Need at least 0.1 ETH, have ${hre.ethers.utils.formatEther(balance)} ETH`);
    }

    // Load policy root from file
    let policyData;
    let policyRoot;
    let transitions;

    try {
        const policyPath = path.join(__dirname, "../policy_root.json");
        if (!fs.existsSync(policyPath)) {
            // Try parent directory
            const altPath = path.join(__dirname, "../../policy_root.json");
            if (fs.existsSync(altPath)) {
                policyData = JSON.parse(fs.readFileSync(altPath, "utf8"));
            } else {
                throw new Error("policy_root.json not found. Run: cargo run --bin generate_policy");
            }
        } else {
            policyData = JSON.parse(fs.readFileSync(policyPath, "utf8"));
        }

        policyRoot = policyData.root;
        transitions = policyData.transitions;

        
        console.log("   Policy root:", policyRoot);
        console.log("   Transitions:", transitions.length);
        console.log("   Tree depth:", policyData.tree_depth, "\n");
    } catch (error) {
        console.error(" Error loading policy:", error.message);
        console.log("\n  Using default zero policy root for testing");
        policyRoot = "0x" + "0".repeat(64);
        transitions = [];
    }

    const deployments = {};
    const deploymentFile = path.join(__dirname, "../deployment-complete.json");

    // ============================================================
    // STEP 1: Deploy Groth16Verifier
    // ============================================================


    const Groth16Verifier = await hre.ethers.getContractFactory("Groth16Verifier");
    const groth16Verifier = await Groth16Verifier.deploy();
    await groth16Verifier.deployed();
    
    deployments.groth16Verifier = groth16Verifier.address;
    
    console.log(" Groth16Verifier deployed to:", groth16Verifier.address);
   

    // ============================================================
    // STEP 2: Deploy EpochManager
    // ============================================================


    const EpochManager = await hre.ethers.getContractFactory("EpochManager");
    const epochManager = await EpochManager.deploy();
    await epochManager.deployed();
    
    deployments.epochManager = epochManager.address;
    
    const currentEpoch = await epochManager.getCurrentEpoch();
    const epochDuration = await epochManager.EPOCH_DURATION();
    
    console.log(" EpochManager deployed to:", epochManager.address);
    console.log("   Current epoch:", currentEpoch.toString());
    console.log("   Epoch duration:", epochDuration.toString(), "seconds (24 hours)\n");

    // ============================================================
    // STEP 3: Deploy RateLimiter
    // ============================================================


    const RateLimiter = await hre.ethers.getContractFactory("RateLimiter");
    const rateLimiter = await RateLimiter.deploy();
    await rateLimiter.deployed();
    
    deployments.rateLimiter = rateLimiter.address;
    
    console.log(" RateLimiter deployed to:", rateLimiter.address);


    // ============================================================
    // STEP 4: Deploy AuthorizationVerifier
    // ============================================================


    const AuthorizationVerifier = await hre.ethers.getContractFactory("AuthorizationVerifier");
    const authVerifier = await AuthorizationVerifier.deploy();
    await authVerifier.deployed();
    
    deployments.authorizationVerifier = authVerifier.address;
    
    console.log(" AuthorizationVerifier deployed to:", authVerifier.address);


    // ============================================================
    // STEP 5: Deploy LineageVerifier (Main Contract)
    // ============================================================


    const genesisLineageCommitment = "0x" + "0".repeat(64);
    const allowDuplicateStates = false;

    console.log("     - Groth16Verifier:", groth16Verifier.address);
    console.log("     - EpochManager:", epochManager.address);
    console.log("     - RateLimiter:", rateLimiter.address);
    console.log("     - AuthVerifier:", authVerifier.address);
    console.log("     - Genesis commitment:", genesisLineageCommitment);
    console.log("     - Policy root:", policyRoot);
    console.log("     - Allow duplicates:", allowDuplicateStates, "\n");

    const LineageVerifier = await hre.ethers.getContractFactory("LineageVerifier");
    const lineageVerifier = await LineageVerifier.deploy(
        groth16Verifier.address,
        epochManager.address,
        rateLimiter.address,
        authVerifier.address,
        genesisLineageCommitment,
        policyRoot,
        allowDuplicateStates
    );
    await lineageVerifier.deployed();
    
    deployments.lineageVerifier = lineageVerifier.address;
    
    console.log(" LineageVerifier deployed to:", lineageVerifier.address);
    console.log("   Max depth:", (await lineageVerifier.MAX_DEPTH()).toString());
    console.log("   Version:", (await lineageVerifier.VERSION()).toString(), "\n");

    // ============================================================
    // STEP 6: Transfer RateLimiter Admin to LineageVerifier
    // ============================================================


    console.log("   Transferring RateLimiter admin to LineageVerifier...");
    const transferTx = await rateLimiter.transferAdmin(lineageVerifier.address);
    await transferTx.wait();
    
    console.log("   New admin:", lineageVerifier.address, "\n");

    // ============================================================
    // STEP 7: Deploy PolicyRegistry
    // ============================================================

    const PolicyRegistry = await hre.ethers.getContractFactory("PolicyRegistry");
    const policyRegistry = await PolicyRegistry.deploy();
    await policyRegistry.deployed();
    
    deployments.policyRegistry = policyRegistry.address;
    
    console.log(" PolicyRegistry deployed to:", policyRegistry.address, "\n");

    // ============================================================
    // STEP 8: Create and Activate Initial Policy
    // ============================================================
    if (transitions.length > 0) {


        console.log("   Creating policy with", transitions.length, "transitions...");
        const createPolicyTx = await policyRegistry.createPolicy(
            policyRoot,
            "ZK-ORIGIN Default Policy v1.0",
            transitions
        );
        const receipt = await createPolicyTx.wait();
        console.log(" Policy created (tx:", receipt.transactionHash, ")");
        console.log("   Gas used:", receipt.gasUsed.toString(), "\n");

        console.log("   Activating policy...");
        const activateTx = await policyRegistry.activatePolicy(0);
        await activateTx.wait();
        console.log(" Policy activated");
        
        const currentPolicyRoot = await policyRegistry.getCurrentPolicyRoot();
        console.log("   Active policy root:", currentPolicyRoot, "\n");
    } else {
        console.log("  No transitions loaded, skipping policy creation\n");
    }

    // ============================================================
    // STEP 9: Deploy BatchVerifier
    // ============================================================


    const BatchVerifier = await hre.ethers.getContractFactory("BatchVerifier");
    const batchVerifier = await BatchVerifier.deploy(lineageVerifier.address);
    await batchVerifier.deployed();
    
    deployments.batchVerifier = batchVerifier.address;
    
    console.log(" BatchVerifier deployed to:", batchVerifier.address);
    

    // ============================================================
    // STEP 10: Set Genesis State
    // ============================================================


    const genesisStateHash = hre.ethers.utils.keccak256(hre.ethers.utils.toUtf8Bytes("ZK-ORIGIN Genesis State v1.0"));
    const genesisCommitment = hre.ethers.utils.keccak256(hre.ethers.utils.toUtf8Bytes("ZK-ORIGIN Genesis Lineage v1.0"));

    console.log("   Genesis state hash:", genesisStateHash);
    console.log("   Genesis lineage commitment:", genesisCommitment, "\n");

    const setGenesisTx = await lineageVerifier.setGenesis(
        genesisStateHash,
        genesisCommitment
    );
    await setGenesisTx.wait();
    
    console.log(" Genesis state initialized");
    const genesisInitialized = await lineageVerifier.genesisInitialized();
    console.log("   Genesis initialized:", genesisInitialized, "\n");

    // ============================================================
    // Save Deployment Info
    // ============================================================
    deployments.network = hre.network.name;
    deployments.chainId = (await hre.ethers.provider.getNetwork()).chainId.toString();
    deployments.deployer = deployer.address;
    deployments.timestamp = new Date().toISOString();
    deployments.genesisStateHash = genesisStateHash;
    deployments.genesisLineageCommitment = genesisCommitment;
    deployments.policyRoot = policyRoot;
    deployments.transitionCount = transitions.length;

    fs.writeFileSync(
        deploymentFile,
        JSON.stringify(deployments, null, 2)
    );

    

    console.log(" Deployment:");
    console.log("   Network:", deployments.network);
    console.log("   Chain ID:", deployments.chainId);
    console.log("   Deployer:", deployments.deployer);
    console.log("   Timestamp:", deployments.timestamp, "\n");

    console.log(" Contract Addresses:");
    console.log("   Groth16Verifier:", deployments.groth16Verifier);
    console.log("   EpochManager:", deployments.epochManager);
    console.log("   RateLimiter:", deployments.rateLimiter);
    console.log("   AuthorizationVerifier:", deployments.authorizationVerifier);
    console.log("   LineageVerifier:", deployments.lineageVerifier);
    console.log("   PolicyRegistry:", deployments.policyRegistry);
    console.log("   BatchVerifier:", deployments.batchVerifier, "\n");

    console.log(" Deployment saved to:", deploymentFile, "\n");



    return deployments;
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("\n❌ Deployment failed:\n");
        console.error(error);
        process.exit(1);
    });