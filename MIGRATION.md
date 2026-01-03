# UpdateHauler Migration Summary

## Completed Changes

### 1. Package Manager Module Status

**Status:** Deprecated but retained for backwards compatibility

- `src/package_manager.rs` - Still exists but is **NO LONGER USED** in main.rs
- All functionality has been migrated to the plugin framework
- Previously used for: `brew-restore` and `cargo-restore`
- Now using: Plugin-based approach with `BrewPlugin` and `CargoPlugin`

**Migration Impact:** None for users. The CLI commands remain the same.

### 2. Plugin Framework

**Implemented:** Complete plugin-based architecture

**Core Components:**
- `src/plugins/mod.rs` - Plugin trait and registry
- `src/plugins/brew.rs` - Homebrew management
- `src/plugins/cargo.rs` - Cargo crates management
- `src/plugins/nvim.rs` - Neovim plugins management
- `src/plugins/os.rs` - OS package management

**Plugin Trait Methods:**
- `name()` - Returns plugin identifier
- `check_available()` - Checks if plugin can run
- `update()` - Updates packages
- `save()` - Saves current state
- `restore()` - Restores from saved state

### 3. Configuration

**Implemented:** YAML configuration support

**Location:** `~/.config/updatehauler/config.yaml` (default)
**Option:** `--config-file <path>` to specify custom config

**New Configuration Options:**
- `plugins.brew` - Enable/disable brew plugin
- `plugins.cargo` - Enable/disable cargo plugin
- `plugins.nvim` - Enable/disable nvim plugin
- `plugins.os` - Enable/disable os plugin
- All schedule options moved to YAML
- All logging options moved to YAML
- All path options moved to YAML

**Example:** See `examples/config.yaml`

### 4. Main Changes

**Refactored:** Actions to use plugins instead of package_manager

**Changes:**
- Removed `mod package_manager` and `use package_manager::PackageManager`
- `brew-restore` now uses `BrewPlugin::restore()`
- `cargo-restore` now uses `CargoPlugin::restore()`
- Plugin registry initialized and used for all actions

**New Actions:**
- `nvim` - Update Neovim plugins
- `nvim-save` - Note nvim plugin configuration
- `nvim-restore` - Restore nvim plugins

### 5. Tests

**Created:** Comprehensive plugin tests

**New Test File:** `tests/plugins_test.rs`

**Tests Added (14 total):**
- Plugin registry creation and management
- Individual plugin names
- Plugin availability checks
- Dry-run mode
- Save/restore with missing files
- Nvim plugin operations

**All Test Results:** 52 tests pass, 0 failures

### 6. Build Quality

**Status:** Clean build with no errors or warnings

**Results:**
- ✅ `cargo build --release` - Success
- ✅ `cargo test` - All 52 tests pass
- ✅ `cargo clippy` - No warnings
- ✅ `./test_release.sh` - All 22 test sections pass

### 7. Documentation

**Created:** Two new documentation files

**PLUGIN_DEV.md** - Plugin development guide
- Plugin trait documentation
- Step-by-step plugin creation tutorial
- Best practices
- Testing guidelines
- Configuration integration

**CLI_DESIGN.md** - CLI action design rationale
- Why compound action names are used (`brew-save`, not `brew save`)
- Comparison with alternative approaches
- Usability considerations
- Examples of common workflows

## CLI Action Design - Detailed Rationale

### Why `<plugin-name>-<action>` Design?

The current design uses:
- `brew` - Update brew
- `brew-save` - Save brew state
- `brew-restore` - Restore brew state

### 1. Follows Package Manager Conventions

Most package management tools use **separate actions**, not subcommands:

| Tool      | Pattern                     | Examples                         |
|-----------|----------------------------|-----------------------------------|
| **brew**   | `brew <action>`           | `brew install`, `brew upgrade`     |
| **apt**    | `apt <action>`            | `apt update`, `apt upgrade`       |
| **dnf**    | `dnf <action>`            | `dnf update`, `dnf upgrade`       |
| **pip**    | `pip <action>`            | `pip install`, `pip freeze`        |
| **npm**    | `npm <action>`            | `npm install`, `npm update`        |
| **cargo**  | `cargo <action>`           | `cargo build`, `cargo run`        |

Container orchestration tools use subcommands (`kubectl get`, `helm install`), but these are different use cases.

### 2. Easy to Run Multiple Actions

```bash
# Update multiple plugins
updatehauler os brew cargo nvim

# Update and save in one command
updatehauler brew brew-save cargo cargo-save

# Complete workflow
updatehauler os brew brew-save trim-logfile

# Restore from backup
updatehauler brew-restore cargo-restore
```

### 3. Simpler CLI Parsing and Shell Completion

- No complex subcommand parsing required
- Shell completion is straightforward
- Help text is clearer

### 4. Flexible and Discoverable

Users can easily discover all actions:
```bash
updatehauler --help
```

Shows all actions like:
- `brew` - Update brew
- `brew-save` - Save brew state
- `brew-restore` - Restore brew state

### 5. Supports Intuitive Default Actions

The bare plugin name (`brew`, `cargo`, `os`) naturally means "update":
```bash
updatehauler  # Runs: os, brew, brew-save, cargo, cargo-save, trim-logfile
```

This makes sense for the primary use case: updating everything.

### Alternatives Considered

#### Alternative 1: Subcommands
```
updatehauler brew update
updatehauler brew save
updatehauler brew restore
```

**Pros:** More structured, follows patterns like `kubectl get`

**Cons:**
- Requires more typing
- Running multiple actions is verbose: `updatehauler brew update; updatehauler brew save`
- Less common for package managers
- CLI parsing is more complex
- Shell completion more complex

#### Alternative 2: Action Flags
```
updatehauler brew --update
updatehauler brew --save
updatehauler brew --restore
```

**Pros:** Grouped by plugin

**Cons:**
- Running multiple plugins requires separate commands
- More complex flag combinations
- Shell completion more complex

#### Alternative 3: Separate Update Actions
```
updatehauler brew-update
updatehauler brew-save
updatehauler brew-restore
```

This is essentially what we have, just with a dash instead of hyphen.

### Future Extensibility

If we need to add more complex operations in the future, we can add them without breaking existing CLI:

```bash
# Current
updatehauler brew
updatehauler brew-save
updatehauler brew-restore

# Possible future additions (not breaking)
updatehauler brew-list      # List installed packages
updatehauler brew-info       # Show package info
updatehauler brew-search     # Search for packages
```

The design prioritizes **usability for common workflows** (update multiple things, update + save, restore) over strict hierarchical organization.

## Migration Path for Users

### No Breaking Changes

Users don't need to change anything. All existing commands work the same:

```bash
# All still work exactly as before
updatehauler
updatehauler brew
updatehauler brew-save
updatehauler brew-restore
updatehauler cargo
updatehauler cargo-save
updatehauler cargo-restore
updatehauler os
updatehauler trim-logfile
```

### New Capabilities

Users can now:
1. Use YAML configuration file
2. Enable/disable plugins via config
3. Manage Neovim plugins
4. Easier to extend with new plugins

### Example: New Config File

Create `~/.config/updatehauler/config.yaml`:

```yaml
# Disable nvim plugin, keep others
plugins:
  brew: true
  cargo: true
  nvim: false
  os: true

# Customize schedule
schedule:
  hour: "3"
  minute: "30"

# Enable dry-run for testing
dry_run: false
```

## Development Benefits

### For Plugin Developers

- Clear trait to implement
- Consistent API across plugins
- Easy to add new package managers
- Comprehensive examples
- Testing guidelines

### For Core Developers

- Modular architecture
- Easier to maintain
- Plugins can be developed independently
- Clear separation of concerns

## Conclusion

The migration to a plugin-based architecture:
- ✅ Maintains full backwards compatibility
- ✅ Adds new capabilities (nvim, config)
- ✅ Improves code organization
- ✅ Makes it easier to extend
- ✅ Passes all tests and code quality checks
- ✅ Provides comprehensive documentation

The CLI action design (`<plugin-name>-<action>`) is intentional and follows industry best practices for package management tools, prioritizing usability for common workflows over strict hierarchical organization.
