pragma circom 2.1.0;

/*
 * ZK-ORIGIN Constants
 * 
 * Centralized configuration for all circuits.
 * DO NOT modify without updating all circuits.
 */

// ============================================
// ORIGIN CLASSES (7 total)
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
function RATE_LIMIT_USER() { return 0xFFFFFFFF; }
function RATE_LIMIT_ADMIN() { return 10; }
function RATE_LIMIT_BRIDGE() { return 100; }
function RATE_LIMIT_GOVERNANCE() { return 5; }
function RATE_LIMIT_SYSTEM() { return 1000; }
function RATE_LIMIT_EMERGENCY() { return 1; }

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

// ============================================
// POLICY MATRIX (Allowed Transitions)
// ============================================
function isPolicyAllowed(from, to) {
    if (from == ORIGIN_CLASS_GENESIS()) {
        if (to == ORIGIN_CLASS_USER()) return 1;
        if (to == ORIGIN_CLASS_ADMIN()) return 1;
        if (to == ORIGIN_CLASS_SYSTEM()) return 1;
        return 0;
    }
    
    if (from == ORIGIN_CLASS_USER()) {
        if (to == ORIGIN_CLASS_USER()) return 1;
        return 0;
    }
    
    if (from == ORIGIN_CLASS_ADMIN()) {
        if (to == ORIGIN_CLASS_USER()) return 1;
        if (to == ORIGIN_CLASS_ADMIN()) return 1;
        if (to == ORIGIN_CLASS_BRIDGE()) return 1;
        if (to == ORIGIN_CLASS_SYSTEM()) return 1;
        return 0;
    }
    
    if (from == ORIGIN_CLASS_BRIDGE()) {
        if (to == ORIGIN_CLASS_USER()) return 1;
        return 0;
    }
    
    if (from == ORIGIN_CLASS_GOVERNANCE()) {
        if (to == ORIGIN_CLASS_USER()) return 1;
        if (to == ORIGIN_CLASS_ADMIN()) return 1;
        if (to == ORIGIN_CLASS_BRIDGE()) return 1;
        if (to == ORIGIN_CLASS_GOVERNANCE()) return 1;
        if (to == ORIGIN_CLASS_SYSTEM()) return 1;
        if (to == ORIGIN_CLASS_EMERGENCY()) return 1;
        return 0;
    }
    
    if (from == ORIGIN_CLASS_SYSTEM()) {
        if (to == ORIGIN_CLASS_USER()) return 1;
        if (to == ORIGIN_CLASS_SYSTEM()) return 1;
        return 0;
    }
    
    if (from == ORIGIN_CLASS_EMERGENCY()) {
        if (to == ORIGIN_CLASS_USER()) return 1;
        if (to == ORIGIN_CLASS_ADMIN()) return 1;
        if (to == ORIGIN_CLASS_SYSTEM()) return 1;
        return 0;
    }
    
    return 0;
}

// ============================================
// CIRCUIT PARAMETERS
// ============================================
function MAX_LINEAGE_DEPTH() { return 4294967295; }
function MAX_ADMIN_SIGNERS() { return 15; }
function ADMIN_MULTISIG_THRESHOLD() { return 2; }
function MAX_GOVERNANCE_VOTES() { return 1000; }
function GOVERNANCE_VOTE_THRESHOLD() { return 5000; }
function GOVERNANCE_TIMELOCK_SECONDS() { return 172800; }
function POLICY_MERKLE_DEPTH() { return 6; }
function EPOCH_DURATION_SECONDS() { return 86400; }
function MAX_EPOCH_NUMBER() { return 4294967295; }

// ============================================
// FIELD/CONSTRAINT PARAMETERS
// ============================================
function FIELD_BITS() { return 254; }
function TIMESTAMP_BITS() { return 32; }
function EPOCH_BITS() { return 32; }
function DEPTH_BITS() { return 32; }
function NONCE_BITS() { return 64; }