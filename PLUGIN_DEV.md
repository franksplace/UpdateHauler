# Plugin Development Guide

This guide explains how to create custom plugins for updatehauler.

## Overview

UpdateHauler uses a plugin-based architecture to support different package managers and update tools. Each plugin implements the `Plugin` trait defined in `src/plugins/mod.rs`.

## Quick Start

The fastest way to add a new plugin is:

1. Create plugin file in `src/plugins/`
2. Add module declaration in `src/plugins/mod.rs`
3. Register using the `register_plugins!` macro

The macro-based registration is streamlined and reduces boilerplate. See the detailed steps below for the complete process.

## Plugin Trait

Every plugin must implement the following async trait:

```rust
use async_trait::async_trait;
use anyhow::Result;

use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;

#[async_trait]
pub trait Plugin: Send + Sync {
    /// Returns the plugin's name
    fn name(&self) -> &str;

    /// Returns plugin metadata (name, description, available actions)
    fn get_metadata(&self) -> PluginMetadata;

    /// Checks if the plugin is available on the system
    async fn check_available(&self, config: &Config, insights: &Insights) -> bool;

    /// Updates packages managed by this plugin
    async fn update(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()>;

    /// Saves the current state of packages managed by this plugin
    /// Default: logs "No save needed" and returns Ok(()). Override if your plugin supports save.
    async fn save(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()> {
        logger.log("No save needed for this plugin");
        Ok(())
    }

    /// Restores packages from a previously saved state
    /// Default: logs "No restore needed" and returns Ok(()). Override if your plugin supports restore.
    async fn restore(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()> {
        logger.log("No restore needed for this plugin");
        Ok(())
    }

    /// Handle custom actions beyond update/save/restore
    /// Default: returns Ok(false) (action not handled). Override to add custom actions.
    async fn handle_custom_action(
        &self,
        _action_name: &str,
        _config: &Config,
        _insights: &Insights,
        _logger: &mut Logger,
    ) -> Result<bool> {
        Ok(false)
    }
}
```

## Creating a New Plugin

### 1. Create Plugin File

Create a new file in `src/plugins/` directory. For example, `src/plugins/my_plugin.rs`:

```rust
use async_trait::async_trait;
use duct::cmd;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "my_plugin".to_string(),
            description: "My custom plugin".to_string(),
            actions: vec![
                PluginAction {
                    name: "my_plugin".to_string(),
                    description: "Update my_plugin packages".to_string(),
                    action_type: Some(PluginActionType::Update),
                },
                PluginAction {
                    name: "my_plugin-save".to_string(),
                    description: "Save my_plugin state".to_string(),
                    action_type: Some(PluginActionType::Save),
                },
                PluginAction {
                    name: "my_plugin-restore".to_string(),
                    description: "Restore my_plugin state".to_string(),
                    action_type: Some(PluginActionType::Restore),
                },
            ],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        // Check if your tool is available
        // For example, check if a command exists
        cmd("my_tool", &["--version"]).stdout_null().run().is_ok()
    }

    async fn update(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()> {
        // Implement update logic
        super::run_cmd(config, logger, true, "my_tool", &["update"])?;
        super::run_cmd(config, logger, true, "my_tool", &["upgrade"])?;
        Ok(())
    }

    // save() and restore() have default implementations.
    // Only override them if your plugin supports save/restore.
}
```

### 2. Run Command Helper

A shared run_cmd() helper is available in src/plugins/mod.rs that provides consistent command execution with logging, dry-run support, and absolute path handling:

```rust
// No need to define your own — just use:
super::run_cmd(config, logger, true, "my_tool", &["update"])?;
```

The shared helper handles:
- **Dry-run mode**: Logs what would execute without running
- **Header output**: Prefixes output lines with the short command name
- **Error reporting**: Optionally logs errors with `show_error` flag
- **Sudo detection**: Correctly identifies sudo by basename for clean short-cmd display
- **Absolute paths**: Uses provided paths directly (system binaries resolved at build time)

### 3. Register Plugin

Add your plugin to `src/plugins/mod.rs`:

```rust
pub mod brew;
pub mod cargo;
pub mod nvim;
pub mod os;
pub mod my_plugin;  // Add this

pub use brew::BrewPlugin;
pub use cargo::CargoPlugin;
pub use nvim::NvimPlugin;
pub use os::OsPlugin;
pub use my_plugin::MyPlugin;  // Add this

// ... rest of file
```

### 4. Register with PluginRegistry

In `src/main.rs`, use the `register_plugins!` macro to register your plugin:

```rust
use update_hauler::{
    plugins::BrewPlugin,
    plugins::CargoPlugin,
    plugins::NvimPlugin,
    plugins::OsPlugin,
    plugins::MyPlugin,  // Add this
    plugins::PluginActionType,
    plugins::PluginRegistry,
    register_plugins,
};

fn create_plugin_registry() -> PluginRegistry<'static> {
    let mut registry = PluginRegistry::new();
    register_plugins!(
        registry,
        BrewPlugin,
        CargoPlugin,
        NvimPlugin,
        OsPlugin,
        MyPlugin,  // Add this
    );
    registry
}
```

**Benefits of macro-based registration:**
- Less boilerplate - no repetitive `registry.register(Box::new(...))` lines
- Consistent registration pattern
- Easy to add/remove plugins
- Trailing commas are supported for better diff formatting

### 5. Add Configuration Support (Optional)

In `src/config.rs`, add configuration for your plugin:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    pub brew: Option<bool>,
    pub cargo: Option<bool>,
    pub nvim: Option<bool>,
    pub os: Option<bool>,
    pub my_plugin: Option<bool>,  // Add this
}
```

### 5b. Add Custom Actions (Optional)

Plugins can expose custom actions beyond the standard update/save/restore by overriding `handle_custom_action()`:

```rust
use super::{Plugin, PluginAction, PluginActionType, PluginMetadata, Plugin};

#[async_trait]
impl Plugin for MyPlugin {
    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "my_plugin".to_string(),
            description: "My custom plugin".to_string(),
            actions: vec![
                // Standard actions
                PluginAction { name: "my_plugin".to_string(), description: "Update...".to_string(), action_type: Some(PluginActionType::Update) },
                // Custom actions (action_type: None)
                PluginAction { name: "my_plugin-list".to_string(), description: "List...".to_string(), action_type: None },
            ],
        }
    }

    async fn handle_custom_action(
        &self,
        action_name: &str,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<bool> {
        match action_name {
            "my_plugin-list" => {
                // Custom action logic
                super::run_cmd(config, logger, true, "my_tool", &["list"])?;
                Ok(true) // Return true when handled
            }
            _ => Ok(false), // Return false for unrecognized actions
        }
    }
}
```

Custom actions are discoverable via `updatehauler --help` and per-plugin help (`updatehauler my_plugin help`).

### 6. Add CLI Actions (Optional)

If you want users to be able to trigger your plugin explicitly, add actions in `src/main.rs`:

```rust
for action in &actions {
    match action.as_str() {
        "my-plugin" => {
            if let Some(plugin) = plugin_registry.get_plugin("my_plugin") {
                if rt.block_on(plugin.check_available(&config, &insights)) {
                    rt.block_on(plugin.update(&config, &insights, &mut logger))?;
                } else {
                    logger.error("my_plugin is not available");
                }
            }
        }
        "my-plugin-save" => {
            // Save action
        }
        "my-plugin-restore" => {
            // Restore action
        }
        // ... other actions
    }
}
```

## Plugin Examples

See existing plugins for reference:
- **Brew Plugin** (`src/plugins/brew.rs`) - Updates Homebrew packages
- **Cargo Plugin** (`src/plugins/cargo.rs`) - Updates Rust crates
- **Nvim Plugin** (`src/plugins/nvim.rs`) - Updates Neovim plugins
- **OS Plugin** (`src/plugins/os.rs`) - Updates system packages

## Best Practices

1. **Check Availability**: Always check if the tool is installed before attempting to use it
2. **Dry-Run Support**: Respect `config.dry_run` flag to preview changes
3. **Error Handling**: Use proper error handling with `anyhow::Result`
4. **Logging**: Use the provided `Logger` for consistent output
5. **Non-Blocking**: Make `save()` and `restore()` operations non-blocking - return `Ok(())` gracefully if files don't exist
6. **Platform Detection**: Use `insights` for OS/architecture-specific behavior
7. **Directory Creation**: Create necessary directories before writing files

## Testing

Create tests in `tests/plugins_test.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_plugin_name() {
        let plugin = MyPlugin;
        assert_eq!(plugin.name(), "my_plugin");
    }

    #[tokio::test]
    async fn test_my_plugin_dry_run() {
        let mut config = Config::new("/tmp/test");
        config.dry_run = true;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let plugin = MyPlugin;

        // Should not panic in dry-run mode
        let _ = plugin.update(&config, &insights, &mut logger).await;
    }
}
```

## Building and Testing

```bash
# Build the project
cargo build --release

# Run all tests
cargo test

# Run plugin tests only
cargo test plugins_test

# Check code quality
cargo clippy
```

## Configuration

Plugins can be enabled/disabled via YAML configuration:

```yaml
# ~/.config/updatehauler/config.yaml
plugins:
  brew: true
  cargo: true
  nvim: false
  os: true
  my_plugin: true  # Your plugin
```

## Contributing

When contributing a new plugin:
1. Follow the existing code style
2. Add comprehensive tests
3. Update this documentation
4. Add the plugin to `README.md`
5. Ensure clippy passes with no warnings
6. Update `examples/config.yaml` if relevant
