#!/bin/bash
set -e

echo "ZK-ORIGIN Test Suite"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

FAILED=0

# Circuit tests
echo " Running circuit tests..."
if npm test; then
    echo -e "${GREEN} Circuit tests passed${NC}\n"
else
    echo -e "${RED} Circuit tests failed${NC}\n"
    FAILED=1
fi
cd ..

# Contract tests
echo " Running contract tests..."
cd contracts
if npm test; then
    echo -e "${GREEN} Contract tests passed${NC}\n"
else
    echo -e "${RED} Contract tests failed${NC}\n"
    FAILED=1
fi
cd ..

# Summary
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN} All tests passed!${NC}"
else
    echo -e "${RED} Some tests failed${NC}"
    exit 1
fi