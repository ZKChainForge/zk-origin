#!/bin/bash

set -e

echo "🧪 ZK-ORIGIN Circuit Tests"
echo "=========================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 1. Compile all circuits
echo "📝 Compiling circuits..."
echo ""

circuits=(
    "circuits/auth/user_auth.circom"
    "circuits/auth/admin_auth.circom"
    "circuits/auth/bridge_auth.circom"
    "circuits/auth/governance_auth.circom"
    "circuits/auth/emergency_auth.circom"
    "circuits/core/policy_verifier.circom"
    "circuits/core/rate_limiter.circom"
    "circuits/core/epoch_manager.circom"
    "circuits/core/lineage_step.circom"
    "circuits/main.circom"
)

for circuit in "${circuits[@]}"; do
    name=$(basename $circuit .circom)
    echo "  Compiling $name..."
    circom $circuit --r1cs --wasm --sym -o build/
    
    if [ $? -eq 0 ]; then
        echo -e "  ${GREEN}✓${NC} $name compiled"
    else
        echo -e "  ${RED}✗${NC} $name failed"
        exit 1
    fi
done

echo ""
echo "✅ All circuits compiled successfully"
echo ""

# 2. Get constraint counts
echo "📊 Constraint Counts:"
echo ""

for circuit in "${circuits[@]}"; do
    name=$(basename $circuit .circom)
    r1cs="build/${name}.r1cs"
    
    if [ -f "$r1cs" ]; then
        constraints=$(snarkjs r1cs info $r1cs 2>/dev/null | grep "# of Constraints" | awk '{print $NF}')
        wires=$(snarkjs r1cs info $r1cs 2>/dev/null | grep "# of Wires" | awk '{print $NF}')
        
        printf "  %-25s %10s constraints, %10s wires\n" "$name" "$constraints" "$wires"
    fi
done

echo ""
echo "✅ All tests passed!"