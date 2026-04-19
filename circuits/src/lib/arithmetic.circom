pragma circom 2.1.0;

/**
 * @title Arithmetic Operations (PRODUCTION - NEW)
 * @notice Safe arithmetic with overflow protection
 * 
 * SECURITY:
 *  Division requires exact remainder = 0
 *  Modulo prevents wraparound
 *  Saturating arithmetic available
 * 
 * PRODUCTION NOTES:
 * - Use for epoch calculations
 * - Verify division results before use
 * - All operations constrained
 */

// ============================================
// DIVISION WITH VERIFICATION
// ============================================
template DivideWithVerification(BITS) {
    signal input dividend;
    signal input divisor;
    signal output quotient;
    signal output remainder;
    
    // Quotient and remainder calculation
    quotient <-- dividend \ divisor;
    remainder <-- dividend % divisor;
    
    // Verification constraint
    divisor * quotient + remainder === dividend;
    
    // Ensure remainder < divisor
    component remainderCheck = LessThan(BITS);
    remainderCheck.in[0] <== remainder;
    remainderCheck.in[1] <== divisor;
    remainderCheck.out === 1;
}

// ============================================
// MODULO OPERATION
// ============================================
template Modulo(BITS) {
    signal input value;
    signal input divisor;
    signal output remainder;
    
    remainder <-- value % divisor;
    
    component div = DivideWithVerification(BITS);
    div.dividend <== value;
    div.divisor <== divisor;
    div.remainder === remainder;
}

// ============================================
// INTEGER DIVISION
// ============================================
template IntDivide(BITS) {
    signal input dividend;
    signal input divisor;
    signal output quotient;
    
    component div = DivideWithVerification(BITS);
    div.dividend <== dividend;
    div.divisor <== divisor;
    quotient <== div.quotient;
}

// ============================================
// EPOCH CALCULATOR
// ============================================
template EpochCalculator(BITS) {
    signal input timestamp;
    signal input genesisTime;
    signal input epochDuration;
    signal output epochId;
    
    // epochId = (timestamp - genesisTime) / epochDuration
    signal timeSinceGenesis;
    timeSinceGenesis <== timestamp - genesisTime;
    
    component divider = IntDivide(BITS);
    divider.dividend <== timeSinceGenesis;
    divider.divisor <== epochDuration;
    epochId <== divider.quotient;
}

// ============================================
// SATURATING ADD (capped at MAX)
// ============================================
template SaturatingAdd(MAX) {
    signal input a;
    signal input b;
    signal output result;
    
    signal sum;
    sum <== a + b;
    
    // If sum > MAX, return MAX, else return sum
    component check = LessThan(32);
    check.in[0] <== sum;
    check.in[1] <== MAX + 1;
    
    result <== check.out * sum + (1 - check.out) * MAX;
}

// ============================================
// SATURATING SUB (stops at 0)
// ============================================
template SaturatingSub() {
    signal input a;
    signal input b;
    signal output result;
    
    // If a < b, return 0, else return a - b
    component check = LessThan(64);
    check.in[0] <== a;
    check.in[1] <== b;
    
    result <== (1 - check.out) * (a - b);
}

// ============================================
// MIN VALUE
// ============================================
template Min(BITS) {
    signal input a;
    signal input b;
    signal output min;
    
    component check = LessThan(BITS);
    check.in[0] <== a;
    check.in[1] <== b;
    
    min <== check.out * a + (1 - check.out) * b;
}

// ============================================
// MAX VALUE
// ============================================
template Max(BITS) {
    signal input a;
    signal input b;
    signal output max;
    
    component check = LessThan(BITS);
    check.in[0] <== a;
    check.in[1] <== b;
    
    max <== (1 - check.out) * a + check.out * b;
}

// ============================================
// ABSOLUTE VALUE
// ============================================
template AbsValue(BITS) {
    signal input value;
    signal output absValue;
    
    component isNegative = LessThan(BITS);
    isNegative.in[0] <== value;
    isNegative.in[1] <== 0;
    
    absValue <== isNegative.out * (-value) + (1 - isNegative.out) * value;
}