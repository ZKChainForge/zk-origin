const { expect } = require("chai");
const { buildPoseidon } = require("circomlibjs");

describe("Merkle Tree Policy", function() {
    let poseidon;
    let F;
    
    before(async function() {
        poseidon = await buildPoseidon();
        F = poseidon.F;
    });
    
    // Origin class enum
    const Origin = {
        Genesis: 0,
        User: 1,
        Admin: 2,
        Bridge: 3
    };
    
    // Allowed transitions
    const allowedTransitions = [
        [Origin.Genesis, Origin.User],
        [Origin.Genesis, Origin.Admin],
        [Origin.User, Origin.User],
        [Origin.Admin, Origin.User],
        [Origin.Admin, Origin.Admin],
        [Origin.Admin, Origin.Bridge],
        [Origin.Bridge, Origin.User]
    ];
    
    function hashPair(left, right) {
        const result = poseidon([left, right]);
        return F.toObject(result);
    }
    
    function computePolicyLeaf(from, to) {
        const result = poseidon([BigInt(from), BigInt(to)]);
        return F.toObject(result);
    }
    
    function buildMerkleTree(leaves) {
        if (leaves.length === 0) return { root: BigInt(0), layers: [] };
        
        // Pad to power of 2
        const targetLength = Math.pow(2, Math.ceil(Math.log2(leaves.length)));
        while (leaves.length < targetLength) {
            leaves.push(BigInt(0));
        }
        
        const layers = [leaves.slice()];
        
        while (layers[layers.length - 1].length > 1) {
            const currentLayer = layers[layers.length - 1];
            const nextLayer = [];
            
            for (let i = 0; i < currentLayer.length; i += 2) {
                const left = currentLayer[i];
                const right = currentLayer[i + 1];
                nextLayer.push(hashPair(left, right));
            }
            
            layers.push(nextLayer);
        }
        
        return {
            root: layers[layers.length - 1][0],
            layers
        };
    }
    
    function getMerkleProof(tree, index) {
        const path = [];
        const indices = [];
        
        let currentIndex = index;
        
        for (let i = 0; i < tree.layers.length - 1; i++) {
            const layer = tree.layers[i];
            const isLeft = currentIndex % 2 === 0;
            const siblingIndex = isLeft ? currentIndex + 1 : currentIndex - 1;
            
            path.push(layer[siblingIndex]);
            indices.push(isLeft ? 0 : 1);
            
            currentIndex = Math.floor(currentIndex / 2);
        }
        
        return { path, indices };
    }
    
    describe("Policy Tree Construction", function() {
        it("should build correct policy tree", function() {
            const leaves = allowedTransitions.map(([from, to]) => 
                computePolicyLeaf(from, to)
            );
            
            const tree = buildMerkleTree(leaves);
            
            expect(tree.root).to.be.a("bigint");
            expect(tree.layers.length).to.equal(4); // log2(8) + 1
        });
        
        it("should generate valid Merkle proofs", function() {
            const leaves = allowedTransitions.map(([from, to]) => 
                computePolicyLeaf(from, to)
            );
            
            const tree = buildMerkleTree(leaves);
            
            // Verify proof for each leaf
            for (let i = 0; i < allowedTransitions.length; i++) {
                const { path, indices } = getMerkleProof(tree, i);
                
                let current = leaves[i];
                for (let j = 0; j < path.length; j++) {
                    if (indices[j] === 0) {
                        current = hashPair(current, path[j]);
                    } else {
                        current = hashPair(path[j], current);
                    }
                }
                
                expect(current).to.equal(tree.root);
            }
        });
        
        it("should have correct leaf for User -> User", function() {
            const leaf = computePolicyLeaf(Origin.User, Origin.User);
            expect(leaf).to.be.a("bigint");
            
            // Verify it's in allowed list
            const leaves = allowedTransitions.map(([from, to]) => 
                computePolicyLeaf(from, to)
            );
            expect(leaves).to.include(leaf);
        });
        
        it("should NOT have leaf for User -> Admin", function() {
            const leaf = computePolicyLeaf(Origin.User, Origin.Admin);
            
            const leaves = allowedTransitions.map(([from, to]) => 
                computePolicyLeaf(from, to)
            );
            expect(leaves).to.not.include(leaf);
        });
    });
});