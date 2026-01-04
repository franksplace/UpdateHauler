# Contributing to UpdateHauler

Thank you for your interest in contributing to UpdateHauler! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and considerate in all interactions. We aim to maintain a welcoming and inclusive community.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- Make (optional, for convenient commands)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/franksplace/updatehauler.git
cd updatehauler

# Install dependencies
cargo build

# Run tests
cargo test

# Run with release optimization
cargo build --release
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Changes

- Write clean, readable code
- Follow existing code style
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run all tests
cargo test --all-targets

# Run clippy (code quality checks)
cargo clippy --all-targets -- -D warnings

# Check code formatting
cargo fmt -- --check

# Format code if needed
cargo fmt

# Run the comprehensive test suite
./test_release.sh
```

### 4. Commit Changes

Use [conventional commits](https://www.conventionalcommits.org/) for commit messages:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting)
- `refactor:` Code refactoring
- `test:` Adding or updating tests
- `chore:` Maintenance tasks
- `build:` Build system changes
- `ci:` CI/CD changes
- `perf:` Performance improvements
- `revert:` Revert a previous commit

Examples:
```
feat: add support for npm plugin
fix: prevent hang when brew-restore has no packages
docs: update README with new examples
test: add integration test for --run command
```

### 5. Create a Pull Request

Push your branch and create a pull request:

```bash
git push origin feature/your-feature-name
```

## Coding Standards

### Rust Best Practices

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `anyhow` for error handling
- Use `async_trait` for async trait implementations
- Prefer `Result<T>` over panicking

### Code Style

- Use meaningful variable and function names
- Keep functions focused and small
- Add doc comments for public APIs
- Follow existing naming conventions

### Testing

- Write unit tests for all new functions
- Add integration tests for new features
- Ensure all tests pass before submitting
- Aim for good test coverage

## Plugin Development

See [PLUGIN_DEV.md](PLUGIN_DEV.md) for detailed guide on creating new plugins.

### Quick Start

1. Create plugin file in `src/plugins/`
2. Implement the `Plugin` trait
3. Register plugin in `src/plugins/mod.rs`
4. Register plugin in `src/main.rs`
5. Add tests in `tests/plugins_test.rs`

## Documentation

### README.md

Keep the README up to date:
- Update feature descriptions
- Add new examples
- Document configuration options
- Update installation instructions

### Code Comments

- Add doc comments to public functions
- Explain non-obvious logic
- Keep comments concise and helpful

## Testing Guidelines

### Unit Tests

Test individual functions and methods in isolation.

```rust
#[tokio::test]
async fn test_plugin_name() {
    let plugin = BrewPlugin;
    assert_eq!(plugin.name(), "brew");
}
```

### Integration Tests

Test multiple components working together.

```rust
#[tokio::test]
async fn test_multiple_actions() {
    let config = Config::new("/tmp/test");
    let insights = Insights::new()?;
    let mut logger = Logger::new(&config);
    let plugin = BrewPlugin;
    plugin.update(&config, &insights, &mut logger).await?;
}
```

### Release Testing

Run the comprehensive test suite before proposing changes:

```bash
./test_release.sh
```

## CI/CD

### Pull Request Checks

All pull requests must pass:
- Build on Ubuntu and macOS
- Clippy checks (no warnings)
- Rustfmt checks
- All tests
- Dry-run tests

### Workflows

- `.github/workflows/pr-validation.yml` - Validates PR format and size
- `.github/workflows/pr.yml` - Runs tests on PRs
- `.github/workflows/release.yml` - Tests releases on main branch

## Release Process

Releases are automated when pushing to `main` branch:
1. All tests run on Ubuntu and macOS
2. Release test suite runs
3. Dry-run tests execute
4. Manual release creation on GitHub

### Versioning

UpdateHauler uses [semantic versioning](https://semver.org/):
- MAJOR: Breaking changes
- MINOR: New features (backwards compatible)
- PATCH: Bug fixes (backwards compatible)

## Questions?

- Check [README.md](README.md) for usage
- See [PLUGIN_DEV.md](PLUGIN_DEV.md) for plugin development
- Review [CLI_DESIGN.md](CLI_DESIGN.md) for design decisions
- See [SECURITY.md](SECURITY.md) for security policy and vulnerability reporting
- Open an issue for bugs or feature requests

## License

By contributing, you agree that your contributions will be licensed under the Apache-2.0 License.
