# Changelog

All notable changes to UpdateHauler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Hybrid plugin system with dynamic action discovery
- Per-plugin help system (`updatehauler <plugin> help`)
- Comprehensive test suite with 47 tests
- Shell completion support (bash and zsh)
- Code coverage reporting
- CONTRIBUTING.md guide for contributors

### Changed
- Improved error messages with helpful suggestions
- Refactored plugin registry for better extensibility
- Dynamic help text generation from plugin metadata

### Fixed
- Fixed test_release.sh hanging on restore commands
- Fixed missing imports in main.rs
- Fixed duplicate help text sections

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

## [0.2.0] - Upcoming

### Planned
- Custom action handlers for plugins
- Windows support
- Additional package manager plugins (npm, pip, etc.)
- Enhanced error reporting with suggestions
- Automated releases to crates.io
