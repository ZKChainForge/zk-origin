#!/bin/bash
set -e

echo " ZK-ORIGIN Trusted Setup"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check dependencies
if ! command -v snarkjs &> /dev/null; then
    echo -e "${RED}Error: snarkjs is not installed${NC}"
    echo "Install with: npm install -g snarkjs"
    exit 1
fi

CIRCUIT=${1:-"lineage_step_simple"}
R1CS_FILE="build/${CIRCUIT}.r1cs"

if [ ! -f "$R1CS_FILE" ]; then
    echo -e "${RED}Error: R1CS file not found: $R1CS_FILE${NC}"
    echo "Run ./scripts/compile.sh first"
    exit 1
fi

# Download Powers of Tau if not present
PTAU_FILE="pot14_final.ptau"
if [ ! -f "$PTAU_FILE" ]; then
    echo -e "${YELLOW} Downloading Powers of Tau...${NC}"
    echo "This may take a few minutes..."
    wget -q --show-progress https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_14.ptau -O $PTAU_FILE
    echo ""
fi

echo -e "${YELLOW} Generating proving key...${NC}"
snarkjs groth16 setup "$R1CS_FILE" "$PTAU_FILE" "build/${CIRCUIT}_0.zkey"

echo ""
echo -e "${YELLOW} Adding contribution (entropy)...${NC}"
# Generate random entropy
ENTROPY=$(head -c 64 /dev/urandom | xxd -p -c 256)
echo "$ENTROPY" | snarkjs zkey contribute "build/${CIRCUIT}_0.zkey" "build/${CIRCUIT}.zkey" \
    --name="ZK-ORIGIN Setup" -v

# Clean up intermediate file
rm -f "build/${CIRCUIT}_0.zkey"

echo ""
echo -e "${YELLOW} Exporting verification key...${NC}"
snarkjs zkey export verificationkey "build/${CIRCUIT}.zkey" "build/verification_key.json"

echo ""
echo -e "${YELLOW} Generating Solidity verifier...${NC}"
mkdir -p ../contracts/contracts
snarkjs zkey export solidityverifier "build/${CIRCUIT}.zkey" "../contracts/contracts/Groth16Verifier.sol"

echo ""
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo " Generated files:"
echo "   - build/${CIRCUIT}.zkey (proving key)"
echo "   - build/verification_key.json (verification key)"
echo "   - ../contracts/contracts/Groth16Verifier.sol (Solidity verifier)"
echo ""
echo -e "${GREEN}Next step: run 'npm test' to verify the circuit${NC}"