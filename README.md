# UpdateHauler

UpdateHauler is a command-line caretaker for your entire development stack that rounds up, updates, and tidies everything in one go: operating system packages (macOS, Linux), Homebrew formulae and casks, Cargo crates, and any other updatable tools you wire in.

## Features

### Package Manager Support

**General Package Managers:**
- **Homebrew** - Update, upgrade, cleanup, and backup/restore brew formulas and casks
- **Cargo** - Update installed cargo packages and backup/restore cargo packages

**OS Package Managers:**
- **macOS** - System updates via `softwareupdate` and Mac App Store updates via `mas`
- **Debian/Ubuntu** - Updates via `apt-get`
- **Red Hat/CentOS/Fedora/Rocky Linux/Oracle Linux** - Updates via `dnf`
- **Alpine Linux** - Updates via `apk`
- **Arch Linux** - Updates via `pacman`
- **NixOS** - Updates via `nix-channel` and `nix-env`

### Key Capabilities

- **Automated Updates** - Update OS packages, Homebrew, and Cargo in a single command
- **Backup & Restore** - Save and restore package configurations for brew and cargo
- **Scheduling** - Set up automated updates via cron (Linux) or launchd (macOS)
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
| `os` | Update OS and app-based packages |

#### Backup/Restore Actions

| Action | Description |
|--------|-------------|
| `brew-save` | Save current brew installation to Brewfile |
| `brew-restore` | Restore brew installation from Brewfile |
| `cargo-save` | Save current cargo packages to backup JSON (requires `cargo-backup`) |
| `cargo-restore` | Restore cargo packages from backup JSON (requires `cargo-restore`) |

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

When run without actions, updatehauler automatically:
1. Updates OS packages
2. Updates Homebrew (if installed)
3. Saves brew configuration
4. Updates Cargo packages (if installed)
5. Saves cargo configuration
6. Trims logfile

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
All default locations can be overridden using command-line options:
- `--logfile` to specify a custom log file location
- `--brew-save-file` to specify a custom brew backup file
- `--cargo-save-file` to specify a custom cargo backup file
- `--installdir` to specify a custom installation directory

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

#### Linux Scheduling Details
- Adds cron entry to user's crontab
- Runs updates at scheduled time
- Default: 2:00 AM daily (`0 2 * * *`)
- System must be awake at scheduled time

## What UpdateHauler Does

- ✅ Updates system packages across multiple operating systems
- ✅ Manages package backups and restores
- ✅ Automates updates via scheduling
- ✅ Provides comprehensive logging
- ✅ Supports color-coded output
- ✅ Self-updates via install/update commands
- ✅ Runs arbitrary commands with logging

## What UpdateHauler Does NOT Do

- ❌ Update Neovim plugins (planned feature, not yet implemented)
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
- ❌ Dry-run mode (preview changes without applying)

## Installation

### From Cargo Crates.io (when published)
```bash
cargo install updatehauler
```

### Building from Source

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

MIT
