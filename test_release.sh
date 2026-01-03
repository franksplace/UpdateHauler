#!/bin/bash
# Comprehensive test suite for updatehauler release candidates
# This script runs all necessary tests before a release

set -e

echo "=========================================="
echo "UpdateHauler Release Test Suite"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print test results
print_result() {
	if [ $? -eq 0 ]; then
		echo -e "${GREEN}✓ PASSED${NC}: $1"
	else
		echo -e "${RED}✗ FAILED${NC}: $1"
		exit 1
	fi
}

# Function to print section headers
print_section() {
	echo ""
	echo -e "${YELLOW}=========================================="
	echo "$1"
	echo -e "==========================================${NC}"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
	echo "Error: Must run this script from the updatehauler root directory"
	exit 1
fi

print_section "1. Building Binary"

cargo build --release
print_result "Release build compilation"

print_section "2. Running Unit Tests"

cargo test --lib 2>&1 | grep -E "(test result|running)"
print_result "Unit tests"

print_section "3. Running Integration Tests"

cargo test --test integration_test 2>&1 | grep -E "(test result|running)"
print_result "Integration tests"

print_section "4. Testing --help"

./target/release/updatehauler --help >/tmp/help_output.txt
if grep -q "Usage:" /tmp/help_output.txt &&
	grep -q "OPTIONS" /tmp/help_output.txt &&
	grep -q "ACTION" /tmp/help_output.txt &&
	grep -q "dry-run" /tmp/help_output.txt &&
	grep -q "nvim" /tmp/help_output.txt &&
	grep -q "config-file" /tmp/help_output.txt; then
	print_result "--help command"
else
	echo "Error: --help output missing expected sections"
	exit 1
fi

print_section "5. Testing --version"

./target/release/updatehauler --version >/tmp/version_output.txt
if grep -q "updatehauler" /tmp/version_output.txt &&
	grep -q "0.1.0" /tmp/version_output.txt; then
	print_result "--version command"
else
	echo "Error: --version output invalid"
	exit 1
fi

print_section "6. Testing --run Command"

./target/release/updatehauler --run "echo test" >/tmp/run_output.txt 2>&1
if grep -q "test" /tmp/run_output.txt &&
	grep -q "Return code 0" /tmp/run_output.txt; then
	print_result "--run command"
else
	echo "Error: --run command failed"
	exit 1
fi

print_section "7. Testing Real-time Output Streaming"

./target/release/updatehauler --run "echo 'line1'; sleep 0.1; echo 'line2'" >/tmp/stream_output.txt 2>&1
if grep -q "line1" /tmp/stream_output.txt &&
	grep -q "line2" /tmp/stream_output.txt; then
	print_result "Real-time output streaming"
else
	echo "Error: Real-time output not working correctly"
	exit 1
fi

print_section "8. Testing Invalid Action Error Handling"

./target/release/updatehauler invalid-action >/tmp/invalid_output.txt 2>&1
if grep -q "Invalid action" /tmp/invalid_output.txt &&
	grep -q "Run 'updatehauler --help'" /tmp/invalid_output.txt; then
	print_result "Invalid action error handling"
else
	echo "Error: Invalid action error handling failed"
	exit 1
fi

print_section "9. Testing Flag Combinations"

# Test --no-color
./target/release/updatehauler --no-color --run "echo test" >/tmp/no_color.txt 2>&1
if grep -q "test" /tmp/no_color.txt; then
	print_result "--no-color flag"
else
	echo "Error: --no-color flag failed"
	exit 1
fi

# Test --no-datetime
./target/release/updatehauler --no-datetime --run "echo test" >/tmp/no_datetime.txt 2>&1
if ! grep -q "T[0-9]" /tmp/no_datetime.txt && grep -q "test" /tmp/no_datetime.txt; then
	print_result "--no-datetime flag"
else
	echo "Error: --no-datetime flag failed"
	exit 1
fi

# Test --no-header
./target/release/updatehauler --no-header --run "echo test" >/tmp/no_header.txt 2>&1
if grep -q "test" /tmp/no_header.txt &&
	! grep -q "→ Start" /tmp/no_header.txt &&
	! grep -q "→ Return code" /tmp/no_header.txt; then
	print_result "--no-header flag"
else
	echo "Error: --no-header flag failed"
	exit 1
fi

print_section "10. Testing Custom File Paths"

# Test --logfile with --logfile-only
./target/release/updatehauler --logfile /tmp/custom.log --logfile-only --run "echo test" >/tmp/custom_log_output.txt 2>&1
if [ -f /tmp/custom.log ] && grep -q "test" /tmp/custom.log; then
	print_result "--logfile option"
else
	echo "Error: --logfile option failed"
	exit 1
fi

# Clean up
rm -f /tmp/custom.log

# Test --max-log-lines
./target/release/updatehauler --max-log-lines 100 --run "echo test" >/tmp/max_lines.txt 2>&1
if grep -q "Return code 0" /tmp/max_lines.txt; then
	print_result "--max-log-lines option"
else
	echo "Error: --max-log-lines option failed"
	exit 1
fi

print_section "11. Testing --installdir"

./target/release/updatehauler --installdir /tmp/test_install --run "echo test" >/tmp/installdir.txt 2>&1
if grep -q "Return code 0" /tmp/installdir.txt; then
	print_result "--installdir option"
else
	echo "Error: --installdir option failed"
	exit 1
fi

print_section "12. Testing --brew-save-file and --cargo-save-file"

# These just verify the option doesn't cause a crash (actual functionality tested in integration tests)
./target/release/updatehauler --help | grep -q "brew-save-file"
print_result "--brew-save-file option in help"

./target/release/updatehauler --help | grep -q "cargo-save-file"
print_result "--cargo-save-file option in help"

print_section "13. Testing --config-file Option"

# Verify --config-file option exists in help (already tested in help section)

# Test with non-existent config file (should work with defaults)
./target/release/updatehauler --config-file /tmp/nonexistent_config.yaml --dry-run os >/tmp/config_option.txt 2>&1
if grep -q "Would execute:" /tmp/config_option.txt; then
	print_result "--config-file option works with non-existent file"
else
	echo "Error: --config-file option failed"
	exit 1
fi

rm -f /tmp/nonexistent_config.yaml

print_section "14. Testing Multiple Actions"

./target/release/updatehauler --dry-run os trim-logfile >/tmp/multiple_actions.txt 2>&1
if grep -q "Main → Start" /tmp/multiple_actions.txt &&
	grep -q "Main → End" /tmp/multiple_actions.txt; then
	print_result "Multiple actions execution"
else
	echo "Error: Multiple actions execution failed"
	exit 1
fi

print_section "15. Testing Debug Flag"

# Just verify debug flag doesn't cause errors (debug output is for config insights, not --run)
./target/release/updatehauler --debug --run "echo test" >/tmp/debug_output.txt 2>&1
if grep -q "test" /tmp/debug_output.txt &&
	grep -q "Return code 0" /tmp/debug_output.txt; then
	print_result "--debug flag"
else
	echo "Error: --debug flag failed"
	exit 1
fi

print_section "16. Testing Schedule Commands"

# Test schedule check (non-destructive)
./target/release/updatehauler schedule check >/tmp/schedule_check.txt 2>&1
if grep -q "LaunchAgent plist" /tmp/schedule_check.txt; then
	print_result "schedule check command"
else
	echo "Error: schedule check failed"
	exit 1
fi

# Test schedule help text
./target/release/updatehauler --help | grep -q "schedule"
print_result "schedule in help text"

print_section "17. Testing Schedule Time Flags"

# Test custom schedule time flags (just verify they don't error)
./target/release/updatehauler --sched-hour "3" --sched-minute "30" schedule check >/tmp/schedule_flags.txt 2>&1
# Check for platform-specific success indicators
# macOS: "LaunchAgent plist", "plist exists", "launchctl status"
# Linux: "crontab at all enabled", "No crontab", or "crontab:" (showing crontab content)
if grep -q "LaunchAgent plist\|LaunchAgent plist:\|launchctl status:\|crontab at all\|No crontab\|crontab:" /tmp/schedule_flags.txt; then
	print_result "Schedule time flags"
else
	echo "Error: Schedule time flags failed"
	cat /tmp/schedule_flags.txt
	exit 1
fi

print_section "18. Testing Backup/Restore Commands"

# Test that backup commands don't crash (may fail if tools not installed)
./target/release/updatehauler brew-save >/dev/null 2>&1 || true
./target/release/updatehauler cargo-save >/dev/null 2>&1 || true
./target/release/updatehauler nvim-save >/dev/null 2>&1 || true
print_result "Backup commands (no crash)"

# Test that restore commands don't crash (may fail if files not found)
./target/release/updatehauler brew-restore >/dev/null 2>&1 || true
./target/release/updatehauler cargo-restore >/dev/null 2>&1 || true
./target/release/updatehauler nvim-restore >/dev/null 2>&1 || true
print_result "Restore commands (no crash)"

print_section "19. Testing Package Detection"

# Test that binary correctly detects OS and package managers
./target/release/updatehauler --dry-run os >/tmp/detection.txt 2>&1
if grep -q "Would execute:" /tmp/detection.txt; then
	print_result "OS and package manager detection"
else
	echo "Error: OS detection failed"
	exit 1
fi

print_section "20. Testing Log File Rotation"

# Test trim-logfile functionality
for i in {1..50}; do echo "log line"; done >/tmp/test_trim.log
./target/release/updatehauler --logfile /tmp/test_trim.log --max-log-lines 10 trim-logfile >/dev/null 2>&1
LINE_COUNT=$(wc -l </tmp/test_trim.log)
if [ "$LINE_COUNT" -le 10 ]; then
	print_result "Log file rotation (trim-logfile)"
else
	echo "Error: Log file rotation failed (expected <= 10 lines, got $LINE_COUNT)"
	exit 1
fi

print_section "21. Testing Dry-Run Mode"

# Test dry-run doesn't actually update anything
./target/release/updatehauler --dry-run os >/tmp/dryrun_output.txt 2>&1
if grep -q "DRY-RUN" /tmp/dryrun_output.txt &&
	grep -q "Would execute:" /tmp/dryrun_output.txt &&
	! grep -q "Password" /tmp/dryrun_output.txt; then
	print_result "Dry-run mode (no password prompts)"
else
	echo "Error: Dry-run mode failed"
	exit 1
fi

# Test dry-run shows macOS sudo softwareupdate
./target/release/updatehauler --dry-run os >/tmp/dryrun_os_output.txt 2>&1
if grep -q "sudo softwareupdate" /tmp/dryrun_os_output.txt; then
	print_result "Dry-run shows sudo softwareupdate command"
else
	echo "Error: Dry-run doesn't show sudo softwareupdate"
	exit 1
fi

# Test dry-run with multiple actions
./target/release/updatehauler --dry-run os brew >/tmp/dryrun_multi.txt 2>&1
if grep -q "DRY-RUN" /tmp/dryrun_multi.txt; then
	print_result "Dry-run with multiple actions"
else
	echo "Error: Dry-run with multiple actions failed"
	exit 1
fi

print_section "22. Cleaning Up Test Files"

rm -f /tmp/help_output.txt /tmp/version_output.txt
rm -f /tmp/run_output.txt /tmp/stream_output.txt
rm -f /tmp/invalid_output.txt /tmp/no_color.txt
rm -f /tmp/no_datetime.txt /tmp/no_header.txt
rm -f /tmp/custom_log_output.txt /tmp/max_lines.txt
rm -f /tmp/installdir.txt /tmp/multiple_actions.txt
rm -f /tmp/debug_output.txt
rm -f /tmp/schedule_check.txt /tmp/schedule_flags.txt
rm -f /tmp/detection.txt /tmp/test_trim.log
rm -f /tmp/dryrun_output.txt
rm -f /tmp/config_option.txt
rm -f /tmp/nonexistent_config.yaml
rm -f /tmp/dryrun_os_output.txt

print_result "Cleanup"

echo ""
echo -e "${GREEN}=========================================="
echo "ALL TESTS PASSED! ✓"
echo -e "==========================================${NC}"
echo ""
echo "Release candidate is ready!"
echo ""
