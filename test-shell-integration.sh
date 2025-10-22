#!/bin/bash
# Test script for VIP shell integration
# Tests bash, zsh, and fish integration

# Don't exit on error - we handle errors manually
set +e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PASSED=0
FAILED=0

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🐍 VIP Shell Integration Test Suite${NC}"
echo "========================================"
echo ""

# Test function
test_shell() {
    local shell_name=$1
    local shell_cmd=$2
    local integration_file=$3
    local test_cmd=$4
    
    echo -e "${BLUE}Testing $shell_name...${NC}"
    
    if ! command -v "$shell_cmd" &> /dev/null; then
        echo -e "${YELLOW}⊘ $shell_name not installed - skipping${NC}"
        echo ""
        return
    fi
    
    # Check if integration file exists
    if [ ! -f "$SCRIPT_DIR/$integration_file" ]; then
        echo -e "${RED}✗ Integration file missing: $integration_file${NC}"
        ((FAILED++))
        echo ""
        return
    fi
    echo -e "${GREEN}✓ Integration file exists${NC}"
    
    # Check if executable
    if [ -x "$SCRIPT_DIR/$integration_file" ]; then
        echo -e "${GREEN}✓ Integration file is executable${NC}"
    else
        echo -e "${RED}✗ Integration file not executable${NC}"
        ((FAILED++))
    fi
    
    # Try to source (syntax check)
    if eval "$test_cmd" &> /dev/null; then
        echo -e "${GREEN}✓ Integration file sources without errors${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ Integration file has syntax errors${NC}"
        ((FAILED++))
    fi
    
    echo ""
}

# Test Bash
test_shell "Bash" "bash" "vip-integration.sh" \
    "bash -c 'source $SCRIPT_DIR/vip-integration.sh 2>&1 | grep -q \"VIP\"'"

# Test Zsh
test_shell "Zsh" "zsh" "vip-integration.zsh" \
    "zsh -c 'source $SCRIPT_DIR/vip-integration.zsh 2>&1 | grep -q \"VIP\"'"

# Test Fish
test_shell "Fish" "fish" "vip-integration.fish" \
    "fish -c 'source $SCRIPT_DIR/vip-integration.fish 2>&1 | grep -q \"VIP\"'"

# Check VIP executable
echo -e "${BLUE}Testing VIP executable...${NC}"
if [ -f "$SCRIPT_DIR/vip" ]; then
    if [ -x "$SCRIPT_DIR/vip" ]; then
        echo -e "${GREEN}✓ vip executable exists and is executable${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ vip exists but is not executable${NC}"
        ((FAILED++))
    fi
else
    echo -e "${RED}✗ vip executable not found${NC}"
    ((FAILED++))
fi
echo ""

# Check Python dependencies
echo -e "${BLUE}Testing Python dependencies...${NC}"
if command -v python3 &> /dev/null; then
    echo -e "${GREEN}✓ Python3 installed${NC}"
    ((PASSED++))
    
    if python3 -c "import pygame" &> /dev/null; then
        echo -e "${GREEN}✓ Pygame installed${NC}"
        ((PASSED++))
    else
        echo -e "${YELLOW}⚠ Pygame not installed (visual mode will be disabled)${NC}"
    fi
else
    echo -e "${RED}✗ Python3 not found${NC}"
    ((FAILED++))
fi
echo ""

# Summary
echo "========================================"
echo -e "${BLUE}Test Summary${NC}"
echo "========================================"
echo -e "${GREEN}Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED${NC}"
else
    echo -e "${GREEN}Failed: $FAILED${NC}"
fi
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "To enable VIP in your shell:"
    echo ""
    echo "  Bash:  source $SCRIPT_DIR/vip-integration.sh"
    echo "  Zsh:   source $SCRIPT_DIR/vip-integration.zsh"
    echo "  Fish:  source $SCRIPT_DIR/vip-integration.fish"
    echo ""
    echo "Or run the installer:"
    echo "  $SCRIPT_DIR/install-visual.sh"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    echo ""
    echo "Please check the errors above and:"
    echo "  1. Ensure all integration files exist"
    echo "  2. Make them executable: chmod +x vip-integration.*"
    echo "  3. Install Python3 and pygame if missing"
    exit 1
fi
