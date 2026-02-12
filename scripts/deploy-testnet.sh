#!/bin/bash
set -e

echo "ZK-ORIGIN Testnet Deployment"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check .env
if [ ! -f "contracts/.env" ]; then
    echo -e "${YELLOW}Warning: contracts/.env not found${NC}"
    echo "Copying from .env.example..."
    cp contracts/.env.example contracts/.env
    echo -e "${YELLOW}Please edit contracts/.env with your credentials${NC}"
    exit 1
fi

# Deploy
echo "Deploying to Sepolia..."
cd contracts
npx hardhat run scripts/deploy.js --network sepolia

echo ""
echo -e "${GREEN} Deployment complete!${NC}"
echo ""
echo "Check deployments/sepolia/addresses.json for contract addresses"