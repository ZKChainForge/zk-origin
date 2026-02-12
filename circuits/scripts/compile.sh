#!/bin/bash
set -e

echo " ZK-ORIGIN Circuit Compiler"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if circom is installed
if ! command -v circom &> /dev/null; then
    echo -e "${RED}Error: circom is not installed${NC}"
    echo "Install with: npm install -g circom"
    exit 1
fi

# Create build directory
mkdir -p build

# Determine which circuit to compile
CIRCUIT=${1:-"lineage_step_simple"}
CIRCUIT_PATH="src/main/${CIRCUIT}.circom"

if [ ! -f "$CIRCUIT_PATH" ]; then
    echo -e "${RED}Error: Circuit not found: $CIRCUIT_PATH${NC}"
    exit 1
fi

echo -e "${YELLOW} Compiling: $CIRCUIT_PATH${NC}"
echo ""

# Compile the circuit
circom "$CIRCUIT_PATH" \
    --r1cs \
    --wasm \
    --sym \
    --output build \
    -l node_modules

echo ""
echo -e "${GREEN} Compilation successful!${NC}"
echo ""

# Print circuit statistics
echo " Circuit Statistics:"

if command -v snarkjs &> /dev/null; then
    snarkjs r1cs info "build/${CIRCUIT}.r1cs"
else
    echo "Install snarkjs to see detailed stats: npm install -g snarkjs"
fi

echo ""
echo "Output files:"
ls -lh build/

echo ""
echo -e "${GREEN}Done! Next step: run ./scripts/setup.sh${NC}"