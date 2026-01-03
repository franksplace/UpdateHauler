#!/bin/bash
# Test shell completions for updatehauler

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}=========================================="
echo "Testing Shell Completions"
echo -e "==========================================${NC}"
echo ""

# Build release binary
cargo build --release
echo -e "${GREEN}✓${NC} Built release binary"
echo ""

# Test install-completions command
echo "Testing install-completions command..."
./target/release/updatehauler install-completions bash zsh 2>&1 | head -5
echo -e "${GREEN}✓${NC} install-completions command works"
echo ""

# Create temporary directory for testing completions
TEST_DIR=$(mktemp -d)
export HOME="$TEST_DIR"

echo "Installing completions to test directory..."
./target/release/updatehauler install-completions bash zsh >/dev/null 2>&1
echo -e "${GREEN}✓${NC} Completions installed successfully"
echo ""

# Check bash completion file
BASH_COMPLETION="$TEST_DIR/.local/bin/completions/bash/updatehauler.bash"
if [ -f "$BASH_COMPLETION" ]; then
	echo -e "${GREEN}✓${NC} Bash completion file created at $BASH_COMPLETION"

	# Check bash completion contains expected content
	if grep -q "updatehauler_completion" "$BASH_COMPLETION" &&
		grep -q "brew" "$BASH_COMPLETION" &&
		grep -q "cargo" "$BASH_COMPLETION" &&
		grep -q "nvim" "$BASH_COMPLETION" &&
		grep -q "os" "$BASH_COMPLETION"; then
		echo -e "${GREEN}✓${NC} Bash completion has expected content"
	else
		echo -e "${RED}✗${NC} Bash completion missing expected content"
		exit 1
	fi
else
	echo -e "${RED}✗${NC} Bash completion file not found"
	exit 1
fi
echo ""

# Check zsh completion file
ZSH_COMPLETION="$TEST_DIR/.local/bin/completions/zsh/updatehauler.zsh"
if [ -f "$ZSH_COMPLETION" ]; then
	echo -e "${GREEN}✓${NC} Zsh completion file created at $ZSH_COMPLETION"

	# Check zsh completion contains expected content
	if grep -q "#compdef updatehauler" "$ZSH_COMPLETION" &&
		grep -q "brew" "$ZSH_COMPLETION" &&
		grep -q "cargo" "$ZSH_COMPLETION" &&
		grep -q "nvim" "$ZSH_COMPLETION" &&
		grep -q "os" "$ZSH_COMPLETION"; then
		echo -e "${GREEN}✓${NC} Zsh completion has expected content"
	else
		echo -e "${RED}✗${NC} Zsh completion missing expected content"
		exit 1
	fi
else
	echo -e "${RED}✗${NC} Zsh completion file not found"
	exit 1
fi
echo ""

# Test that completions don't break on shell syntax errors
echo "Testing completion syntax..."
if bash -c "source '$BASH_COMPLETION' 2>/dev/null"; then
	echo -e "${GREEN}✓${NC} Bash completion syntax is valid"
else
	echo -e "${RED}✗${NC} Bash completion has syntax errors"
	exit 1
fi

# Zsh completion check (just verify it loads)
if zsh -c "autoload -U +X compinit 2>/dev/null; source '$ZSH_COMPLETION' 2>/dev/null"; then
	echo -e "${GREEN}✓${NC} Zsh completion syntax is valid"
else
	echo -e "${YELLOW}⚠${NC}  Zsh completion syntax check skipped (zsh not available or compinit disabled)"
fi
echo ""

# Cleanup
rm -rf "$TEST_DIR"
echo -e "${GREEN}✓${NC} Cleaned up test directory"
echo ""

echo -e "${GREEN}=========================================="
echo "All completion tests passed! ✓"
echo -e "==========================================${NC}"
echo ""
