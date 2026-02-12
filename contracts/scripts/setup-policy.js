const { ethers, network } = require("hardhat");
const fs = require("fs");
const path = require("path");
const { buildPoseidon } = require("circomlibjs");

async function main() {
    console.log("ZK-ORIGIN Policy Setup");
    
    // Load deployment info
    const deploymentsPath = path.join(__dirname, "../deployments", network.name, "addresses.json");
    if (!fs.existsSync(deploymentsPath)) {
        console.error("❌ Deployment not found. Run deploy.js first.");
        process.exit(1);
    }
    
    const deployment = JSON.parse(fs.readFileSync(deploymentsPath));
    console.log("Network:", network.name);
    console.log("PolicyRegistry:", deployment.contracts.PolicyRegistry);
    
    // Initialize Poseidon
    const poseidon = await buildPoseidon();
    const F = poseidon.F;
    
    // Define origin classes
    const Origin = {
        Genesis: 0,
        User: 1,
        Admin: 2,
        Bridge: 3
    };
    
    // Define allowed transitions
    const allowedTransitions = [
        [Origin.Genesis, Origin.User],
        [Origin.Genesis, Origin.Admin],
        [Origin.User, Origin.User],
        [Origin.Admin, Origin.User],
        [Origin.Admin, Origin.Admin],
        [Origin.Admin, Origin.Bridge],
        [Origin.Bridge, Origin.User]
    ];
    
    console.log("\nAllowed transitions:");
    for (const [from, to] of allowedTransitions) {
        const fromName = Object.keys(Origin).find(k => Origin[k] === from);
        const toName = Object.keys(Origin).find(k => Origin[k] === to);
        console.log(`   ${fromName} -> ${toName}`);
    }
    
    // Build Merkle tree
    function hashPair(left, right) {
        const result = poseidon([left, right]);
        return F.toObject(result);
    }
    
    function computePolicyLeaf(from, to) {
        const result = poseidon([BigInt(from), BigInt(to)]);
        return F.toObject(result);
    }
    
    // Compute leaves
    let leaves = allowedTransitions.map(([from, to]) => computePolicyLeaf(from, to));
    
    // Pad to power of 2
    const targetLength = Math.pow(2, Math.ceil(Math.log2(leaves.length)));
    while (leaves.length < targetLength) {
        leaves.push(BigInt(0));
    }
    
    console.log(`\nTree size: ${leaves.length} leaves (padded from ${allowedTransitions.length})`);
    
    // Build tree
    while (leaves.length > 1) {
        const nextLevel = [];
        for (let i = 0; i < leaves.length; i += 2) {
            nextLevel.push(hashPair(leaves[i], leaves[i + 1]));
        }
        leaves = nextLevel;
    }
    
    const merkleRoot = leaves[0];
    const merkleRootHex = "0x" + merkleRoot.toString(16).padStart(64, "0");
    
    console.log("Merkle root:", merkleRootHex);
    
    // Get contract instance
    const policyRegistry = await ethers.getContractAt(
        "PolicyRegistry",
        deployment.contracts.PolicyRegistry
    );
    
    // Create policy
    console.log("\nCreating policy on-chain...");
    const tx = await policyRegistry.createPolicy(
        merkleRootHex,
        "ZK-ORIGIN Default Policy v1.0"
    );
    const receipt = await tx.wait();
    
    // Get policy ID from event
    const event = receipt.logs.find(log => {
        try {
            return policyRegistry.interface.parseLog(log).name === "PolicyCreated";
        } catch {
            return false;
        }
    });
    
    const policyId = policyRegistry.interface.parseLog(event).args.policyId;
    console.log("Policy created with ID:", policyId.toString());
    
    // Activate policy
    console.log("Activating policy...");
    const activateTx = await policyRegistry.activatePolicy(policyId);
    await activateTx.wait();
    
    console.log("\n✅ Policy setup complete!");
    console.log("Current policy root:", await policyRegistry.getCurrentPolicyRoot());
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });