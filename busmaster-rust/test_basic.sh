#!/bin/bash
# Basic test script for BUSMASTER CLI and TUI

set -e  # Exit on error

echo "========================================="
echo "BUSMASTER Basic Test Script"
echo "========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local command="$2"
    
    echo -n "Testing: $test_name... "
    
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

echo "Building applications..."
~/.cargo/bin/cargo build --package busmaster-cli --package busmaster-tui --quiet
echo -e "${GREEN}✓ Build successful${NC}"
echo ""

echo "========================================="
echo "CLI Tests"
echo "========================================="

run_test "CLI help" "~/.cargo/bin/cargo run --quiet --bin busmaster -- --help"
run_test "CLI version" "~/.cargo/bin/cargo run --quiet --bin busmaster -- --version"
run_test "CLI list" "~/.cargo/bin/cargo run --quiet --bin busmaster -- list"
run_test "CLI send" "~/.cargo/bin/cargo run --quiet --bin busmaster -- send --id 0x123 --data '01 02 03 04'"

echo ""
echo "========================================="
echo "Integration Tests"
echo "========================================="

run_test "CLI integration tests" "~/.cargo/bin/cargo test --quiet --package busmaster-cli"

echo ""
echo "========================================="
echo "File Tests"
echo "========================================="

run_test "Sample DBC exists" "test -f examples/test.dbc"
run_test "CLI README exists" "test -f crates/busmaster-cli/README.md"
run_test "TUI README exists" "test -f crates/busmaster-tui/README.md"
run_test "CLI quickstart exists" "test -f QUICKSTART_CLI.md"
run_test "TUI quickstart exists" "test -f QUICKSTART_TUI.md"
run_test "Testing guide exists" "test -f TESTING_GUIDE.md"

echo ""
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Read TESTING_GUIDE.md for comprehensive testing"
    echo "2. Try the CLI: cargo run --bin busmaster -- --help"
    echo "3. Try the TUI: cargo run --package busmaster-tui"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
