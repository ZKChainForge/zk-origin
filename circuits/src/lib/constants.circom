pragma circom 2.1.0;

// ============================================
// ORIGIN CLASSES
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
// RATE LIMITS (per 24-hour epoch)
// ============================================
function RATE_LIMIT_GENESIS() { return 1; }
function RATE_LIMIT_USER() { return 4294967295; }
function RATE_LIMIT_ADMIN() { return 10; }
function RATE_LIMIT_BRIDGE() { return 100; }
function RATE_LIMIT_GOVERNANCE() { return 5; }
function RATE_LIMIT_SYSTEM() { return 1000; }
function RATE_LIMIT_EMERGENCY() { return 1; }

// ============================================
// CIRCUIT PARAMETERS
// ============================================
function MAX_LINEAGE_DEPTH() { return 4294967295; }
function POLICY_MERKLE_DEPTH() { return 4; }
function BRIDGE_MERKLE_DEPTH() { return 8; }
function MAX_ADMIN_SIGNERS() { return 15; }
function MAX_BRIDGE_VALIDATORS() { return 21; }
function MIN_BRIDGE_CONFIRMATIONS() { return 64; }
function BRIDGE_QUORUM_NUMERATOR() { return 2; }
function BRIDGE_QUORUM_DENOMINATOR() { return 3; }
function GOVERNANCE_TIMELOCK_SECONDS() { return 172800; }
function MAX_GOVERNANCE_VOTES() { return 1000000000; }
function EMERGENCY_TVL_MULTIPLIER() { return 2; }
function EMERGENCY_MAX_BLOCK_TIME() { return 3600; }
function EPOCH_DURATION_SECONDS() { return 86400; }
function COUNTER_MAX() { return 4294967295; }
function MAX_NONCE() { return 18446744073709551615; }