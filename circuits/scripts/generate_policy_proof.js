const { buildPoseidon } = require("circomlibjs");
const fs = require("fs");

async function generatePolicyProof() {
    const poseidon = await buildPoseidon();
    
    // Define allowed transitions (matches contract policy)
    const allowedTransitions = [
        [0, 1], // Genesis → User
        [0, 2], // Genesis → Admin
        [0, 5], // Genesis → System
        [1, 1], // User → User
        [2, 1], // Admin → User
        [2, 2], // Admin → Admin
        [2, 3], // Admin → Bridge
        [2, 5], // Admin → System
        [3, 1], // Bridge → User
        [4, 0], // Governance → Genesis
        [4, 1], // Governance → User
        [4, 2], // Governance → Admin
        [4, 3], // Governance → Bridge
        [4, 4], // Governance → Governance
        [4, 5], // Governance → System
        [4, 6], // Governance → Emergency
        [5, 1], // System → User
        [5, 5], // System → System
        [6, 1], // Emergency → User
        [6, 2], // Emergency → Admin
        [6, 5], // Emergency → System
    ];
    
    // Compute leaves (hash each transition pair)
    const leaves = allowedTransitions.map(([from, to]) => {
        const hash = poseidon.F.toString(poseidon([from, to]));
        return hash;
    });
    
    console.log(`Total transitions: ${leaves.length}`);
    console.log(`Merkle tree depth needed: ${Math.ceil(Math.log2(leaves.length))}`);
    
    // Pad leaves to next power of 2 (for depth 6 = 64 leaves)
    const targetSize = 64;
    while (leaves.length < targetSize) {
        leaves.push(poseidon.F.toString(poseidon([0, 0])));
    }
    
    // Build Merkle tree
    let currentLevel = leaves;
    const tree = [currentLevel];
    
    while (currentLevel.length > 1) {
        const nextLevel = [];
        for (let i = 0; i < currentLevel.length; i += 2) {
            const left = currentLevel[i];
            const right = currentLevel[i + 1];
            const parent = poseidon.F.toString(poseidon([left, right]));
            nextLevel.push(parent);
        }
        tree.push(nextLevel);
        currentLevel = nextLevel;
    }
    
    const root = tree[tree.length - 1][0];
    console.log(`\nMerkle Root: ${root}`);
    
    // Generate proof for (0, 1) - Genesis → User
    const targetFrom = 0;
    const targetTo = 1;
    const targetLeaf = poseidon.F.toString(poseidon([targetFrom, targetTo]));
    const leafIndex = allowedTransitions.findIndex(([f, t]) => f === targetFrom && t === targetTo);
    
    console.log(`\nGenerating proof for transition (${targetFrom}, ${targetTo})`);
    console.log(`Leaf index: ${leafIndex}`);
    console.log(`Leaf hash: ${targetLeaf}`);
    
    // Generate Merkle proof
    const proof = [];
    const indices = [];
    let index = leafIndex;
    
    for (let level = 0; level < tree.length - 1; level++) {
        const isLeft = index % 2 === 0;
        const siblingIndex = isLeft ? index + 1 : index - 1;
        const sibling = tree[level][siblingIndex];
        
        proof.push(sibling);
        indices.push(isLeft ? 0 : 1);
        
        index = Math.floor(index / 2);
    }
    
    console.log(`\nProof (${proof.length} elements):`);
    proof.forEach((p, i) => console.log(`  [${i}] ${p} (index: ${indices[i]})`));
    
    // Save to file
    const output = {
        root: root,
        transitions: allowedTransitions,
        proof: {
            from: targetFrom,
            to: targetTo,
            leaf: targetLeaf,
            proof: proof,
            indices: indices
        }
    };
    
    fs.writeFileSync("policy_tree.json", JSON.stringify(output, null, 2));
    fs.writeFileSync("../contracts/policy_root.json", JSON.stringify({ root }, null, 2));
    
    
    
    return output;
}

generatePolicyProof().catch(console.error);