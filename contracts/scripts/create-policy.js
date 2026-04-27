/**
 * @title Create Policy Script
 * @notice Set up origin class transition policies
 */

const hre = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
    console.log(" Setting Up Origin Policies...\n");
    
    const [deployer] = await ethers.getSigners();
    console.log("Deployer:", deployer.address);
    
    // Read deployment addresses
    const deploymentPath = path.join(__dirname, "../deployment-hardhat.json");
    const deployment = JSON.parse(fs.readFileSync(deploymentPath, "utf8"));
    const lineageVerifierAddress = deployment.contracts.LineageVerifier;
    
    // Origin classes
    const ORIGIN_GENESIS = 0;
    const ORIGIN_USER = 1;
    const ORIGIN_ADMIN = 2;
    const ORIGIN_BRIDGE = 3;
    const ORIGIN_GOVERNANCE = 4;
    const ORIGIN_SYSTEM = 5;
    const ORIGIN_EMERGENCY = 6;
    
    const originNames = {
        0: "GENESIS",
        1: "USER",
        2: "ADMIN",
        3: "BRIDGE",
        4: "GOVERNANCE",
        5: "SYSTEM",
        6: "EMERGENCY",
    };
    
    // Get LineageVerifier contract
    const LineageVerifier = await ethers.getContractFactory("LineageVerifier");
    const lineageVerifier = LineageVerifier.attach(lineageVerifierAddress);
    
    // Define policy transitions (from → to)
    const policies = [
        // Genesis transitions
        { from: ORIGIN_GENESIS, to: ORIGIN_USER, allowed: true },
        { from: ORIGIN_GENESIS, to: ORIGIN_ADMIN, allowed: true },
        { from: ORIGIN_GENESIS, to: ORIGIN_SYSTEM, allowed: true },
        
        // User transitions
        { from: ORIGIN_USER, to: ORIGIN_USER, allowed: true },
        
        // Admin transitions
        { from: ORIGIN_ADMIN, to: ORIGIN_USER, allowed: true },
        { from: ORIGIN_ADMIN, to: ORIGIN_ADMIN, allowed: true },
        { from: ORIGIN_ADMIN, to: ORIGIN_BRIDGE, allowed: true },
        { from: ORIGIN_ADMIN, to: ORIGIN_SYSTEM, allowed: true },
        
        // Bridge transitions
        { from: ORIGIN_BRIDGE, to: ORIGIN_USER, allowed: true },
        
        // System transitions
        { from: ORIGIN_SYSTEM, to: ORIGIN_USER, allowed: true },
        { from: ORIGIN_SYSTEM, to: ORIGIN_SYSTEM, allowed: true },
        
        // Emergency transitions
        { from: ORIGIN_EMERGENCY, to: ORIGIN_USER, allowed: true },
        { from: ORIGIN_EMERGENCY, to: ORIGIN_ADMIN, allowed: true },
        { from: ORIGIN_EMERGENCY, to: ORIGIN_SYSTEM, allowed: true },
    ];
    
    try {
        console.log("\nSetting policies...");
        console.log("-".repeat(50));
        
        for (const policy of policies) {
            const fromName = originNames[policy.from];
            const toName = originNames[policy.to];
            
            console.log(`${fromName} → ${toName}: ${policy.allowed ? " ALLOWED" : "DENIED"}`);
            
            const tx = await lineageVerifier.setPolicyTransition(
                policy.from,
                policy.to,
                policy.allowed
            );
            await tx.wait();
        }
        
        console.log("-".repeat(50));
        console.log("\n All policies set successfully!");
        
        // Verify a few policies
        console.log("\n Verifying policies...");
        const testCases = [
            { from: ORIGIN_GENESIS, to: ORIGIN_USER },
            { from: ORIGIN_ADMIN, to: ORIGIN_BRIDGE },
            { from: ORIGIN_USER, to: ORIGIN_ADMIN },
        ];
        
        for (const test of testCases) {
            const allowed = await lineageVerifier.isTransitionAllowed(test.from, test.to);
            const fromName = originNames[test.from];
            const toName = originNames[test.to];
            console.log(`${fromName} → ${toName}: ${allowed ? "YES" : "NO"}`);
        }
        
    } catch (error) {
        console.error("\n Error setting policies:", error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch(error => {
        console.error("Fatal error:", error);
        process.exit(1);
    });