pragma circom 2.1.0;

/*
 * ZK-ORIGIN Constants (CANONICAL VERSION)
 * 
 *  DO NOT CREATE DUPLICATE constants.circom FILES
 * This is the SINGLE SOURCE OF TRUTH for all constants.
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
// RATE LIMITS (per epoch)
// ============================================
function RATE_LIMIT_GENESIS() { return 1; }
function RATE_LIMIT_USER() { return 4294967295; }      // Unlimited (u32::MAX)
function RATE_LIMIT_ADMIN() { return 10; }
function RATE_LIMIT_BRIDGE() { return 100; }
function RATE_LIMIT_GOVERNANCE() { return 5; }
function RATE_LIMIT_SYSTEM() { return 1000; }
function RATE_LIMIT_EMERGENCY() { return 1; }

// ============================================
// CIRCUIT PARAMETERS
// ============================================
function MAX_LINEAGE_DEPTH() { return 4294967295; }    // u32::MAX
function MAX_ADMIN_SIGNERS() { return 15; }
function ADMIN_MULTISIG_THRESHOLD() { return 2; }
function MAX_GOVERNANCE_VOTES() { return 1000000000; } // 1 billion max
function GOVERNANCE_VOTE_THRESHOLD() { return 5000; }
function GOVERNANCE_TIMELOCK_SECONDS() { return 172800; } // 48 hours
function POLICY_MERKLE_DEPTH() { return 6; }
function EPOCH_DURATION_SECONDS() { return 86400; }    // 24 hours
function MAX_EPOCH_NUMBER() { return 4294967295; }     // u32::MAX

// ============================================
// BRIDGE PARAMETERS
// ============================================
function MIN_BRIDGE_CONFIRMATIONS() { return 64; }
function MAX_BRIDGE_VALIDATORS() { return 21; }
function BRIDGE_QUORUM_NUMERATOR() { return 2; }       // 2/3 quorum
function BRIDGE_QUORUM_DENOMINATOR() { return 3; }

// ============================================
// EMERGENCY PARAMETERS
// ============================================
function EMERGENCY_TVL_MULTIPLIER() { return 2; }      // 2x normal TVL = emergency
function EMERGENCY_MAX_BLOCK_TIME() { return 3600; }   // 1 hour stall = emergency

// ============================================
// FIELD/CONSTRAINT PARAMETERS
// ============================================
function FIELD_BITS() { return 254; }
function TIMESTAMP_BITS() { return 32; }
function EPOCH_BITS() { return 32; }
function DEPTH_BITS() { return 32; }
function NONCE_BITS() { return 64; }
function COUNTER_MAX() { return 4294967295; }          // u32::MAX

// ============================================
// HELPER: GET RATE LIMIT BY CLASS
// ============================================
function getRateLimit(originClass) {
    if (originClass == ORIGIN_CLASS_GENESIS()) return RATE_LIMIT_GENESIS();
    if (originClass == ORIGIN_CLASS_USER()) return RATE_LIMIT_USER();
    if (originClass == ORIGIN_CLASS_ADMIN()) return RATE_LIMIT_ADMIN();
    if (originClass == ORIGIN_CLASS_BRIDGE()) return RATE_LIMIT_BRIDGE();
    if (originClass == ORIGIN_CLASS_GOVERNANCE()) return RATE_LIMIT_GOVERNANCE();
    if (originClass == ORIGIN_CLASS_SYSTEM()) return RATE_LIMIT_SYSTEM();
    if (originClass == ORIGIN_CLASS_EMERGENCY()) return RATE_LIMIT_EMERGENCY();
    return 0;
}