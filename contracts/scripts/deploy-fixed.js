
const hre = require("hardhat");

async function main() {
    const [deployer] = await ethers.getSigners();
    console.log("Deploying with account:", deployer.address);
    
    // ============ Deploy RateLimiter ============
    console.log("\n1. Deploying RateLimiter...");
    const genesisTime = Math.floor(Date.now() / 1000);
    
    const RateLimiter = await ethers.getContractFactory("RateLimiter");
    const rateLimiter = await RateLimiter.deploy(genesisTime);
    await rateLimiter.deployed();
    console.log("RateLimiter deployed to:", rateLimiter.address);
    
    // ============ Deploy PolicyRegistry ============
    console.log("\n2. Deploying PolicyRegistry...");
    
    const PolicyRegistry = await ethers.getContractFactory("PolicyRegistry");
    const policyRegistry = await PolicyRegistry.deploy();
    await policyRegistry.deployed();
    console.log("PolicyRegistry deployed to:", policyRegistry.address);
    
    // ============ Configure RateLimiter ============
    console.log("\n3. Setting LineageVerifier (will be set later)...");
    // NOTE: Set after LineageVerifier is deployed
    
    // ============ Create Initial Policy ============
    console.log("\n4. Creating initial policy...");
    
    // Define transitions (7x7 grid, all allowed for demo)
    const transitions = [];
    for (let from = 0; from < 7; from++) {
        for (let to = 0; to < 7; to++) {
            transitions.push([from, to]);
        }
    }
    
    // Mock merkle root (in production, compute actual tree)
    const mockPolicyRoot = ethers.utils.keccak256(
        ethers.utils.defaultAbiCoder.encode(
            ["uint8[][]"],
            [transitions]
        )
    );
    
    const createTx = await policyRegistry.createPolicy(
        mockPolicyRoot,
        "Initial Policy v1",
        transitions
    );
    await createTx.wait();
    console.log("Policy created");
    
    // Propose activation
    console.log("\n5. Proposing policy activation (2 day timelock)...");
    const proposeTx = await policyRegistry.proposePolicyActivation(0);
    await proposeTx.wait();
    console.log("Policy activation proposed");
    
    
    // ============ Output Deployment Info ============
    console.log("\n" + "=".repeat(60));
    console.log("DEPLOYMENT COMPLETE");
    console.log("=".repeat(60));
    console.log({
        RateLimiter: rateLimiter.address,
        PolicyRegistry: policyRegistry.address,
        GenesisTime: genesisTime,
        Network: hre.network.name
    });
    
    // Save to file
    const deployment = {
        timestamp: new Date().toISOString(),
        network: hre.network.name,
        deployer: deployer.address,
        contracts: {
            RateLimiter: rateLimiter.address,
            PolicyRegistry: policyRegistry.address
        },
        config: {
            genesisTime: genesisTime,
            policyActivationTimelock: "2 days",
            epochDuration: "24 hours"
        }
    };
    
    const fs = require("fs");
    fs.writeFileSync(
        `deployments/${hre.network.name}-deployment.json`,
        JSON.stringify(deployment, null, 2)
    );
    
    console.log("\n Deployment info saved to deployments/");
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });