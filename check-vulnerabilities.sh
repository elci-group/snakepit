#!/usr/bin/env bash
# Local vulnerability scanning script for snakepit

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}===========================================${NC}"
echo -e "${BLUE}  Snakepit Vulnerability Scanner${NC}"
echo -e "${BLUE}===========================================${NC}"
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check and install cargo-audit
if ! command_exists cargo-audit; then
    echo -e "${YELLOW}Installing cargo-audit...${NC}"
    cargo install cargo-audit
    echo ""
fi

# Check and install cargo-deny
if ! command_exists cargo-deny; then
    echo -e "${YELLOW}Installing cargo-deny...${NC}"
    cargo install cargo-deny
    echo ""
fi

# Check and install cargo-outdated
if ! command_exists cargo-outdated; then
    echo -e "${YELLOW}Installing cargo-outdated...${NC}"
    cargo install cargo-outdated
    echo ""
fi

# Update advisory database
echo -e "${BLUE}[1/5] Updating advisory database...${NC}"
cargo audit --update-only
echo -e "${GREEN}✓ Database updated${NC}"
echo ""

# Run cargo audit
echo -e "${BLUE}[2/5] Running security audit...${NC}"
if cargo audit; then
    echo -e "${GREEN}✓ No known vulnerabilities found${NC}"
else
    echo -e "${RED}✗ Vulnerabilities detected! Check output above.${NC}"
    echo -e "${YELLOW}  Run 'cargo update' to attempt fixes${NC}"
fi
echo ""

# Run cargo deny
echo -e "${BLUE}[3/5] Running cargo-deny checks...${NC}"
if cargo deny check; then
    echo -e "${GREEN}✓ All cargo-deny checks passed${NC}"
else
    echo -e "${RED}✗ Some cargo-deny checks failed${NC}"
fi
echo ""

# Check for outdated dependencies
echo -e "${BLUE}[4/5] Checking for outdated dependencies...${NC}"
cargo outdated --root-deps-only || echo -e "${YELLOW}Some dependencies may be outdated${NC}"
echo ""

# Generate detailed report
echo -e "${BLUE}[5/5] Generating detailed reports...${NC}"
REPORT_DIR="security-reports"
mkdir -p "$REPORT_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

cargo audit --json > "$REPORT_DIR/audit_${TIMESTAMP}.json" 2>/dev/null || true
cargo audit > "$REPORT_DIR/audit_${TIMESTAMP}.txt" 2>/dev/null || true
cargo outdated --format json > "$REPORT_DIR/outdated_${TIMESTAMP}.json" 2>/dev/null || true

echo -e "${GREEN}✓ Reports saved to $REPORT_DIR/${NC}"
echo ""

# Summary
echo -e "${BLUE}===========================================${NC}"
echo -e "${BLUE}  Scan Complete${NC}"
echo -e "${BLUE}===========================================${NC}"
echo ""
echo -e "Next steps:"
echo -e "  1. Review any vulnerabilities or issues above"
echo -e "  2. Run ${YELLOW}cargo update${NC} to update dependencies"
echo -e "  3. Check reports in ${YELLOW}$REPORT_DIR/${NC} for details"
echo -e "  4. Run ${YELLOW}cargo test${NC} after updates to verify"
echo ""
