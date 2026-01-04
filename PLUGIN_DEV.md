# Plugin Development Guide

This guide explains how to create custom plugins for updatehauler.

## Overview

UpdateHauler uses a plugin-based architecture to support different package managers and update tools. Each plugin implements the `Plugin` trait defined in `src/plugins/mod.rs`.

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

    /// Checks if the plugin is available on the system
    async fn check_available(&self, config: &Config, insights: &Insights) -> bool;

    /// Updates packages managed by this plugin
    async fn update(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()>;

    /// Saves the current state of packages managed by this plugin
    async fn save(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()>;

    /// Restores packages from a previously saved state
    async fn restore(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()>;
}
```

## Creating a New Plugin

### 1. Create Plugin File

Create a new file in `src/plugins/` directory. For example, `src/plugins/my_plugin.rs`:

```rust
use async_trait::async_trait;
use duct::cmd;

use super::Plugin;
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

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        // Check if your tool is available
        // For example, check if a command exists
        cmd("my_tool", &["--version"]).stdout_null().run().is_ok()
    }

    async fn update(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()> {
        // Implement update logic
        Self::run_cmd(config, logger, true, "my_tool", &["update"])?;
        Self::run_cmd(config, logger, true, "my_tool", &["upgrade"])?;
        Ok(())
    }

    async fn save(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()> {
        // Implement save logic (optional)
        logger.log("Saving my_plugin state...");
        // Save current package list to a file, etc.
        Ok(())
    }

    async fn restore(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()> {
        // Implement restore logic (optional)
        logger.log("Restoring my_plugin state...");
        // Restore packages from a previously saved file
        Ok(())
    }
}
```

### 2. Run Command Helper

The `run_cmd` helper method provides consistent command execution with logging and dry-run support:

```rust
impl MyPlugin {
    fn run_cmd(
        config: &Config,
        logger: &mut Logger,
        show_error: bool,
        command: &str,
        args: &[&str],
    ) -> Result<()> {
        let cmd_str = format!("{} {}", command, args.join(" "));

        let short_cmd = if command == "sudo" && args.len() >= 4 {
            args[3]
        } else {
            command
        };

        if config.dry_run {
            if config.show_header {
                logger.log(&format!("{} → Start (DRY-RUN)", cmd_str));
            }
            logger.log(&format!("Would execute: {}", cmd_str));
            if config.show_header {
                logger.log(&format!("{} → Return code 0 (DRY-RUN)", cmd_str));
            }
            return Ok(());
        }

        if config.show_header {
            logger.log(&format!("{} → Start", cmd_str));
        }

        // Execute command using duct for better output handling
        let result = cmd(command, args).stdout_capture().stderr_capture().run();

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                for line in stdout.lines() {
                    let formatted = if config.show_header {
                        format!("{} → {}", short_cmd, line)
                    } else {
                        line.to_string()
                    };
                    logger.log(&formatted);
                }

                for line in stderr.lines() {
                    let formatted = if config.show_header {
                        format!("{} → {}", short_cmd, line)
                    } else {
                        line.to_string()
                    };
                    logger.log(&formatted);
                }

                if config.show_header {
                    if show_error && !output.status.success() {
                        logger.error(&format!(
                            "{} → Return code {}",
                            cmd_str,
                            output.status.code().unwrap_or(1)
                        ));
                    } else {
                        logger.log(&format!(
                            "{} → Return code {}",
                            cmd_str,
                            output.status.code().unwrap_or(0)
                        ));
                    }
                }
            }
            Err(e) => {
                if config.show_header {
                    if show_error {
                        logger.error(&format!("{} → Error: {}", cmd_str, e));
                    } else {
                        logger.log(&format!("{} → Return code {}", cmd_str, 1));
                    }
                }
            }
        }

        Ok(())
    }
}
```

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

// ... rest of the file
```

### 4. Register with PluginRegistry

In `src/main.rs`, register your plugin with the registry:

```rust
let mut plugin_registry = PluginRegistry::new();
plugin_registry.register(Box::new(BrewPlugin));
plugin_registry.register(Box::new(CargoPlugin));
plugin_registry.register(Box::new(NvimPlugin));
plugin_registry.register(Box::new(OsPlugin));
plugin_registry.register(Box::new(MyPlugin));  // Add this
```

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
