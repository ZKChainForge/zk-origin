pragma circom 2.1.0;

/*
  Define all protocol constants here for easy modification
 */

// Number of origin classes
// 0 = Genesis, 1 = User, 2 = Admin, 3 = Bridge
function NUM_ORIGIN_CLASSES() {
    return 4;
}

// Maximum lineage depth (2^32 - 1)
function MAX_LINEAGE_DEPTH() {
    return 4294967295;
}

// Policy tree depth (supports up to 16 allowed pairs)
function POLICY_MERKLE_DEPTH() {
    return 4;
}

// Epoch duration in seconds (24 hours)
function EPOCH_DURATION() {
    return 86400;
}

// Rate limits per origin class per epoch
function RATE_LIMIT_GENESIS() {
    return 1;
}

function RATE_LIMIT_USER() {
    return 4294967295; // Unlimited
}

function RATE_LIMIT_ADMIN() {
    return 10;
}

function RATE_LIMIT_BRIDGE() {
    return 100;
}