pragma circom 2.1.0;

/**
 * @title ZK-ORIGIN Constants (CANONICAL VERSION)
 * @notice SINGLE SOURCE OF TRUTH for all circuit constants
 * 
 * SECURITY NOTES:
 * - Do not create duplicate constants files
 * - All contracts must reference this file
 * - Changes must be synchronized across all systems
 * 
 * PRODUCTION CHECKLIST:
 *  Immutable constants
 *  No magic numbers elsewhere
 *  Documented rate limits
 *  Verified against contract deployments
 */

// ============================================
// ORIGIN CLASSES (0-6 = 7 total)
// ============================================
function NUM_ORIGIN_CLASSES() { return 7; }
function ORIGIN_CLASS_GENESIS() { return 0; }
function ORIGIN_CLASS_USER() { return 1; }
function ORIGIN_CLASS_ADMIN() { return 2; }
function ORIGIN_CLASS_BRIDGE() { return 3; }
function ORIGIN_CLASS_GOVERNANCE() { return 4; }
function ORIGIN_CLASS_SYSTEM() { return 5; }
function ORIGIN_CLASS_EMERGENCY() { return 6; }

// ============================================
// RATE LIMITS (per epoch = 24 hours)
// ============================================
function RATE_LIMIT_GENESIS() { return 1; }
function RATE_LIMIT_USER() { return 4294967295; }           // u32::MAX (unlimited)
function RATE_LIMIT_ADMIN() { return 10; }
function RATE_LIMIT_BRIDGE() { return 100; }
function RATE_LIMIT_GOVERNANCE() { return 5; }
function RATE_LIMIT_SYSTEM() { return 1000; }
function RATE_LIMIT_EMERGENCY() { return 1; }

// Helper: Get rate limit by class
function GET_RATE_LIMIT(originClass) {
    if (originClass == ORIGIN_CLASS_GENESIS()) return RATE_LIMIT_GENESIS();
    if (originClass == ORIGIN_CLASS_USER()) return RATE_LIMIT_USER();
    if (originClass == ORIGIN_CLASS_ADMIN()) return RATE_LIMIT_ADMIN();
    if (originClass == ORIGIN_CLASS_BRIDGE()) return RATE_LIMIT_BRIDGE();
    if (originClass == ORIGIN_CLASS_GOVERNANCE()) return RATE_LIMIT_GOVERNANCE();
    if (originClass == ORIGIN_CLASS_SYSTEM()) return RATE_LIMIT_SYSTEM();
    if (originClass == ORIGIN_CLASS_EMERGENCY()) return RATE_LIMIT_EMERGENCY();
    return 0;
}

// ============================================
// CIRCUIT ARCHITECTURE PARAMETERS
// ============================================

// Maximum lineage depth before overflow protection
function MAX_LINEAGE_DEPTH() { return 4294967295; }    // u32::MAX

// Merkle tree depths
function POLICY_MERKLE_DEPTH() { return 6; }           // 2^6 = 64 transitions max
function BRIDGE_MERKLE_DEPTH() { return 8; }           // For bridge attestations

// Admin multisig parameters
function MAX_ADMIN_SIGNERS() { return 15; }            // Max signers in multisig
function ADMIN_MULTISIG_THRESHOLD() { return 2; }      // M-of-N default

// Governance voting
function MAX_GOVERNANCE_VOTES() { return 1000000000; } // 1 billion max votes
function GOVERNANCE_VOTE_THRESHOLD() { return 5000; }  // Votes needed to pass
function GOVERNANCE_TIMELOCK_SECONDS() { return 172800; } // 48 hours

// ============================================
// BRIDGE PARAMETERS (CROSS-CHAIN SPECIFIC)
// ============================================

// Minimum confirmations before finality
function MIN_BRIDGE_CONFIRMATIONS() { return 64; }

// Validator quorum for bridge attestations
function MAX_BRIDGE_VALIDATORS() { return 21; }
function BRIDGE_QUORUM_NUMERATOR() { return 2; }       // 2/3 quorum
function BRIDGE_QUORUM_DENOMINATOR() { return 3; }

// ============================================
// EMERGENCY PARAMETERS
// ============================================

// TVL multiplier for emergency detection (2x = emergency)
function EMERGENCY_TVL_MULTIPLIER() { return 2; }

// Max time between blocks before emergency (1 hour)
function EMERGENCY_MAX_BLOCK_TIME() { return 3600; }

// ============================================
// EPOCH MANAGEMENT
// ============================================

function EPOCH_DURATION_SECONDS() { return 86400; }    // 24 hours
function MAX_EPOCH_NUMBER() { return 4294967295; }     // u32::MAX

// ============================================
// FIELD AND BIT PARAMETERS
// ============================================

function FIELD_BITS() { return 254; }                  // Poseidon field size
function TIMESTAMP_BITS() { return 32; }
function EPOCH_BITS() { return 32; }
function DEPTH_BITS() { return 32; }
function NONCE_BITS() { return 64; }
function COUNTER_MAX() { return 4294967295; }          // u32::MAX

// ============================================
// SECURITY PARAMETERS
// ============================================

// Hash output size in bits
function HASH_BITS() { return 254; }

// State hash size in bits
function STATE_HASH_BITS() { return 254; }

// Commitment size in bits
function COMMITMENT_BITS() { return 254; }

// ============================================
// VALIDATION PARAMETERS
// ============================================

// Minimum number of signers
function MIN_SIGNERS() { return 1; }

// Maximum nonce before rollover
function MAX_NONCE() { return 18446744073709551615; } // 2^64 - 1

// Verification constraints budget per component
// (informal - for optimization planning)
function POSEIDON_CONSTRAINTS_PER_INPUT() { return 40; }  // ~300 total for 2 inputs
function EDDSA_CONSTRAINTS() { return 7500; }             // EdDSA verification
function MERKLE_CONSTRAINTS_PER_LEVEL() { return 300; }   // Per Merkle level

// ============================================
// VERSION AND COMPATIBILITY
// ============================================

function CIRCUIT_VERSION() { return 2; }

// Increment when circuit ABI changes
function CIRCUIT_ABI_VERSION() { return 1; }

// Minimum compatible contract version
function MIN_CONTRACT_VERSION() { return 2; }