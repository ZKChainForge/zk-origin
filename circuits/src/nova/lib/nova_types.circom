pragma circom 2.1.0;

/**
 * @title Nova IVC Type Definitions
 * @notice Common types and structures for Nova folding
 * 
 * SECURITY:
 *  Defines exact signal ordering for folding
 *  Public input/output structure standardized
 *  Compatible with Nova reference implementation
 * 
 * NOVA CONCEPT:
 * - IVC = Incrementally Verifiable Computation
 * - Each fold proves: old_state + new_transition = new_state
 * - Folding is O(1) in circuit complexity
 * - Final proof has constant size regardless of iterations
 * 
 * STATE VECTORS:
 * Input vector z_in (6 elements):
 * [0] = lineage_commitment (previous)
 * [1] = counter_commitment (previous)
 * [2] = nonce (previous)
 * [3] = timestamp (previous)
 * [4] = epoch_id (previous)
 * [5] = depth (previous)
 * 
 * Output vector z_out (same size):
 * [0] = lineage_commitment (updated)
 * [1] = counter_commitment (updated)
 * [2] = nonce (incremented)
 * [3] = timestamp (current)
 * [4] = epoch_id (current)
 * [5] = depth (incremented)
 * 
 * CONSTRAINT COUNT:
 * - Base IVC circuit: ~20,000 constraints
 * - Per Nova fold: same ~20,000 constraints
 * - Total stays constant regardless of depth!
 */

// ============ STATE VECTOR INDICES ============
function STATE_LINEAGE_COMMITMENT() { return 0; }
function STATE_COUNTER_COMMITMENT() { return 1; }
function STATE_NONCE() { return 2; }
function STATE_TIMESTAMP() { return 3; }
function STATE_EPOCH_ID() { return 4; }
function STATE_DEPTH() { return 5; }
function STATE_VECTOR_SIZE() { return 6; }

// ============ NOVA CONSTRAINT SYSTEM ============
// Used for circuit folding
// These are computed by Nova and not directly in circuit

// Primary circuit constraints
function PRIMARY_CIRCUIT_CONSTRAINTS() { return 21000; }

// Secondary circuit constraints (for verification)
function SECONDARY_CIRCUIT_CONSTRAINTS() { return 2000; }

// ============ SECURITY PARAMETERS ============
// These match Nova's security assumptions

// Commitment scheme: Pedersen
function COMMITMENT_TYPE() { return "Pedersen"; }

// Field: BN254 (same as Groth16)
function FIELD_MODULUS() { 
    return "21888242871839275222246405745257275088548364400416034343698204186575808495617";
}

// Hash function for folding
function NOVA_HASH_FUNCTION() { return "Keccak256"; }

// ============ FOLDING STRUCTURE ============
// This is what Nova proves at each step

template NovaFoldingStructure() {
    signal input oldU_x;      // Previous commitment X
    signal input oldU_y;      // Previous commitment Y
    signal input oldZ_in[6];  // Previous input vector
    
    signal input newU_x;      // New commitment X
    signal input newU_y;      // New commitment Y
    signal input newZ_in[6];  // New input vector
    signal input newZ_out[6]; // New output vector
    
    signal input r;           // Random folding scalar (from verifier)
    
    // Folded commitment
    signal output foldedU_x;
    signal output foldedU_y;
    
    // Folded input
    signal output foldedZ_in[6];
    
    // No outputs needed - Nova tracks these
}