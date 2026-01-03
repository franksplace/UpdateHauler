# UpdateHauler

UpdateHauler is a command-line caretaker for your entire development stack that rounds up, updates, and tidies everything in one go: operating system packages (macOS, Linux), Homebrew formulae and casks, Cargo crates, and any other updatable tools you wire in.

## Features

### Plugin Framework

UpdateHauler now uses a modular plugin architecture:
- **Extensible** - Easy to add new package manager plugins
- **Async** - Built with async support for parallel operations
- **Configurable** - Plugin behavior controlled via YAML configuration

### Package Manager Plugins

**General Package Manager Plugins:**
- **Homebrew Plugin** - Update, upgrade, cleanup, and backup/restore brew formulas and casks
- **Cargo Plugin** - Update installed cargo packages and backup/restore cargo packages
- **Neovim Plugin** - Update Neovim plugins (supports lazy.nvim, packer.nvim, vim-plug)

**OS Package Manager Plugin:**
- **OS Plugin** - System updates for multiple platforms:
  - **macOS** - System updates via `softwareupdate` and Mac App Store updates via `mas`
    - Uses `sudo softwareupdate` first (for CI/CD environments)
    - Falls back to regular `softwareupdate` if sudo fails
    - Requires password unless using dry-run mode
  - **Debian/Ubuntu** - Updates via `apt-get`
  - **Red Hat/CentOS/Fedora/Rocky Linux/Oracle Linux** - Updates via `dnf`
  - **Alpine Linux** - Updates via `apk`
  - **Arch Linux** - Updates via `pacman`
  - **NixOS** - Updates via `nix-channel` and `nix-env`

### Key Capabilities

- **Plugin-Based Architecture** - Modular, extensible system for adding new package managers
- **Automated Updates** - Update OS packages, Homebrew, Cargo, and Neovim plugins in a single command
- **YAML Configuration** - Optional configuration file for fine-grained control over behavior
- **Backup & Restore** - Save and restore package configurations for brew, cargo, and nvim
- **Scheduling** - Set up automated updates via cron (Linux) or launchd (macOS)
- **Dry-Run Mode** - Preview changes without actually executing (perfect for CI/CD and testing)
- **Logging** - Comprehensive logging with optional rotation
- **Color Output** - Readable color-coded terminal output
- **Self-Installation** - Easy installation and updates of application itself

## Usage

```bash
updatehauler [OPTIONS] [ACTION]...
```

### Options

| Option | Description |
|--------|-------------|
| `--debug` | Enable debug output |
| `--no-debug` | Disable debug output (default) |
| `--datetime` | Enable ISO8601 timestamps with microseconds (default) |
| `--no-datetime` | Disable timestamps |
| `--header` | Enable header output for commands (default) |
| `--no-header` | Disable header output |
| `--color` | Enable color output (default) |
| `--no-color` | Disable color output |
| `--logfile-only` | Output only to logfile (no stdout) |
| `--dry-run` | Preview what would be done without making changes (no password prompts, perfect for CI/CD) |
| `--config <FILE>` | YAML configuration file path (default: `~/.config/updatehauler/config.yaml`) |
| `--logfile <FILE>` | Specify custom logfile location (default: `~/.local/updates.log`) |
| `--max-log-lines <N>` | Set maximum logfile lines for rotation (default: 10000) |
| `--installdir <PATH>` | Set installation directory (default: `~/.local/bin`) |
| `--brew-save-file <FILE>` | Specify custom brew save file location (default: `~/.config/brew/{os}-Brewfile`) |
| `--cargo-save-file <FILE>` | Specify custom cargo save file location (default: `~/.config/cargo/{os}-{arch}-cargo-backup.json`) |
| `--sched-minute <MIN>` | Set schedule minute (default: 0) |
| `--sched-hour <HOUR>` | Set schedule hour (default: 2) |
| `--sched-day-of-month <DAY>` | Set schedule day of month (default: *) |
| `--sched-month <MONTH>` | Set schedule month (default: *) |
| `--sched-day-of-week <DAY_OF_WEEK>` | Set schedule day of week (default: *) |
| `--run <CMD>...` | Run an arbitrary command with logging |
| `-h, --help` | Show help information |
| `-V, --version` | Print version information |

### Actions

#### Update Actions

| Action | Description |
|--------|-------------|
| `brew` | Update, upgrade, and cleanup brew formulas and casks |
| `cargo` | Update cargo-installed packages (requires `cargo-install-update`) |
| `nvim` | Update Neovim plugins (supports lazy.nvim, packer.nvim, vim-plug) |
| `os` | Update OS and app-based packages |

#### Backup/Restore Actions

| Action | Description |
|--------|-------------|
| `brew-save` | Save current brew installation to Brewfile |
| `brew-restore` | Restore brew installation from Brewfile |
| `cargo-save` | Save current cargo packages to backup JSON (requires `cargo-backup`) |
| `cargo-restore` | Restore cargo packages from backup JSON (requires `cargo-restore`) |
| `nvim-save` | Note nvim plugin configuration (plugins are defined in your nvim config) |
| `nvim-restore` | Restore nvim plugins using your configured plugin manager |

#### Scheduling Actions

| Action | Description |
|--------|-------------|
| `schedule enable` | Enable scheduled updates (cron on Linux, launchd on macOS) |
| `schedule disable` | Disable scheduled updates |
| `schedule check` | Check current scheduling status |

#### Maintenance Actions

| Action | Description |
|--------|-------------|
| `trim-logfile` | Trim logfile to maximum lines |

#### Self-Installation Actions

| Action | Description |
|--------|-------------|
| `install` | Install updatehauler to system |
| `update` | Update installed updatehauler binary |
| `remove` | Remove updatehauler from system |

### Default Behavior

When run without actions, updatehauler automatically (controlled by YAML config):
1. Updates OS packages (if enabled)
2. Updates Homebrew (if installed and enabled)
3. Saves brew configuration (if enabled)
4. Updates Cargo packages (if installed and enabled)
5. Saves cargo configuration (if enabled)
6. Updates Neovim plugins (if enabled)
7. Trims logfile

### Error Handling

If you provide an invalid action, updatehauler will display an error message with a suggestion to run `--help` for available actions.

## Examples

### Update everything
```bash
updatehauler
```

### Update only OS packages
```bash
updatehauler os
```

### Update brew and save configuration
```bash
updatehauler brew brew-save
```

### Save to custom backup file locations
```bash
updatehauler --brew-save-file "/custom/path/Brewfile" brew-save
updatehauler --cargo-save-file "/custom/path/cargo-backup.json" cargo-save
```

### Run with debug output
```bash
updatehauler --debug
```

### Dry-run mode - preview changes
```bash
# See what would be updated without actually updating
updatehauler --dry-run

# Dry-run specific actions
updatehauler --dry-run os
updatehauler --dry-run brew cargo

# Dry-run with custom schedule time
updatehauler --dry-run --sched-hour "3" --sched-minute "30" schedule enable
```

### Schedule daily updates at 2 AM
```bash
updatehauler schedule enable
```

### Schedule updates for specific time
```bash
# Schedule for 3:30 PM
updatehauler --sched-hour "15" --sched-minute "30" schedule enable

# Schedule for 1st day of month at 2 AM
updatehauler --sched-day-of-month "1" schedule enable

# Schedule for Monday, Wednesday, Friday at 10 AM
updatehauler --sched-day-of-week "MWF" --sched-hour "10" schedule enable
```

### Check scheduling status
```bash
updatehauler schedule check
```

### Restore from backup
```bash
updatehauler brew-restore
updatehauler cargo-restore
```

### Install to system
```bash
updatehauler install
```

### Run arbitrary command with logging
```bash
updatehauler --run echo "Hello World"
```

### Dry-run mode
```bash
# See what would be updated without actually updating
updatehauler --dry-run

# Dry-run specific actions
updatehauler --dry-run os brew
```

## Dependencies

### Required
- **macOS**: None (uses built-in tools)
- **Linux**: Appropriate package manager for your distribution

### Optional (for enhanced functionality)
- **cargo-install-update** - For updating cargo packages (`cargo install cargo-install-update`)
- **cargo-backup** - For backing up cargo packages (`cargo install cargo-backup`)
- **cargo-restore** - For restoring cargo packages (`cargo install cargo-restore`)
- **mas** (macOS only) - For Mac App Store updates (`brew install mas`)
- **brew cu** (macOS only) - For updating casks (`brew tap buo/cask-upgrade && brew install buo/cask-upgrade/brew-cu-completion`)

## Configuration

### Default Locations
- **Installation directory**: `~/.local/bin`
- **Log file**: `~/.local/updates.log`
- **Brew backup**: `~/.config/brew/{OS}-Brewfile`
- **Cargo backup**: `~/.config/cargo/{OS}-{ARCH}-cargo-backup.json`

### Custom Configuration

#### Command-Line Options
All default locations can be overridden using command-line options:
- `--logfile` to specify a custom log file location
- `--brew-save-file` to specify a custom brew backup file
- `--cargo-save-file` to specify a custom cargo backup file
- `--installdir` to specify a custom installation directory

#### YAML Configuration File

UpdateHauler supports an optional YAML configuration file for more advanced customization:

**Default location:** `~/.config/updatehauler/config.yaml`

**Custom location:** Use `--config` flag to specify a custom path

Example configuration:
```yaml
# Debug and output options
debug: false
datetime: true
show_header: true
color: true

# Logging options
use_log: false
dry_run: false
max_log_lines: 10000
# logfile: ~/.local/updates.log

# Installation and paths
# installdir: ~/.local/bin

# Custom save file locations
# brew_save_file: ~/.config/brew/Darwin-Brewfile
# cargo_save_file: ~/.config/cargo/Darwin-arm64-cargo-backup.json

# Schedule configuration
schedule:
  minute: "0"
  hour: "2"
  day_of_month: "*"
  month: "*"
  day_of_week: "*"

# Plugin configuration
plugins:
  brew: true
  cargo: true
  nvim: false
  os: true
```

**Available YAML Options:**

| Option | Type | Description |
|--------|------|-------------|
| `debug` | bool | Enable debug output |
| `datetime` | bool | Enable ISO8601 timestamps |
| `show_header` | bool | Enable header output |
| `color` | bool | Enable color output |
| `use_log` | bool | Enable logging to file |
| `dry_run` | bool | Enable dry-run mode |
| `max_log_lines` | number | Maximum log lines before rotation |
| `logfile` | string | Custom log file path |
| `installdir` | string | Installation directory |
| `brew_save_file` | string | Custom brew save file path |
| `cargo_save_file` | string | Custom cargo save file path |
| `schedule.minute` | string | Schedule minute (0-59) |
| `schedule.hour` | string | Schedule hour (0-23) |
| `schedule.day_of_month` | string | Schedule day of month (1-31 or *) |
| `schedule.month` | string | Schedule month (1-12 or *) |
| `schedule.day_of_week` | string | Schedule day of week (0-7 or *) |
| `plugins.brew` | bool | Enable/disable brew plugin |
| `plugins.cargo` | bool | Enable/disable cargo plugin |
| `plugins.nvim` | bool | Enable/disable nvim plugin |
| `plugins.os` | bool | Enable/disable OS plugin |

**Note:** Command-line options override YAML configuration settings.

### Scheduling Configuration

Default schedule (can be modified with command-line flags):
- **Minute**: 0
- **Hour**: 2 (2 AM)
- **Day of Month**: * (every day)
- **Month**: * (every month)
- **Day of Week**: * (every day)

#### macOS Scheduling Details
- Creates `~/Library/LaunchAgents/net.franksplace.wake-update-hauler.plist`
- Configures `launchd` to run updates at scheduled time
- Uses `pmset` to wake/power on the system at scheduled time
- Default: 2:00 AM daily
- Requires sudo privileges for `pmset` command
- Automatically wakes system to run updates via LaunchAgent
- **OS Updates**: Tries `sudo softwareupdate` first (for CI/CD environments), falls back to regular `softwareupdate` if sudo fails

#### Linux Scheduling Details
- Adds cron entry to user's crontab
- Runs updates at scheduled time
- Default: 2:00 AM daily (`0 2 * * *`)
- System must be awake at scheduled time

## Dry-Run Mode

UpdateHauler supports a `--dry-run` mode that previews what would happen without actually making changes. This is particularly useful for:

- **CI/CD Pipelines**: Test automation without triggering system updates or requiring authentication
- **Configuration Validation**: Verify your schedule and options work before enabling
- **Debugging**: See exactly which commands will be executed
- **Safety Audits**: Review update plans before running them

### How Dry-Run Works

When you use `--dry-run`:
- Commands are printed with `(DRY-RUN)` prefix
- Shows "Would execute: {command}" for each action
- Returns immediately without executing anything
- **No password prompts** - perfect for automated testing
- No network calls to package repositories
- No system state changes
- No actual package installations or updates

### Dry-Run Examples

```bash
# Preview all default updates
updatehauler --dry-run

# Preview OS updates only
updatehauler --dry-run os

# Preview scheduled task setup
updatehauler --dry-run schedule enable

# Preview with custom schedule time
updatehauler --dry-run --sched-hour "3" --sched-minute "30" schedule check

# Preview brew and cargo updates
updatehauler --dry-run brew brew-save cargo cargo-save
```

## What UpdateHauler Does

- ✅ Updates system packages across multiple operating systems
- ✅ Manages package backups and restores
- ✅ Automates updates via scheduling
- ✅ Provides dry-run mode for safe preview and CI/CD testing
- ✅ Supports sudo fallback for macOS softwareupdate in CI environments
- ✅ Provides comprehensive logging
- ✅ Supports color-coded output
- ✅ Self-updates via install/update commands
- ✅ Runs arbitrary commands with logging

## What UpdateHauler Does NOT Do

- ❌ Update Node.js packages (npm/yarn/pnpm)
- ❌ Update Python packages (pip)
- ❌ Update Ruby gems
- ❌ Update Go modules
- ❌ Update docker containers
- ❌ Snap packages (Linux)
- ❌ Flatpak packages (Linux)
- ❌ AUR packages (Arch Linux - requires yay/paru)
- ❌ GUI configuration or settings
- ❌ Version pinning or rollback capabilities
- ❌ Dependency conflict resolution

## Installation

### From Cargo Crates.io (when published)
```bash
cargo install updatehauler
```

### CI/CD Integration

UpdateHauler is designed to work well in CI/CD pipelines:

### Using Dry-Run for Testing

```bash
# In your CI pipeline, use dry-run to test configuration
- name: Test updatehauler configuration
  run: ./updatehauler --dry-run schedule check

# Validate what would be updated
- name: Preview updates
  run: ./updatehauler --dry-run

# Test specific actions
- name: Test backup commands
  run: ./updatehauler --dry-run brew-save cargo-save
```

### GitHub Actions Example

```yaml
name: Test Update Configuration
on: [pull_request]

jobs:
  test-config:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        run: cargo install --path .
      - name: Test schedule configuration
        run: updatehauler --dry-run schedule check
      - name: Preview updates
        run: updatehauler --dry-run os brew
```

### Benefits in CI/CD

- ✅ **No Password Prompts**: Dry-run mode doesn't require sudo password
- ✅ **No Side Effects**: Won't install packages or modify system state
- ✅ **Fast Execution**: No network calls to package repositories
- ✅ **Validation**: Confirms your configuration is correct
- ✅ **Safe**: Perfect for testing on PRs before merging

## GitHub Actions Workflows

This repository uses GitHub Actions for automated CI/CD:

### PR Validation Workflow (`.github/workflows/pr-validation.yml`)

Triggered on: Pull request events (opened, synchronize, reopened, edited)

**Features:**
- **Title Validation**: Checks PR titles follow conventional commit format
  - Supports: feat, fix, docs, style, refactor, test, chore, build, ci, perf, revert
  - Warns if format doesn't match
- **Body Validation**: Ensures PR has a description
  - Checks for change descriptions (added, removed, fixed, updated)
- **PR Size Analysis**: Calculates and labels PR by size
  - XS: < 100 lines
  - S: 100-499 lines
  - M: 500-999 lines
  - L: 1000-1999 lines
  - XL: 2000+ lines
- **Automatic Labeling**: Adds labels based on title and size
  - Type labels: type/feature, type/bug, type/documentation, etc.
  - Size labels: size/XS, size/S, size/M, size/L, size/XL
- **First-Time Contributors**: Welcomes new contributors with helpful resources
- **Dependabot Auto-Merge**: Automatically merges Dependabot PRs if checks pass
- **Large PR Notifications**: Comments on XL PRs requesting review and careful consideration

### PR Check Workflow (`.github/workflows/pr.yml`)

Triggered on: Pull requests to `main` or `develop`

**Features:**
- **Build (Debug & Release)**: Compiles on Ubuntu and macOS
- **Clippy**: Lint checks with warnings as errors
- **Rustfmt**: Code formatting checks
- **Test**: Runs full test suite on both platforms
- **Caching**: Optimizes build times with cargo caching
- **Dry-Run Tests**: Tests `--dry-run` flag for OS updates
  - Verifies dry-run mode doesn't prompt for passwords
  - Tests dry-run with multiple actions (os, brew, cargo)
  - Tests dry-run with backup commands
  - Tests dry-run with custom schedule times
  - Verifies dry-run doesn't modify system

### Release Build Workflow (`.github/workflows/release.yml`)

Triggered on: Push to `main` branch

**Features:**
- All checks from PR workflow (Build, Clippy, Rustfmt, Test)
- **Release Test Suite**: Runs `./test_release.sh` for end-to-end testing
- **Dry-Run Tests**: Tests `--dry-run` flag for OS updates
  - Verifies dry-run mode doesn't prompt for passwords
  - Tests dry-run with multiple actions (os, brew, cargo)
  - Tests dry-run with backup commands
  - Tests dry-run with custom schedule times
  - Verifies dry-run doesn't modify system
- Validates on both Ubuntu and macOS

## Building from Source

```bash
# Clone the repository
git clone https://github.com/franksplace/updatehauler.git
cd updatehauler

# Build debug version
cargo build

# Build release version (optimized)
cargo build --release

# Run tests
cargo test

# Run clippy for code quality checks
cargo clippy
```

The release binary will be located at `target/release/updatehauler`. You can install it system-wide with:

```bash
cargo install --path .
# or manually
cp target/release/updatehauler ~/.local/bin/
```

### Pre-Release Testing

Before creating a release candidate, run the comprehensive test suite:

```bash
./test_release.sh
```

This test suite validates:
- Binary compilation
- Unit and integration tests
- Command-line options (--help, --version, --run, etc.)
- Real-time output streaming
- Error handling for invalid actions
- All flag combinations (--no-color, --no-datetime, --no-header, --debug)
- Custom file paths (--logfile, --max-log-lines, --installdir, --brew-save-file, --cargo-save-file)
- Multiple action execution

The release binary will be located at `target/release/updatehauler`.

## License

Apache-2.0
