# Changelog

All notable changes to UpdateHauler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Hybrid plugin system with dynamic action discovery
- Per-plugin help system (`updatehauler <plugin> help`)
- Custom actions: brew-list, brew-outdated, brew-upgrade-pinned, cargo-list, cargo-outdated, nvim-list, nvim-clean, nvim-health
- New plugins: npm (update/save/restore), pip (update/save/restore)
- Plugin enable/disable via CLI flags (`--enable-plugin`, `--disable-plugin`)
- Desktop notifications on completion (`--notify`)
- Run summary with success/failure counts at end of execution
- Comprehensive test suite with 80 tests
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
- Hybrid design documentation (HYBRID_DESIGN.md)
- Implementation status tracking (IMPLEMENTATION_STATUS.md)

### Testing
- Unit tests for all modules
- Integration tests for end-to-end workflows
- Release test suite (test_release.sh)
- CI/CD workflows for GitHub Actions

## [0.2.0] - Upcoming (merged into 0.3.0)
