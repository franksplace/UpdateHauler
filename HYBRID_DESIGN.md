# Hybrid Plugin System - Design Proposal

## Problem Analysis

### Current Issues

1. **Hardcoded Actions**: Actions and help text are hardcoded in `main.rs` and `Args.after_help`
2. **Coupling**: Adding a plugin requires editing 3+ places:
   - Plugin file
   - `main.rs` action handler
   - Help text in `Args.after_help`
3. **No Custom Actions**: Plugins can't define custom actions like `brew-list`, `brew-upgrade-pinned`
4. **Rigid**: Can't add new action types without modifying core

## Proposed Hybrid Solution

### Core Concepts

1. **Convention-Based Defaults**: `<plugin>` = "update" action (default)
2. **Explicit Specific Actions**: `<plugin>-<action>` = specific action
3. **Dynamic Discovery**: Plugins self-describe their actions via metadata
4. **Custom Actions Support**: Plugins can add arbitrary custom actions

### Design

#### Plugin Metadata

```rust
pub struct PluginMetadata {
    pub name: String,
    pub description: String,
    pub actions: Vec<PluginAction>,
}

pub struct PluginAction {
    pub name: String,              // e.g., "brew", "brew-save", "brew-list"
    pub description: String,         // User-friendly description
    pub action_type: Option<PluginActionType>,  // Some(Update/Save/Restore) or None for custom
}

pub enum PluginActionType {
    Update,   // Standard update action
    Save,     // Standard save action
    Restore,  // Standard restore action
}
```

#### Action Resolution Logic

```rust
pub async fn execute_action(&self, action_name: &str, ...) -> Result<()> {
    // Case 1: Default action - just plugin name (e.g., "brew")
    if !action_name.contains('-') {
        if let Some(plugin) = self.get_plugin(action_name) {
            plugin.update(config, insights, logger).await?;
        }
    }
    // Case 2: Specific action with dash (e.g., "brew-save", "brew-list")
    else if let Some((plugin_name, action_suffix)) = action_name.split_once('-') {
        if let Some(plugin) = self.get_plugin(plugin_name) {
            if let Some(action_meta) = self.get_action_by_name(action_name) {
                match action_meta.action_type {
                    Some(PluginActionType::Update) => plugin.update(...).await?,
                    Some(PluginActionType::Save) => plugin.save(...).await?,
                    Some(PluginActionType::Restore) => plugin.restore(...).await?,
                    // Custom action - treat as update operation
                    None => plugin.update(config, insights, logger).await?,
                }
            }
        }
    }
}
```

### Examples

#### Example 1: Built-in Actions (Current Functionality)

```rust
impl BrewPlugin {
    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "brew".to_string(),
            description: "Update, upgrade, and clean brew formulas and casks".to_string(),
            actions: vec![
                PluginAction {
                    name: "brew".to_string(),                    // Default
                    description: "Update, upgrade, and clean brew formulas and casks".to_string(),
                    action_type: Some(PluginActionType::Update),
                },
                PluginAction {
                    name: "brew-save".to_string(),              // Specific
                    description: "Save brew bundle to Brewfile".to_string(),
                    action_type: Some(PluginActionType::Save),
                },
                PluginAction {
                    name: "brew-restore".to_string(),           // Specific
                    description: "Restore from the brew bundle".to_string(),
                    action_type: Some(PluginActionType::Restore),
                },
            ],
        }
    }
}
```

#### Example 2: Custom Actions (New Capability)

```rust
impl BrewPlugin {
    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "brew".to_string(),
            description: "Update, upgrade, and clean brew formulas and casks".to_string(),
            actions: vec![
                // Built-in actions...
                PluginAction { name: "brew".to_string(), action_type: Some(PluginActionType::Update), ... },
                PluginAction { name: "brew-save".to_string(), action_type: Some(PluginActionType::Save), ... },
                PluginAction { name: "brew-restore".to_string(), action_type: Some(PluginActionType::Restore), ... },

                // Custom actions - NEW!
                PluginAction {
                    name: "brew-list".to_string(),                 // Custom
                    description: "List all installed brew packages".to_string(),
                    action_type: None,  // Custom action
                },
                PluginAction {
                    name: "brew-upgrade-pinned".to_string(),     // Custom
                    description: "Only upgrade pinned brew packages".to_string(),
                    action_type: None,  // Custom action
                },
                PluginAction {
                    name: "brew-info".to_string(),                // Custom
                    description: "Show information about brew packages".to_string(),
                    action_type: None,  // Custom action
                },
            ],
        }
    }

    // New custom action handler
    pub async fn handle_custom_action(
        &self,
        action_name: &str,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        match action_name {
            "brew-list" => {
                Self::run_cmd(config, logger, true, "brew", &["list"])?;
            }
            "brew-upgrade-pinned" => {
                Self::run_cmd(config, logger, true, "brew", &["upgrade", "--pinned"])?;
            }
            "brew-info" => {
                // Show package info logic
                Self::run_cmd(config, logger, true, "brew", &["info"])?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}
```

#### Example 3: Main.rs Usage (Dynamic)

```rust
for action in &actions {
    match action.as_str() {
        // These are handled the same way - via plugin registry
        "brew" | "brew-save" | "brew-restore" | "brew-list" | "brew-info" => {
            plugin_registry.execute_action(action, &config, &insights, &mut logger).await?;
        }
        "cargo" | "cargo-save" | "cargo-restore" => {
            plugin_registry.execute_action(action, &config, &insights, &mut logger).await?;
        }
        "nvim" | "nvim-save" | "nvim-restore" => {
            plugin_registry.execute_action(action, &config, &insights, &mut logger).await?;
        }
        "os" => {
            plugin_registry.execute_action(action, &config, &insights, &mut logger).await?;
        }
        "install" | "update" | "remove" => {
            // Non-plugin actions remain unchanged
        }
        _ => {}
    }
}
```

#### Example 4: Dynamic Help Generation

```rust
// In main.rs - generate help from plugin metadata
fn build_help_text(plugin_registry: &PluginRegistry) -> String {
    let mut help = String::from("ACTIONS:\n\n");

    help.push_str("Update Actions:\n");
    for metadata in plugin_registry.get_all_metadata() {
        help.push_str(&format!("  {:<20} {}\n", metadata.name, metadata.description));

        for action in &metadata.actions {
            if action.name != metadata.name {  // Don't duplicate default action
                help.push_str(&format!("  {:<20} {}\n", action.name, action.description));
            }
        }
    }

    // ... add other sections
    help
}

// Args struct with dynamic help
#[derive(Parser)]
struct Args {
    #[arg(long, value_parser = build_help_text_from_plugins(...))]
    after_help: String,
    // ... other fields
}
```

### Benefits

#### For Users

1. **Discovery**: All actions available via `--help`
2. **Consistency**: `brew` = update (convention), `brew-save` = save (specific)
3. **Flexibility**: Plugins can add custom actions without core changes
4. **Multiple Actions**: Easy: `updatehauler brew brew-save os cargo`

#### For Plugin Developers

1. **Independence**: No need to modify `main.rs` - just register plugin
2. **Custom Actions**: Can add `brew-list`, `brew-info`, etc.
3. **Clear Interface**: `PluginActionType` for standard actions, `None` for custom
4. **Self-Documenting**: Help generated from `get_metadata()`

#### For Maintainers

1. **Decoupling**: Actions not hardcoded
2. **Extensible**: New actions from plugins appear automatically
3. **Maintainable**: Single source of truth (plugin metadata)
4. **Dynamic Help**: Help text generated from plugins

### Migration Path

#### Phase 1: Metadata Foundation (Current Task)

1. Add `PluginMetadata`, `PluginAction`, `PluginActionType` to `src/plugins/mod.rs`
2. Implement `get_metadata()` in all plugins
3. Add `get_action_by_name()` and `get_all_metadata()` to `PluginRegistry`
4. Implement `execute_action()` in `PluginRegistry` with the hybrid logic

#### Phase 2: Dynamic Actions (Future Enhancement)

1. Add custom action support to plugin trait (optional method)
2. Update action execution to call custom handlers
3. Generate help dynamically from plugin metadata
4. Remove hardcoded help text

### Comparison

| Feature | Current | Hybrid |
|----------|---------|--------|
| Default action convention | ✅ `brew` = update | ✅ `brew` = update |
| Specific actions | ✅ `brew-save` | ✅ `brew-save` |
| Custom actions | ❌ Can't add | ✅ Can add |
| Dynamic help | ❌ Hardcoded | ✅ Generated |
| Plugin independence | ⚠️  Edit main.rs | ✅ Auto-registered |
| Action discovery | ❌ Hidden | ✅ Visible in help |

## Summary

The hybrid system combines:
- **Convention-based simplicity**: `brew` = update (current behavior)
- **Explicit control**: `brew-save` = specific action (current behavior)
- **Dynamic extensibility**: Plugins define their own actions (NEW)
- **Self-documenting**: Help generated from metadata (FUTURE)

This maintains current UX while enabling plugin independence and custom actions without breaking existing workflows.

## Examples of Custom Actions

### Brew Plugin
```bash
# Built-in
updatehauler brew              # Update
updatehauler brew-save          # Save
updatehauler brew-restore       # Restore

# Custom (FUTURE)
updatehauler brew-list          # List all packages
updatehauler brew-info <pkg>    # Show package info
updatehauler brew-upgrade-pinned # Upgrade only pinned
updatehauler brew-outdated     # Show outdated packages
```

### Nvim Plugin
```bash
# Built-in
updatehauler nvim              # Update
updatehauler nvim-save          # Save (noted in config)

# Custom (FUTURE)
updatehauler nvim-list         # List installed plugins
updatehauler nvim-clean         # Clean unused plugins
updatehauler nvim-health        # Check plugin health
```

### Cargo Plugin
```bash
# Built-in
updatehauler cargo              # Update
updatehauler cargo-save          # Save
updatehauler cargo-restore       # Restore

# Custom (FUTURE)
updatehauler cargo-list        # List installed crates
updatehauler cargo-outdated    # Show outdated crates
updatehauler cargo-update-selective <crate> # Update specific crate
```

## Implementation Priority

### Immediate (This Session)
1. ✅ Add `PluginMetadata`, `PluginAction`, `PluginActionType` to trait
2. ✅ Implement `get_metadata()` for all plugins
3. ✅ Add registry methods: `get_action_by_name()`, `get_all_metadata()`
4. ✅ Implement `execute_action()` with hybrid logic

### Future (Next Version)
1. Support custom action handlers
2. Dynamic help generation from plugins
3. Action completion for shell
4. Per-plugin action discovery (`updatehauler brew --help`)

## Conclusion

The hybrid approach gives us:
- ✅ **Backwards Compatible**: All current commands work exactly the same
- ✅ **Plugin Independent**: Add plugins by registering only
- ✅ **Extensible**: Custom actions without core changes
- ✅ **Convention-Based**: `brew` = update is intuitive
- ✅ **Well-Structured**: Clear separation of concerns

The core insight: `execute_action()` intelligently handles both convention (`brew`) and explicit actions (`brew-save`, `brew-list`) by checking the dash delimiter and action metadata.
