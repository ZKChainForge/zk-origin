pragma circom 2.1.0;

include "../lib/comparators.circom";

/**
 * @title System Authentication (PRODUCTION)
 * @notice Verifies system-level operations are from authorized address
 * 
 * SECURITY:
 * Requires exact address match
 *  No partial matching
 *  Circuit fails if address mismatch
 * 
 * PROTECTION: SYSTEM PROTECTED
 * - Restricts system operations to authorized caller
 * - Prevents unauthorized system state changes
 * - One address per system
 * 
 * INPUT AUTHORIZATION:
 * - callerAddress: Address attempting system call
 * - expectedSystemAddress: Authorized system address
 * 
 * OUTPUT GUARANTEE:
 * - valid: 1 if addresses match exactly, circuit fails if not
 * 
 * CONSTRAINTS: ~200 (address equality)
 * 
 * PRODUCTION CHECKLIST:
 *  Addresses must match exactly
 *  No partial validation
 *  Circuit fails if mismatch
 *  Address not constrained to valid range
 * (Ethereum addresses are 20 bytes, verified off-chain)
 * 
 * ATTACK VECTORS MITIGATED:
 *  Wrong caller: Address check prevents
 *  Address reuse: Contract prevents via storage
 * 
 * NOTES:
 * - Most minimal auth circuit
 * - Addresses are provided by off-chain code
 * - Contract must validate caller address matches msg.sender
 * - Can be extended with role-based access control
 */

template SystemAuth() {
    // ============ PUBLIC INPUTS ============
    signal input callerAddress;          // System caller address
    signal input expectedSystemAddress;  // Authorized system address
    
    // ============ PUBLIC OUTPUTS ============
    signal output valid;  // 1 if authorized
    
    // ============ VERIFY ADDRESS MATCHES ============
    component addrMatch = ZKIsEqual();
    addrMatch.in[0] <== callerAddress;
    addrMatch.in[1] <== expectedSystemAddress;
    
    // ENFORCE: Addresses must match exactly
    addrMatch.out === 1;
    
    valid <== 1;
}

component main {public [callerAddress, expectedSystemAddress]} = SystemAuth();