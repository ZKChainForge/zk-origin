const hre = require("hardhat");
const fs = require("fs");

async function main() {
    console.log("Deploying ZK-ORIGIN contracts...\n");

    // Deploy Groth16Verifier
    console.log("1. Deploying Groth16Verifier...");
    const Groth16Verifier = await hre.ethers.getContractFactory("Groth16Verifier");
    const groth16Verifier = await Groth16Verifier.deploy();
    await groth16Verifier.deployed();
    const groth16Address = groth16Verifier.address;
    console.log("   Groth16Verifier deployed to:", groth16Address);

    // Deploy EpochManager
    console.log("\n2. Deploying EpochManager...");
    const EpochManager = await hre.ethers.getContractFactory("EpochManager");
    const epochManager = await EpochManager.deploy();
    await epochManager.deployed();
    const epochAddress = epochManager.address;
    console.log("   EpochManager deployed to:", epochAddress);

    // Deploy RateLimiter
    console.log("\n3. Deploying RateLimiter...");
    const RateLimiter = await hre.ethers.getContractFactory("RateLimiter");
    const rateLimiter = await RateLimiter.deploy();
    await rateLimiter.deployed();
    const rateLimiterAddress = rateLimiter.address;
    console.log("   RateLimiter deployed to:", rateLimiterAddress);

    // Deploy AuthorizationVerifier
    console.log("\n4. Deploying AuthorizationVerifier...");
    const AuthVerifier = await hre.ethers.getContractFactory("AuthorizationVerifier");
    const authVerifier = await AuthVerifier.deploy();
    await authVerifier.deployed();
    const authAddress = authVerifier.address;
    console.log("   AuthorizationVerifier deployed to:", authAddress);

    // Load policy root and convert to bytes32
    let policyRoot;
    try {
        const policyData = JSON.parse(fs.readFileSync("./policy_root.json", "utf8"));
        // Convert decimal string to hex bytes32
        const policyRootBigInt = BigInt(policyData.root);
        policyRoot = "0x" + policyRootBigInt.toString(16).padStart(64, "0");
    } catch (e) {
        console.log("\n  policy_root.json not found, using default");
        policyRoot = "0x" + "0".repeat(64);
    }
    console.log("\n5. Using policy root:", policyRoot);

    // Genesis lineage commitment
    const genesisLineage = "0x" + "0".repeat(64);

    // Deploy LineageVerifier
    console.log("\n6. Deploying LineageVerifier...");
    const LineageVerifier = await hre.ethers.getContractFactory("LineageVerifier");
    const lineageVerifier = await LineageVerifier.deploy(
        groth16Address,
        epochAddress,
        rateLimiterAddress,
        authAddress,
        genesisLineage,
        policyRoot,
        false 
    );
    await lineageVerifier.deployed();
    const lineageAddress = lineageVerifier.address;
    console.log("   LineageVerifier deployed to:", lineageAddress);

    // Transfer RateLimiter admin to LineageVerifier
    console.log("\n7. Transferring RateLimiter admin...");
    const tx = await rateLimiter.transferAdmin(lineageAddress);
    await tx.wait();
    console.log("   RateLimiter admin transferred");

    // Save deployment addresses
    const deployment = {
        network: hre.network.name,
        timestamp: new Date().toISOString(),
        contracts: {
            Groth16Verifier: groth16Address,
            EpochManager: epochAddress,
            RateLimiter: rateLimiterAddress,
            AuthorizationVerifier: authAddress,
            LineageVerifier: lineageAddress
        },
        config: {
            policyRoot: policyRoot,
            genesisLineage: genesisLineage
        }
    };

    fs.writeFileSync(
        "./deployment-complete.json",
        JSON.stringify(deployment, null, 2)
    );

    console.log(" Addresses saved to deployment-complete.json\n");
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });