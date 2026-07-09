# Changelog

All notable changes to UpdateHauler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1]

### Security
- **Sudo path hardening**: All sudo invocations now validate that `/usr/bin/sudo` exists at the expected system path before execution, preventing PATH hijacking attacks
- **Centralized sudo utility**: Consolidated all sudo usage into `sudo_command()` and `run_with_sudo()` in a single module for consistent security validation
- **Command audit logging**: The `run` plugin now logs all executed commands to the log file with timestamps and usernames via `[AUDIT]` entries, providing an audit trail
- **Confirmation prompt for arbitrary commands**: New `--confirm-run` flag prompts for user confirmation before executing commands via `updatehauler run --cmd`

### Added
- `--no-sudo` CLI flag and `no_sudo` config option to skip sudo elevation entirely
- `--confirm-run` CLI flag and `confirm_run` config option to prompt before running arbitrary commands
- `UPDATEHAULER_NO_SUDO` environment variable support for non-interactive sudo bypass
- `Logger::audit()` method for always-on audit log entries regardless of `--logfile-only` setting

### Changed
- Cleaned up markdown formatting across README and documentation
- Improved `--help` output formatting and readability
- Refactored `os.rs` to use centralized `run_with_sudo()` instead of raw sudo paths
- Refactored `scheduler.rs` `pmset` calls to use `sudo_command()` instead of raw `Command::new("/usr/bin/sudo")`

## [0.3.0]

### Added
- Hybrid plugin system with dynamic action discovery
- Per-plugin help system (`updatehauler <plugin> help`)
- Custom actions: brew-list, brew-outdated, brew-upgrade-pinned, cargo-list, cargo-outdated, nvim-list, nvim-clean, nvim-health
- New plugins: npm (update/save/restore), pip (update/save/restore)
- Plugin enable/disable via CLI flags (`--enable-plugin`, `--disable-plugin`)
- Desktop notifications on completion (`--notify`)
- Run summary with success/failure counts at end of execution
- Comprehensive test suite with 123 tests
- Shell completion support (bash, zsh, fish, powershell, elvish) with descriptive help
- Context-aware completions (schedule subcommands, shell types after parent commands)
- Code coverage reporting
- CONTRIBUTING.md guide for contributors

### Changed
- Improved error messages with helpful suggestions
- Refactored plugin registry for better extensibility
- Dynamic help text generation from plugin metadata
- Dynamic plugin listing in error messages
- Enhanced shell completions to use clap_complete for better integration
- Added action validation with PossibleValuesParser
- `--run` command now respects `--dry-run` mode
- Schedule values now accept ASCII letters (MON,WED,FRI style day names)
- Crontab entry format fixed (removed stray semicolon)
- Removed unused sys-info dependency
- Removed dead code: package_manager.rs, utils.rs, log_save_dir field
- Moved tempfile to dev-dependencies

### Fixed
- Fixed test_release.sh hanging on restore commands
- Fixed missing imports in main.rs
- Fixed duplicate help text sections
- Fixed shell completion installation paths
- Fixed nvim restore commands (incorrect -c arg wrapping)
- Fixed test_release.sh version check (0.2.0 → 0.3.0)

## [0.1.0] - 2026-01-02

### Added
- Initial release of UpdateHauler
- Plugin-based architecture for package manager support
- Homebrew plugin with update, save, restore actions
- Cargo plugin with update, save, restore actions
- Neovim plugin with update, save, restore actions
- OS plugin for system updates (macOS and Linux)
- YAML configuration file support
- Dry-run mode for safe testing
- Comprehensive logging with rotation
- Scheduled updates (cron on Linux, launchd on macOS)
- Self-installation and updates
- Shell completion generation
- Color output and timestamps
- Multi-platform support (macOS and Linux)

### Features
- Update OS packages across multiple platforms (macOS, Debian, Ubuntu, RHEL, Alpine, Arch, NixOS)
- Manage Homebrew formulas and casks
- Update Cargo packages
- Update Neovim plugins (lazy.nvim, packer.nvim, vim-plug)
- Backup and restore package configurations
- Automated scheduling
- Custom configuration via YAML
- Flexible CLI with multiple flags

### Documentation
- Comprehensive README with examples
- Plugin development guide (PLUGIN_DEV.md)
- CLI design documentation (CLI_DESIGN.md)

### Testing
- Unit tests for all modules
- Integration tests for end-to-end workflows
- Release test suite (test_release.sh)
- CI/CD workflows for GitHub Actions

## [0.2.0] - Upcoming (merged into 0.3.0)
