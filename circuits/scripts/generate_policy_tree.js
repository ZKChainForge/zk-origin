/**
 * Generate Merkle Tree for Policy
 * 
 * Creates a Merkle tree of allowed transitions for ZK verification
 */

const fs = require("fs");
const crypto = require("crypto");
const path = require("path");

// Poseidon hash implementation (compatible with Circom)
const poseidon = require("poseidon-js");

class PolicyMerkleTree {
    constructor() {
        this.leaves = [];
        this.tree = [];
    }
    
    /**
     * Add allowed transition
     */
    addTransition(fromClass, toClass) {
        // Hash: Poseidon(fromClass, toClass)
        const leaf = poseidon([fromClass, toClass]);
        this.leaves.push({
            fromClass,
            toClass,
            leaf: leaf.toString(),
        });
    }
    
    /**
     * Build Merkle tree from leaves
     */
    build() {
        if (this.leaves.length === 0) {
            throw new Error("No transitions added");
        }
        
        // Create leaf nodes
        let nodes = this.leaves.map(l => l.leaf);
        this.tree = [nodes];
        
        // Build tree bottom-up
        while (nodes.length > 1) {
            const newNodes = [];
            
            // Ensure even number of nodes
            if (nodes.length % 2 !== 0) {
                nodes.push(nodes[nodes.length - 1]);
            }
            
            // Hash pairs
            for (let i = 0; i < nodes.length; i += 2) {
                const left = nodes[i];
                const right = nodes[i + 1];
                const parent = poseidon([left, right]).toString();
                newNodes.push(parent);
            }
            
            this.tree.push(newNodes);
            nodes = newNodes;
        }
    }
    
    /**
     * Get root
     */
    getRoot() {
        if (this.tree.length === 0) {
            throw new Error("Tree not built");
        }
        return this.tree[this.tree.length - 1][0];
    }
    
    /**
     * Get Merkle proof for a transition
     */
    getProof(fromClass, toClass) {
        // Find leaf index
        let leafIndex = this.leaves.findIndex(
            l => l.fromClass === fromClass && l.toClass === toClass
        );
        
        if (leafIndex === -1) {
            throw new Error(`Transition ${fromClass}→${toClass} not found`);
        }
        
        const proof = [];
        const indices = [];
        
        // Walk up tree
        for (let level = 0; level < this.tree.length - 1; level++) {
            const nodes = this.tree[level];
            
            // Find position at this level
            const isRight = leafIndex % 2 === 1;
            const siblingIndex = isRight ? leafIndex - 1 : leafIndex + 1;
            
            if (siblingIndex < nodes.length) {
                proof.push(nodes[siblingIndex]);
                indices.push(isRight ? 1 : 0);
            }
            
            leafIndex = Math.floor(leafIndex / 2);
        }
        
        return { proof, indices };
    }
    
    /**
     * Verify a Merkle proof
     */
    verifyProof(fromClass, toClass, proof, indices) {
        const leaf = poseidon([fromClass, toClass]).toString();
        let hash = leaf;
        
        for (let i = 0; i < proof.length; i++) {
            const isRight = indices[i] === 1;
            
            if (isRight) {
                hash = poseidon([proof[i], hash]).toString();
            } else {
                hash = poseidon([hash, proof[i]]).toString();
            }
        }
        
        return hash === this.getRoot();
    }
}

/**
 * Generate default policy tree
 */
function generateDefaultPolicyTree() {
    const tree = new PolicyMerkleTree();
    
    // Define allowed transitions
    const allowedTransitions = [
        // Genesis (0) → User (1), Admin (2), System (5)
        [0, 1], [0, 2], [0, 5],
        
        // User (1) → User (1)
        [1, 1],
        
        // Admin (2) → User (1), Admin (2), Bridge (3), System (5)
        [2, 1], [2, 2], [2, 3], [2, 5],
        
        // Bridge (3) → User (1)
        [3, 1],
        
        // Governance (4) → All
        [4, 0], [4, 1], [4, 2], [4, 3], [4, 4], [4, 5], [4, 6],
        
        // System (5) → User (1), System (5)
        [5, 1], [5, 5],
        
        // Emergency (6) → User (1), Admin (2), System (5)
        [6, 1], [6, 2], [6, 5],
    ];
    
    for (const [from, to] of allowedTransitions) {
        tree.addTransition(from, to);
    }
    
    tree.build();
    return tree;
}

/**
 * Main execution
 */
async function main() {
    console.log("\n" + "═".repeat(60));
    console.log(" POLICY MERKLE TREE GENERATION");
    console.log("═".repeat(60) + "\n");
    
    // Generate tree
    console.log(" Generating policy tree...");
    const tree = generateDefaultPolicyTree();
    
    const root = tree.getRoot();
    console.log(` Tree generated`);
    console.log(`   Root: ${root.toString().slice(0, 20)}...`);
    console.log(`   Leaves: ${tree.leaves.length}`);
    
    // Generate proof examples
    console.log("\n Generating Merkle proofs for each transition...");
    
    const proofs = {};
    for (const leaf of tree.leaves) {
        const { fromClass, toClass } = leaf;
        const { proof, indices } = tree.getProof(fromClass, toClass);
        
        proofs[`${fromClass}→${toClass}`] = {
            from: fromClass,
            to: toClass,
            proof: proof.map(p => p.toString()),
            indices,
        };
    }
    
    console.log(` ${Object.keys(proofs).length} proofs generated`);
    
    // Save outputs
    console.log("\n Saving outputs...");
    
    const policyTree = {
        root: root.toString(),
        leaves: tree.leaves.map(l => ({
            from: l.fromClass,
            to: l.toClass,
            hash: l.leaf,
        })),
        proofs,
        metadata: {
            generatedAt: new Date().toISOString(),
            numTransitions: tree.leaves.length,
            merkleDepth: tree.tree.length,
        },
    };
    
    fs.writeFileSync(
        path.join(__dirname, "../policy_tree.json"),
        JSON.stringify(policyTree, null, 2)
    );
    
    console.log(` Policy tree saved to policy_tree.json`);
    
    // Generate Solidity deployment constant
    const solidityCode = `
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/// Auto-generated policy root constant
contract PolicyConstants {
    bytes32 public constant POLICY_ROOT = bytes32(0x${root.toString(16).padStart(64, '0')});
    
    uint256 public constant NUM_ALLOWED_TRANSITIONS = ${tree.leaves.length};
    uint256 public constant MERKLE_DEPTH = ${tree.tree.length};
}
`;
    
    fs.writeFileSync(
        path.join(__dirname, "../../contracts/contracts/PolicyConstants.sol"),
        solidityCode
    );
    
    console.log(` Solidity constants saved`);
    
    // Generate test input with proof
    console.log("\n Generating test input with policy proof...");
    
    // Example: User (1) → User (1)
    const testTransition = tree.getProof(1, 1);
    
    const testInput = {
        prevStateHash: "1",
        newStateHash: "2",
        epochId: "0",
        prevOriginClass: "1",
        newOriginClass: "1",
        prevLineageCommitment: "0",
        prevCounterCommitment: "0",
        policyRoot: root.toString(),
        expectedGenesisHash: "0",
        
        prevEpochId: "0",
        prevDepth: "0",
        nonce: "1",
        prevNonce: "0",
        timestamp: "1000",
        prevTimestamp: "999",
        policyProof: testTransition.proof.map(p => p.toString()),
        policyIndices: testTransition.indices,
        prevCounters: ["0", "0", "0", "0", "0", "0", "0"],
        rateLimits: ["1", "4294967295", "10", "100", "5", "1000", "1"],
        publicKeyX: "0",
        publicKeyY: "0",
        signatureR: "0",
        signatureS: "0",
        authorizationValid: "1"
    };
    
    fs.writeFileSync(
        path.join(__dirname, "../test/inputs/main_input.json"),
        JSON.stringify(testInput, null, 2)
    );
    
    console.log(` Test input saved`);
    
    // Verify a proof
    console.log("\n  Verifying policy proof...");
    const isValid = tree.verifyProof(1, 1, testTransition.proof, testTransition.indices);
    console.log(` Verification: ${isValid ? "PASS" : "FAIL"}`);
    

}

main().catch(console.error);