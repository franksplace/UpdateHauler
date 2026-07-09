#!/bin/bash
# Pre-commit validation script for UpdateHauler
# Run this before committing to catch CI failures early
#
# Usage:
#   ./scripts/validate.sh          # Run all checks
#   ./scripts/validate.sh quick    # Run quick checks only (fmt + clippy)
#   ./scripts/validate.sh install  # Install as git pre-commit hook

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo ""
    echo -e "${BLUE}==========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}==========================================${NC}"
}

print_pass() {
    echo -e "${GREEN}✓ PASSED${NC}: $1"
}

print_fail() {
    echo -e "${RED}✗ FAILED${NC}: $1"
    FAILED=1
}

print_warn() {
    echo -e "${YELLOW}⚠ WARN${NC}: $1"
}

FAILED=0

# Install as git hook
install_hook() {
    HOOK_DIR=".git/hooks"
    HOOK_FILE="$HOOK_DIR/pre-commit"
    
    if [ ! -d ".git" ]; then
        echo -e "${RED}Error: Not a git repository. Run 'git init' first.${NC}"
        exit 1
    fi
    
    mkdir -p "$HOOK_DIR"
    
    cat > "$HOOK_FILE" << 'EOF'
#!/bin/bash
# Auto-generated pre-commit hook for UpdateHauler
echo "Running pre-commit validation..."
./scripts/validate.sh quick
if [ $? -ne 0 ]; then
    echo ""
    echo "Pre-commit checks failed. Please fix issues before committing."
    echo "To bypass: git commit --no-verify"
    exit 1
fi
EOF
    
    chmod +x "$HOOK_FILE"
    echo -e "${GREEN}✓ Pre-commit hook installed${NC}"
    echo "  Location: $HOOK_FILE"
    echo "  To bypass: git commit --no-verify"
    exit 0
}

# Handle install command
if [ "$1" = "install" ]; then
    install_hook
fi

echo -e "${BLUE}UpdateHauler Pre-Commit Validation${NC}"
echo ""

# Determine mode
MODE="${1:-full}"

# 1. Format check
print_header "1. Checking formatting (cargo fmt)"
if cargo fmt -- --check 2>/dev/null; then
    print_pass "Code formatting"
else
    print_fail "Code formatting - run 'cargo fmt' to fix"
fi

# 2. Clippy lints
print_header "2. Running Clippy lints"
if cargo clippy --all-targets -- -D warnings 2>/dev/null; then
    print_pass "Clippy lints"
else
    print_fail "Clippy lints found - run 'cargo clippy --all-targets' to see issues"
fi

# Skip remaining checks in quick mode
if [ "$MODE" = "quick" ]; then
    echo ""
    if [ $FAILED -eq 0 ]; then
        echo -e "${GREEN}All quick checks passed!${NC}"
    else
        echo -e "${RED}Some checks failed. Please fix before committing.${NC}"
        exit 1
    fi
    exit 0
fi

# 3. Unit tests
print_header "3. Running unit tests"
if cargo test --quiet 2>/dev/null; then
    print_pass "Unit tests"
else
    print_fail "Unit tests failed"
fi

# 4. Integration tests
print_header "4. Running integration tests"
if cargo test --test integration_test --quiet 2>/dev/null; then
    print_pass "Integration tests"
else
    print_fail "Integration tests failed"
fi

# 5. Build check (release)
print_header "5. Checking release build"
if cargo build --release --quiet 2>/dev/null; then
    print_pass "Release build"
else
    print_fail "Release build failed"
fi

# 6. Version consistency check
print_header "6. Checking version consistency"
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
TEST_VERSION=$(grep -o '0\.[0-9]*\.[0-9]*' test_release.sh | head -1)
if [ "$CARGO_VERSION" = "$TEST_VERSION" ]; then
    print_pass "Version consistency (Cargo.toml: $CARGO_VERSION, test_release.sh: $TEST_VERSION)"
else
    print_fail "Version mismatch: Cargo.toml has '$CARGO_VERSION' but test_release.sh expects '$TEST_VERSION'"
    echo "  Fix: Update the version in test_release.sh to match Cargo.toml"
fi

# Summary
echo ""
echo -e "${BLUE}==========================================${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All checks passed! Ready to commit.${NC}"
else
    echo -e "${RED}Some checks failed. Please fix before committing.${NC}"
    echo "  To bypass: git commit --no-verify"
    exit 1
fi
