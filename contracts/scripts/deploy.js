/**
 * @title Deployment Script (PRODUCTION)
 * @notice Complete deployment of ZK-ORIGIN contracts
 * 
 * DEPLOYMENT FLOW:
 * 1. Deploy Groth16Verifier (auto-generated)
 * 2. Deploy EpochManager
 * 3. Deploy RateLimiter
 * 4. Deploy AuthorizationVerifier
 * 5. Deploy LineageVerifier
 * 6. Deploy PolicyRegistry
 * 7. Deploy StateRegistry
 * 8. Deploy BatchVerifier
 * 9. Deploy NovaLineageVerifier
 * 10. Verify all contracts
 * 11. Record deployment addresses
 */

const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
    console.log(" ZK-ORIGIN Deployment Starting...\n");
    
    const [deployer] = await ethers.getSigners();
    console.log("Deployer Address:", deployer.address);
    console.log("Network:", hre.network.name);
    console.log();
    
    // ============ STEP 1: Deploy Groth16Verifier ============
    console.log("1 Deploying Groth16Verifier...");
    const Groth16Verifier = await ethers.getContractFactory("Groth16Verifier");
    const groth16Verifier = await Groth16Verifier.deploy();
    await groth16Verifier.deployed();
    console.log(" Groth16Verifier:", groth16Verifier.address);
    
    // ============ STEP 2: Deploy EpochManager ============
    console.log("\n2  Deploying EpochManager...");
    const EpochManager = await ethers.getContractFactory("EpochManager");
    const epochManager = await EpochManager.deploy();
    await epochManager.deployed();
    console.log(" EpochManager:", epochManager.address);
    
    // ============ STEP 3: Deploy RateLimiter ============
    console.log("\n3  Deploying RateLimiter...");
    const RateLimiter = await ethers.getContractFactory("RateLimiter");
    const rateLimiter = await RateLimiter.deploy();
    await rateLimiter.deployed();
    console.log(" RateLimiter:", rateLimiter.address);
    
    // ============ STEP 4: Deploy AuthorizationVerifier ============
    console.log("\n4 Deploying AuthorizationVerifier...");
    const AuthorizationVerifier = await ethers.getContractFactory("AuthorizationVerifier");
    const authVerifier = await AuthorizationVerifier.deploy();
    await authVerifier.deployed();
    console.log(" AuthorizationVerifier:", authVerifier.address);
    
    // ============ STEP 5: Deploy LineageVerifier ============
    console.log("\n5 Deploying LineageVerifier...");
    
    // Read genesis commitment and policy root from config
    const genesisLineageCommitment = process.env.GENESIS_LINEAGE_COMMITMENT ||
        "0x0000000000000000000000000000000000000000000000000000000000000001";
    const policyRoot = process.env.POLICY_ROOT ||
        "0x0000000000000000000000000000000000000000000000000000000000000002";
    
    const LineageVerifier = await ethers.getContractFactory("LineageVerifier");
    const lineageVerifier = await LineageVerifier.deploy(
        groth16Verifier.address,
        epochManager.address,
        rateLimiter.address,
        authVerifier.address,
        genesisLineageCommitment,
        policyRoot
    );
    await lineageVerifier.deployed();
    console.log(" LineageVerifier:", lineageVerifier.address);
    
    // ============ STEP 6: Deploy PolicyRegistry ============
    console.log("\n6 Deploying PolicyRegistry...");
    const PolicyRegistry = await ethers.getContractFactory("PolicyRegistry");
    const policyRegistry = await PolicyRegistry.deploy();
    await policyRegistry.deployed();
    console.log(" PolicyRegistry:", policyRegistry.address);
    
    // ============ STEP 7: Deploy StateRegistry ============
    console.log("\n7  Deploying StateRegistry...");
    const StateRegistry = await ethers.getContractFactory("StateRegistry");
    const stateRegistry = await StateRegistry.deploy(lineageVerifier.address);
    await stateRegistry.deployed();
    console.log(" StateRegistry:", stateRegistry.address);
    
    // ============ STEP 8: Deploy BatchVerifier ============
    console.log("\n8 Deploying BatchVerifier...");
    const BatchVerifier = await ethers.getContractFactory("BatchVerifier");
    const batchVerifier = await BatchVerifier.deploy(
        lineageVerifier.address,
        authVerifier.address
    );
    await batchVerifier.deployed();
    console.log(" BatchVerifier:", batchVerifier.address);
    
    // ============ STEP 9: Deploy NovaLineageVerifier ============
    console.log("\n9️ Deploying NovaLineageVerifier...");
    const NovaLineageVerifier = await ethers.getContractFactory("NovaLineageVerifier");
    const novaVerifier = await NovaLineageVerifier.deploy(
        groth16Verifier.address,
        epochManager.address,
        genesisLineageCommitment,
        policyRoot
    );
    await novaVerifier.deployed();
    console.log(" NovaLineageVerifier:", novaVerifier.address);
    
    // ============ STEP 10: Record Deployment ============
    console.log("\n Recording deployment addresses...");
    
    const deployment = {
        network: hre.network.name,
        timestamp: new Date().toISOString(),
        deployer: deployer.address,
        contracts: {
            Groth16Verifier: groth16Verifier.address,
            EpochManager: epochManager.address,
            RateLimiter: rateLimiter.address,
            AuthorizationVerifier: authVerifier.address,
            LineageVerifier: lineageVerifier.address,
            PolicyRegistry: policyRegistry.address,
            StateRegistry: stateRegistry.address,
            BatchVerifier: batchVerifier.address,
            NovaLineageVerifier: novaVerifier.address,
        },
        config: {
            genesisLineageCommitment,
            policyRoot,
        },
    };
    
    const deploymentPath = path.join(
        __dirname,
        `../deployment-${hre.network.name}.json`
    );
    fs.writeFileSync(deploymentPath, JSON.stringify(deployment, null, 2));
    console.log(" Deployment saved to:", deploymentPath);
    
    // ============ STEP 11: Verify Contracts ============
    console.log("\n Verifying contracts...");
    
    if (hre.network.name !== "hardhat" && hre.network.name !== "localhost") {
        console.log("Waiting 30 seconds before verification...");
        await new Promise(resolve => setTimeout(resolve, 30000));
        
        const verifyContracts = [
            { address: groth16Verifier.address, args: [], name: "Groth16Verifier" },
            { address: epochManager.address, args: [], name: "EpochManager" },
            { address: rateLimiter.address, args: [], name: "RateLimiter" },
            { address: authVerifier.address, args: [], name: "AuthorizationVerifier" },
            {
                address: lineageVerifier.address,
                args: [
                    groth16Verifier.address,
                    epochManager.address,
                    rateLimiter.address,
                    authVerifier.address,
                    genesisLineageCommitment,
                    policyRoot,
                ],
                name: "LineageVerifier",
            },
            { address: policyRegistry.address, args: [], name: "PolicyRegistry" },
            { address: stateRegistry.address, args: [lineageVerifier.address], name: "StateRegistry" },
            {
                address: batchVerifier.address,
                args: [lineageVerifier.address, authVerifier.address],
                name: "BatchVerifier",
            },
            {
                address: novaVerifier.address,
                args: [
                    groth16Verifier.address,
                    epochManager.address,
                    genesisLineageCommitment,
                    policyRoot,
                ],
                name: "NovaLineageVerifier",
            },
        ];
        
        for (const contract of verifyContracts) {
            try {
                console.log(`\nVerifying ${contract.name}...`);
                await hre.run("verify:verify", {
                    address: contract.address,
                    constructorArguments: contract.args,
                });
                console.log(` ${contract.name} verified`);
            } catch (error) {
                console.log(`  ${contract.name} verification failed:`, error.message);
            }
        }
    }
    
    // ============ STEP 12: Print Summary ============
    console.log("\n" + "=".repeat(60));
    console.log(" DEPLOYMENT COMPLETE");
    console.log("=".repeat(60));
    console.log("\nContract Addresses:");
    console.log("-".repeat(60));
    Object.entries(deployment.contracts).forEach(([name, address]) => {
        console.log(`${name.padEnd(30)} ${address}`);
    });
    console.log("-".repeat(60));
    console.log("\nDeployment Info:");
    console.log(`Network: ${hre.network.name}`);
    console.log(`Deployer: ${deployer.address}`);
    console.log(`Timestamp: ${deployment.timestamp}`);
    console.log("\nNext Steps:");
    console.log("1. Set genesis state: npx hardhat run scripts/set-genesis.js --network <network>");
    console.log("2. Create policy: npx hardhat run scripts/create-policy.js --network <network>");
    console.log("3. Generate test proofs: npm run generate-proof");
    console.log("4. Verify proofs: npm run verify-proof");
    console.log("\n");
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(" Deployment failed:", error);
        process.exit(1);
    });