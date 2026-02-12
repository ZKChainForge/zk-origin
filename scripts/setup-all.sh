#!/bin/bash
set -e

echo " ZK-ORIGIN Complete Setup"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check Node.js
if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js is not installed${NC}"
    echo "Install from: https://nodejs.org/"
    exit 1
fi

# Check Circom
if ! command -v circom &> /dev/null; then
    echo -e "${YELLOW}Warning: Circom is not installed${NC}"
    echo "Installing circom..."
    npm install -g circom
fi

echo -e "${BLUE}Step 1: Installing root dependencies...${NC}"
npm install

echo -e "\n${BLUE}Step 2: Setting up circuits...${NC}"
cd circuits
npm install

echo -e "\n${BLUE}Step 3: Compiling circuits...${NC}"
chmod +x scripts/*.sh
./scripts/compile.sh lineage_step_simple

echo -e "\n${BLUE}Step 4: Running trusted setup...${NC}"
./scripts/setup.sh lineage_step_simple

echo -e "\n${BLUE}Step 5: Running circuit tests...${NC}"
npm test || echo -e "${YELLOW}Some tests may have failed, continuing...${NC}"

cd ..

echo -e "\n${BLUE}Step 6: Setting up contracts...${NC}"
cd contracts
npm install

echo -e "\n${BLUE}Step 7: Compiling contracts...${NC}"
npx hardhat compile

cd ..

echo -e "\n${BLUE}Step 8: Setting up demo...${NC}"
cd demo
npm install

cd ..

echo -e "\n${GREEN} Setup complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Run tests:        npm test"
echo "  2. Run demo:         npm run demo"
echo "  3. Deploy testnet:   npm run deploy:testnet"
echo ""