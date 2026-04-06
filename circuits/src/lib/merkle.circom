pragma circom 2.1.0;

include "./poseidon.circom";
include "./comparators.circom";

template MerkleProofVerifier(DEPTH) {
    signal input leaf;
    signal input root;
    signal input pathElements[DEPTH];
    signal input pathIndices[DEPTH];
    signal output valid;
    signal levelHashes[DEPTH + 1];
    levelHashes[0] <== leaf;
    for (var i = 0; i < DEPTH; i++) {
        pathIndices[i] * (pathIndices[i] - 1) === 0;
        signal left;
        signal right;
        left <== (1 - pathIndices[i]) * levelHashes[i] + pathIndices[i] * pathElements[i];
        right <== pathIndices[i] * levelHashes[i] + (1 - pathIndices[i]) * pathElements[i];
        component hasher = PoseidonHash2();
        hasher.in[0] <== left;
        hasher.in[1] <== right;
        levelHashes[i + 1] <== hasher.out;
    }
    component eq = IsEqual();
    eq.in[0] <== levelHashes[DEPTH];
    eq.in[1] <== root;
    valid <== eq.out;
}

template PolicyLeaf() {
    signal input fromOrigin;
    signal input toOrigin;
    signal output leaf;
    component hasher = PoseidonHash2();
    hasher.in[0] <== fromOrigin;
    hasher.in[1] <== toOrigin;
    leaf <== hasher.out;
}

template VerifyPolicyMembership(DEPTH) {
    signal input fromOrigin;
    signal input toOrigin;
    signal input policyRoot;
    signal input pathElements[DEPTH];
    signal input pathIndices[DEPTH];
    signal output isAllowed;
    component leaf = PolicyLeaf();
    leaf.fromOrigin <== fromOrigin;
    leaf.toOrigin <== toOrigin;
    component merkleVerifier = MerkleProofVerifier(DEPTH);
    merkleVerifier.leaf <== leaf.leaf;
    merkleVerifier.root <== policyRoot;
    for (var i = 0; i < DEPTH; i++) {
        merkleVerifier.pathElements[i] <== pathElements[i];
        merkleVerifier.pathIndices[i] <== pathIndices[i];
    }
    isAllowed <== merkleVerifier.valid;
}
