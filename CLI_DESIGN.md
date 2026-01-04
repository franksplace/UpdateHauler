# UpdateHauler Action Design

## Current Action Syntax

UpdateHauler uses a compound action naming convention:

```
updatehauler <plugin-name>-<action>
```

Examples:
- `updatehauler brew` - Update Homebrew packages
- `updatehauler brew-save` - Save Homebrew state
- `updatehauler brew-restore` - Restore Homebrew state
- `updatehauler brew brew-save` - Update and save in one command
- `updatehauler os brew cargo` - Run multiple plugin updates

## Why This Design?

### 1. Follows Package Manager Conventions

Most package management tools use **separate actions** rather than subcommands:

| Tool      | Pattern                     | Examples                         |
|-----------|----------------------------|-----------------------------------|
| **brew** | `brew <action>`           | `brew install`, `brew upgrade`     |
| **apt**   | `apt <action>`            | `apt update`, `apt upgrade`       |
| **dnf**   | `dnf <action>`            | `dnf update`, `dnf upgrade`       |
| **pip**   | `pip <action>`            | `pip install`, `pip freeze`        |
| **npm**   | `npm <action>`            | `npm install`, `npm update`        |
| **cargo** | `cargo <action>`           | `cargo build`, `cargo run`        |

### 2. Easy to Run Multiple Actions

With compound action names, users can easily run multiple operations:

```bash
# Update multiple plugins
updatehauler os brew cargo nvim

# Update and save in one command
updatehauler brew brew-save cargo cargo-save

# Complete workflow
updatehauler os brew brew-save trim-logfile
```

### 3. Simpler CLI Parsing

- No complex subcommand parsing required
- Shell completion is straightforward
- Help text is clearer

### 4. Flexible and Discoverable

Users can discover all available actions:

```bash
updatehauler --help
```

Shows all actions like:
- `brew` - Update brew
- `brew-save` - Save brew state
- `brew-restore` - Restore brew state

## Alternatives Considered

### Alternative 1: Subcommands

```
updatehauler brew update
updatehauler brew save
updatehauler brew restore
```

**Pros:**
- More structured
- Follows patterns like `kubectl get`, `helm install`

**Cons:**
- Requires more typing
- Running multiple actions is verbose: `updatehauler brew update; updatehauler brew save`
- Less common for package managers
- CLI parsing is more complex

### Alternative 2: Action Flags

```
updatehauler brew --update
updatehauler brew --save
updatehauler brew --restore
```

**Pros:**
- Grouped by plugin

**Cons:**
- Running multiple plugins requires separate commands
- More complex flag combinations
- Shell completion more complex

### Alternative 3: Separate Update Actions

```
updatehauler brew-update
updatehauler brew-save
updatehauler brew-restore
```

This is essentially what we have, just with a dash instead of the implicit "update" action. Our current design treats the bare plugin name as "update" which is intuitive.

## Multiple Actions in Practice

The current design makes it very easy to run multiple operations:

```bash
# Update all installed package managers
updatehauler

# Update specific plugins
updatehauler os brew

# Update and save specific plugins
updatehauler brew brew-save

# Complete workflow: update, save, cleanup
updatehauler os brew brew-save trim-logfile

# Restore from backup
updatehauler brew-restore
updatehauler cargo-restore
```

## Future Extensibility

If we wanted to support more complex operations in the future, we could add them without breaking existing CLI:

```bash
# Current
updatehauler brew
updatehauler brew-save
updatehauler brew-restore

# Possible future additions (not implemented)
updatehauler brew-list       # List installed packages
updatehauler brew-info        # Show package info
updatehauler brew-search      # Search for packages
```

## Conclusion

The current compound action design (`<plugin-name>-<action>`) is intentionally chosen because:

1. ✅ Follows package manager conventions (not container orchestration tools)
2. ✅ Easy to run multiple actions in one command
3. ✅ Simple CLI parsing and help
4. ✅ Intuitive - bare plugin name means "update"
5. ✅ Flexible for future extensions
6. ✅ Users can compose workflows easily

The design prioritizes **usability for common workflows** (update multiple things, update + save) over strict hierarchical organization.
