use super::*;
use poseidon_rs::{Poseidon, Fr, FrRepr};
use ff_ce::PrimeField;  // ← Use ff_ce instead of ff

/// Merkle tree for policy
#[derive(Debug, Clone)]
pub struct PolicyTree {
    depth: usize,
    leaves: Vec<Fr>,
    tree: Vec<Vec<Fr>>,
    root: Fr,
}

impl PolicyTree {
    /// Build policy tree from allowed transitions
    pub fn new(transitions: Vec<(OriginClass, OriginClass)>) -> Self {
        let depth = 6; // 2^6 = 64 leaves max
        let max_leaves = 1 << depth;
        
        // Hash all transitions
        let mut leaves: Vec<Fr> = transitions
            .iter()
            .map(|(from, to)| hash_transition(*from, *to))
            .collect();
        
        // Pad to power of 2 with zero Fr
        let zero_fr = Fr::from_repr(FrRepr::from(0u64)).unwrap();
        while leaves.len() < max_leaves {
            leaves.push(zero_fr);
        }
        
        // Build tree bottom-up
        let mut tree = vec![leaves.clone()];
        let poseidon = Poseidon::new();
        
        for level in 0..depth {
            let current_level = &tree[level];
            let mut next_level = Vec::new();
            
            for i in (0..current_level.len()).step_by(2) {
                let left = current_level[i];
                let right = current_level[i + 1];
                let parent = poseidon.hash(vec![left, right]).unwrap();
                next_level.push(parent);
            }
            
            tree.push(next_level);
        }
        
        let root = tree.last().unwrap()[0];
        
        Self {
            depth,
            leaves,
            tree,
            root,
        }
    }
    
    /// Get root
    pub fn root(&self) -> Fr {
        self.root
    }
    
    /// Get Merkle proof for transition
    pub fn prove(&self, from: OriginClass, to: OriginClass) -> Option<PolicyProof> {
        let leaf = hash_transition(from, to);
        
        // Find leaf index
        let leaf_index = self.leaves.iter().position(|&l| l == leaf)?;
        
        // Generate proof
        let mut path_elements = Vec::new();
        let mut path_indices = Vec::new();
        let mut index = leaf_index;
        
        for level in 0..self.depth {
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            let sibling = self.tree[level][sibling_index];
            
            path_elements.push(sibling);
            path_indices.push((index % 2) as u8);
            
            index /= 2;
        }
        
        Some(PolicyProof {
            from,
            to,
            leaf,
            path_elements,
            path_indices,
            root: self.root,
        })
    }
    
    /// Verify proof
    pub fn verify(&self, proof: &PolicyProof) -> bool {
        let poseidon = Poseidon::new();
        let mut current = proof.leaf;
        
        for i in 0..self.depth {
            let sibling = proof.path_elements[i];
            let is_right = proof.path_indices[i];
            
            current = if is_right == 0 {
                poseidon.hash(vec![current, sibling]).unwrap()
            } else {
                poseidon.hash(vec![sibling, current]).unwrap()
            };
        }
        
        current == proof.root
    }
}

/// Policy Merkle proof (NO SERIALIZE - Fr doesn't support it)
#[derive(Debug, Clone)]
pub struct PolicyProof {
    pub from: OriginClass,
    pub to: OriginClass,
    pub leaf: Fr,
    pub path_elements: Vec<Fr>,
    pub path_indices: Vec<u8>,
    pub root: Fr,
}

impl PolicyProof {
    /// Convert to circuit inputs
    pub fn to_circuit_inputs(&self) -> (Vec<String>, Vec<u8>) {
        let elements: Vec<String> = self
            .path_elements
            .iter()
            .map(|e| format!("{:?}", e))
            .collect();
        
        (elements, self.path_indices.clone())
    }
    
    /// Convert to hex strings for Solidity
    pub fn to_hex_elements(&self) -> Vec<String> {
        self.path_elements
            .iter()
            .map(|e| {
                let repr = e.into_repr();
                format!("0x{:064x}", repr.0[0])
            })
            .collect()
    }
    
    /// Get root as hex string
    pub fn root_hex(&self) -> String {
        let repr = self.root.into_repr();
        format!("0x{:064x}", repr.0[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_policy_tree() {
        let transitions = get_policy_leaves();
        let tree = PolicyTree::new(transitions.clone());
        
        // Test proof generation and verification
        for (from, to) in transitions.iter().take(5) {
            let proof = tree.prove(*from, *to).unwrap();
            assert!(tree.verify(&proof));
        }
    }
    
    #[test]
    fn test_invalid_transition() {
        let transitions = get_policy_leaves();
        let tree = PolicyTree::new(transitions);
        
        // User → Admin should not be in tree
        let proof = tree.prove(OriginClass::User, OriginClass::Admin);
        assert!(proof.is_none());
    }
    
    #[test]
    fn test_tree_depth() {
        let transitions = get_policy_leaves();
        let tree = PolicyTree::new(transitions);
        
        assert_eq!(tree.depth, 6);
        assert_eq!(tree.leaves.len(), 64);
    }
}