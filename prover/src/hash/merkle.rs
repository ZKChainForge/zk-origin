//! Merkle tree implementation for policy verification

use super::poseidon::poseidon_hash_two;
use serde::{Deserialize, Serialize};

/// A Merkle tree for efficient set membership proofs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleTree {
    /// Depth of the tree
    depth: usize,

    /// All nodes in the tree (level by level, bottom to top)
    nodes: Vec<Vec<[u8; 32]>>,

    /// Leaves of the tree
    leaves: Vec<[u8; 32]>,
}

impl MerkleTree {
    /// Create a new Merkle tree from leaves
    pub fn new(leaves: Vec<[u8; 32]>) -> Self {
        if leaves.is_empty() {
            return Self {
                depth: 0,
                nodes: vec![vec![[0u8; 32]]],
                leaves: vec![],
            };
        }

        // Pad to power of 2
        let n = leaves.len().next_power_of_two();
        let depth = (n as f64).log2() as usize;

        let mut padded_leaves = leaves.clone();
        while padded_leaves.len() < n {
            padded_leaves.push([0u8; 32]); // Pad with zeros
        }

        // Build tree bottom-up
        let mut nodes = vec![padded_leaves.clone()];
        let mut current_level = padded_leaves;

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let hash = poseidon_hash_two(&chunk[0], &chunk[1]);
                next_level.push(hash);
            }
            nodes.push(next_level.clone());
            current_level = next_level;
        }

        Self {
            depth,
            nodes,
            leaves,
        }
    }

    /// Get the root of the tree
    pub fn root(&self) -> [u8; 32] {
        self.nodes
            .last()
            .and_then(|level| level.first())
            .copied()
            .unwrap_or([0u8; 32])
    }

    /// Get the depth of the tree
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Get the number of leaves
    pub fn num_leaves(&self) -> usize {
        self.leaves.len()
    }

    /// Generate a proof for a leaf at the given index
    pub fn prove(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.nodes[0].len() {
            return None;
        }

        let mut path = Vec::new();
        let mut indices = Vec::new();
        let mut current_index = index;

        for level in 0..self.depth {
            let sibling_index = if current_index.is_multiple_of(2) {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < self.nodes[level].len() {
                path.push(self.nodes[level][sibling_index]);
            } else {
                path.push([0u8; 32]);
            }

            // Index indicates if we're the left (false) or right (true) child
            indices.push(current_index % 2 == 1);
            current_index /= 2;
        }

        Some(MerkleProof {
            leaf: self.nodes[0][index],
            path,
            indices,
            root: self.root(),
        })
    }

    /// Find and prove a leaf value
    pub fn prove_value(&self, value: &[u8; 32]) -> Option<MerkleProof> {
        let index = self.nodes[0].iter().position(|v| v == value)?;
        self.prove(index)
    }

    /// Verify a proof against this tree
    pub fn verify_proof(&self, proof: &MerkleProof) -> bool {
        proof.verify() && proof.root == self.root()
    }
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new(vec![])
    }
}

/// A Merkle proof for set membership
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleProof {
    /// The leaf being proven
    pub leaf: [u8; 32],

    /// Sibling hashes along the path to root
    pub path: Vec<[u8; 32]>,

    /// Path indices (true = leaf is right child, false = left)
    pub indices: Vec<bool>,

    /// The expected root
    pub root: [u8; 32],
}

impl MerkleProof {
    /// Verify this proof
    pub fn verify(&self) -> bool {
        if self.path.len() != self.indices.len() {
            return false;
        }

        let mut current = self.leaf;

        for (sibling, &is_right) in self.path.iter().zip(self.indices.iter()) {
            current = if is_right {
                poseidon_hash_two(sibling, &current)
            } else {
                poseidon_hash_two(&current, sibling)
            };
        }

        current == self.root
    }

    /// Get the depth of this proof
    pub fn depth(&self) -> usize {
        self.path.len()
    }

    /// Get path as bytes (for circuit input)
    pub fn path_bytes(&self) -> &[[u8; 32]] {
        &self.path
    }

    /// Get indices as bools
    pub fn indices_bools(&self) -> &[bool] {
        &self.indices
    }
}

/// Build a Merkle tree for origin policy
pub fn build_policy_tree(allowed_transitions: &[(u8, u8)]) -> (MerkleTree, Vec<(u8, u8, usize)>) {
    use super::poseidon::compute_policy_leaf;

    // Compute leaves for all allowed transitions
    let mut leaves = Vec::new();
    let mut mapping = Vec::new();

    for &(from, to) in allowed_transitions {
        let leaf = compute_policy_leaf(from, to);
        mapping.push((from, to, leaves.len()));
        leaves.push(leaf);
    }

    let tree = MerkleTree::new(leaves);
    (tree, mapping)
}

/// Generate a policy proof for a transition
pub fn generate_policy_proof(
    tree: &MerkleTree,
    mapping: &[(u8, u8, usize)],
    from: u8,
    to: u8,
) -> Option<MerkleProof> {
    // Find the index for this transition
    let index = mapping
        .iter()
        .find(|(f, t, _)| *f == from && *t == to)
        .map(|(_, _, idx)| *idx)?;

    tree.prove(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_creation() {
        let leaves: Vec<[u8; 32]> = (0..4)
            .map(|i| {
                let mut arr = [0u8; 32];
                arr[0] = i;
                arr
            })
            .collect();

        let tree = MerkleTree::new(leaves);

        assert_eq!(tree.depth(), 2);
        assert_eq!(tree.num_leaves(), 4);
    }

    #[test]
    fn test_merkle_proof_verify() {
        let leaves: Vec<[u8; 32]> = (0..4)
            .map(|i| {
                let mut arr = [0u8; 32];
                arr[0] = i;
                arr
            })
            .collect();

        let tree = MerkleTree::new(leaves);

        for i in 0..4 {
            let proof = tree.prove(i).unwrap();
            assert!(proof.verify());
            assert!(tree.verify_proof(&proof));
        }
    }

    #[test]
    fn test_merkle_proof_invalid() {
        let leaves: Vec<[u8; 32]> = (0..4)
            .map(|i| {
                let mut arr = [0u8; 32];
                arr[0] = i;
                arr
            })
            .collect();

        let tree = MerkleTree::new(leaves);
        let mut proof = tree.prove(0).unwrap();

        // Tamper with the proof
        proof.leaf[0] = 99;

        assert!(!proof.verify());
    }

    #[test]
    fn test_merkle_tree_root_deterministic() {
        let leaves1: Vec<[u8; 32]> = (0..4)
            .map(|i| {
                let mut arr = [0u8; 32];
                arr[0] = i;
                arr
            })
            .collect();

        let leaves2 = leaves1.clone();

        let tree1 = MerkleTree::new(leaves1);
        let tree2 = MerkleTree::new(leaves2);

        assert_eq!(tree1.root(), tree2.root());
    }

    #[test]
    fn test_merkle_padding() {
        // 3 leaves should be padded to 4
        let leaves: Vec<[u8; 32]> = (0..3)
            .map(|i| {
                let mut arr = [0u8; 32];
                arr[0] = i;
                arr
            })
            .collect();

        let tree = MerkleTree::new(leaves);

        assert_eq!(tree.depth(), 2); // log2(4) = 2
    }

    #[test]
    fn test_policy_tree() {
        use crate::types::OriginClass;

        let allowed = vec![
            (OriginClass::Genesis as u8, OriginClass::User as u8),
            (OriginClass::User as u8, OriginClass::User as u8),
            (OriginClass::Admin as u8, OriginClass::User as u8),
        ];

        let (tree, mapping) = build_policy_tree(&allowed);

        // Should be able to prove allowed transitions
        let proof = generate_policy_proof(
            &tree,
            &mapping,
            OriginClass::User as u8,
            OriginClass::User as u8,
        );
        assert!(proof.is_some());
        assert!(proof.unwrap().verify());

        // Should not be able to prove disallowed transitions
        let proof = generate_policy_proof(
            &tree,
            &mapping,
            OriginClass::User as u8,
            OriginClass::Admin as u8,
        );
        assert!(proof.is_none());
    }

    #[test]
    fn test_prove_value() {
        let leaves: Vec<[u8; 32]> = (0..4)
            .map(|i| {
                let mut arr = [0u8; 32];
                arr[0] = i;
                arr
            })
            .collect();

        let tree = MerkleTree::new(leaves.clone());

        let proof = tree.prove_value(&leaves[2]).unwrap();
        assert!(proof.verify());
        assert_eq!(proof.leaf, leaves[2]);
    }

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::new(vec![]);

        assert_eq!(tree.depth(), 0);
        assert_eq!(tree.num_leaves(), 0);
    }

    #[test]
    fn test_single_leaf() {
        let leaf = [42u8; 32];
        let tree = MerkleTree::new(vec![leaf]);

        let proof = tree.prove(0).unwrap();
        assert!(proof.verify());
    }

    #[test]
    fn test_serialization() {
        let leaves: Vec<[u8; 32]> = (0..4)
            .map(|i| {
                let mut arr = [0u8; 32];
                arr[0] = i;
                arr
            })
            .collect();

        let tree = MerkleTree::new(leaves);
        let proof = tree.prove(0).unwrap();

        let json = serde_json::to_string(&proof).unwrap();
        let recovered: MerkleProof = serde_json::from_str(&json).unwrap();

        assert!(recovered.verify());
        assert_eq!(proof.root, recovered.root);
    }
}
