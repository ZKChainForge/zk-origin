//! Native Poseidon hash implementation using Neptune
//!
//! This provides off-chain Poseidon hashing that matches
//! the in-circuit Poseidon implementation.

use neptune::poseidon::PoseidonConstants;
use neptune::Poseidon;
use pasta_curves::pallas::Scalar as Fr;
use ff::PrimeField;
use ff::{ Field};
use generic_array::typenum::{U2, U3, U4, U5};

/// Poseidon constants for different arities
#[allow(dead_code)]
pub struct PoseidonParams {
    pub constants_2: PoseidonConstants<Fr, U2>,
    pub constants_3: PoseidonConstants<Fr, U3>,
    pub constants_4: PoseidonConstants<Fr, U4>,
    pub constants_5: PoseidonConstants<Fr, U5>,
}

impl PoseidonParams {
    /// Create new Poseidon parameters
    pub fn new() -> Self {
        Self {
            constants_2: PoseidonConstants::new(),
            constants_3: PoseidonConstants::new(),
            constants_4: PoseidonConstants::new(),
            constants_5: PoseidonConstants::new(),
        }
    }
}

impl Default for PoseidonParams {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    /// Global Poseidon parameters (expensive to create, so we cache them)
    pub static ref POSEIDON_PARAMS: PoseidonParams = PoseidonParams::new();
}

/// Native Poseidon hasher using Neptune
#[derive(Clone, Debug)]
pub struct NativePoseidonHasher;

impl NativePoseidonHasher {
    /// Create a new hasher
    pub fn new() -> Self {
        Self
    }

    /// Convert 32 bytes to field element
    pub fn bytes_to_field(bytes: &[u8; 32]) -> Fr {
        // Take first 31 bytes to ensure we're in the field
        let mut repr = [0u8; 32];
        repr[..31].copy_from_slice(&bytes[..31]);
        Fr::from_repr(repr.into()).unwrap_or(Fr::ZERO)
    }

    /// Convert field element to 32 bytes
    pub fn field_to_bytes(field: &Fr) -> [u8; 32] {
        let repr = field.to_repr();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(repr.as_ref());
        bytes
    }

    /// Hash two field elements
    pub fn hash2(&self, a: Fr, b: Fr) -> Fr {
        let mut poseidon = Poseidon::new_with_preimage(
            &[a, b],
            &POSEIDON_PARAMS.constants_2,
        );
        poseidon.hash()
    }

    /// Hash three field elements
    pub fn hash3(&self, a: Fr, b: Fr, c: Fr) -> Fr {
        let mut poseidon = Poseidon::new_with_preimage(
            &[a, b, c],
            &POSEIDON_PARAMS.constants_3,
        );
        poseidon.hash()
    }

    /// Hash four field elements
    pub fn hash4(&self, a: Fr, b: Fr, c: Fr, d: Fr) -> Fr {
        let mut poseidon = Poseidon::new_with_preimage(
            &[a, b, c, d],
            &POSEIDON_PARAMS.constants_4,
        );
        poseidon.hash()
    }

    /// Hash five field elements
    pub fn hash5(&self, a: Fr, b: Fr, c: Fr, d: Fr, e: Fr) -> Fr {
        let mut poseidon = Poseidon::new_with_preimage(
            &[a, b, c, d, e],
            &POSEIDON_PARAMS.constants_5,
        );
        poseidon.hash()
    }

    /// Hash two byte arrays
    pub fn hash2_bytes(&self, a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        let fa = Self::bytes_to_field(a);
        let fb = Self::bytes_to_field(b);
        let result = self.hash2(fa, fb);
        Self::field_to_bytes(&result)
    }

    /// Hash three byte arrays
    pub fn hash3_bytes(&self, a: &[u8; 32], b: &[u8; 32], c: &[u8; 32]) -> [u8; 32] {
        let fa = Self::bytes_to_field(a);
        let fb = Self::bytes_to_field(b);
        let fc = Self::bytes_to_field(c);
        let result = self.hash3(fa, fb, fc);
        Self::field_to_bytes(&result)
    }

    /// Compute lineage commitment
    pub fn compute_lineage_commitment(
        &self,
        prev_commitment: &[u8; 32],
        transition_hash: &[u8; 32],
        depth: u64,
    ) -> [u8; 32] {
        let prev = Self::bytes_to_field(prev_commitment);
        let trans = Self::bytes_to_field(transition_hash);
        let d = Fr::from(depth);
        
        let result = self.hash3(prev, trans, d);
        Self::field_to_bytes(&result)
    }

    /// Compute transition hash
    pub fn compute_transition_hash(
        &self,
        prev_state: &[u8; 32],
        new_state: &[u8; 32],
        origin: u8,
        timestamp: u64,
        epoch: u64,
    ) -> [u8; 32] {
        let ps = Self::bytes_to_field(prev_state);
        let ns = Self::bytes_to_field(new_state);
        let o = Fr::from(origin as u64);
        let t = Fr::from(timestamp);
        let e = Fr::from(epoch);
        
        let result = self.hash5(ps, ns, o, t, e);
        Self::field_to_bytes(&result)
    }

    /// Compute policy leaf
    pub fn compute_policy_leaf(&self, from_origin: u8, to_origin: u8) -> [u8; 32] {
        let from = Fr::from(from_origin as u64);
        let to = Fr::from(to_origin as u64);
        
        let result = self.hash2(from, to);
        Self::field_to_bytes(&result)
    }

    /// Compute genesis commitment
    pub fn compute_genesis_commitment(&self, genesis_state: &[u8; 32]) -> [u8; 32] {
        let state = Self::bytes_to_field(genesis_state);
        let zero = Fr::ZERO;
        
        let result = self.hash3(state, zero, zero);
        Self::field_to_bytes(&result)
    }

    /// Compute counter commitment
    pub fn compute_counter_commitment(&self, epoch: u64, counters: &[u32; 6]) -> [u8; 32] {
        // We'll hash in chunks since we have 7 elements (epoch + 6 counters)
        let e = Fr::from(epoch);
        let c0 = Fr::from(counters[0] as u64);
        let c1 = Fr::from(counters[1] as u64);
        let c2 = Fr::from(counters[2] as u64);
        let c3 = Fr::from(counters[3] as u64);
        
        // First hash: epoch, c0, c1, c2, c3
        let h1 = self.hash5(e, c0, c1, c2, c3);
        
        // Second hash: h1, c4, c5
        let c4 = Fr::from(counters[4] as u64);
        let c5 = Fr::from(counters[5] as u64);
        let result = self.hash3(h1, c4, c5);
        
        Self::field_to_bytes(&result)
    }
}

impl Default for NativePoseidonHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash2_deterministic() {
        let hasher = NativePoseidonHasher::new();
        
        let a = [1u8; 32];
        let b = [2u8; 32];
        
        let h1 = hasher.hash2_bytes(&a, &b);
        let h2 = hasher.hash2_bytes(&a, &b);
        
        assert_eq!(h1, h2);
        assert_ne!(h1, [0u8; 32]);
    }

    #[test]
    fn test_hash2_different_inputs() {
        let hasher = NativePoseidonHasher::new();
        
        let a = [1u8; 32];
        let b = [2u8; 32];
        let c = [3u8; 32];
        
        let h1 = hasher.hash2_bytes(&a, &b);
        let h2 = hasher.hash2_bytes(&a, &c);
        
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_lineage_commitment() {
        let hasher = NativePoseidonHasher::new();
        
        let prev = [1u8; 32];
        let trans = [2u8; 32];
        
        let c1 = hasher.compute_lineage_commitment(&prev, &trans, 5);
        let c2 = hasher.compute_lineage_commitment(&prev, &trans, 5);
        let c3 = hasher.compute_lineage_commitment(&prev, &trans, 6);
        
        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }

    #[test]
    fn test_genesis_commitment() {
        let hasher = NativePoseidonHasher::new();
        
        let genesis = [42u8; 32];
        let commitment = hasher.compute_genesis_commitment(&genesis);
        
        assert_ne!(commitment, [0u8; 32]);
        assert_ne!(commitment, genesis);
    }

    #[test]
    fn test_field_conversion_roundtrip() {
        let original = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
                       0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
                       0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00,
                       0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x00]; // Last byte 0 to stay in field
        
        let field = NativePoseidonHasher::bytes_to_field(&original);
        let recovered = NativePoseidonHasher::field_to_bytes(&field);
        
        // First 31 bytes should match (we truncate to stay in field)
        assert_eq!(&original[..31], &recovered[..31]);
    }
}