pub mod brew;
pub mod cargo;
pub mod deno;
pub mod docker;
pub mod flatpak;
pub mod gem;
pub mod go;
pub mod npm;
pub mod nvim;
pub mod os;
pub mod pip;
pub mod run;
pub mod rustup;
pub mod snap;
pub mod uv;
pub mod vscode;
pub mod yarn;

use anyhow::Result;
use async_trait::async_trait;
pub use brew::BrewPlugin;
pub use cargo::CargoPlugin;
pub use deno::DenoPlugin;
pub use docker::DockerPlugin;
use duct::cmd;
pub use flatpak::FlatpakPlugin;
pub use gem::GemPlugin;
pub use go::GoPlugin;
pub use npm::NpmPlugin;
pub use nvim::NvimPlugin;
pub use os::OsPlugin;
pub use pip::PipPlugin;
pub use run::RunPlugin;
pub use rustup::RustupPlugin;
pub use snap::SnapPlugin;
use std::collections::HashSet;
pub use uv::UvPlugin;
pub use vscode::VscodePlugin;
pub use yarn::YarnPlugin;

use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;

/// Macro to register multiple plugins at once
/// Usage: register_plugins!(registry, BrewPlugin, CargoPlugin, NvimPlugin, OsPlugin);
#[macro_export]
macro_rules! register_plugins {
    ($registry:expr_2021, $($plugin:expr_2021),* $(,)?) => {
        $(
            $registry.register(Box::new($plugin));
        )*
    };
}

#[derive(Debug, PartialEq, Eq)]
pub enum PluginActionType {
    Update,
    Save,
    Restore,
}

pub struct PluginAction {
    pub name: String,
    pub description: String,
    pub action_type: Option<PluginActionType>,
}

pub struct PluginMetadata {
    pub name: String,
    pub description: String,
    pub actions: Vec<PluginAction>,
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;

    fn get_metadata(&self) -> PluginMetadata;

    async fn check_available(&self, config: &Config, insights: &Insights) -> bool;

    async fn update(&self, config: &Config, insights: &Insights, logger: &mut Logger)
    -> Result<()>;

    async fn save(
        &self,
        _config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        logger.log("No save needed for this plugin");
        Ok(())
    }

    async fn restore(
        &self,
        _config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        logger.log("No restore needed for this plugin");
        Ok(())
    }

    /// Handle custom actions (optional)
    /// Override this method to handle custom actions beyond update/save/restore
    /// Returns false if the action is not recognized
    async fn handle_custom_action(
        &self,
        _action_name: &str,
        _config: &Config,
        _insights: &Insights,
        _logger: &mut Logger,
    ) -> Result<bool> {
        // Default implementation: return false (action not handled)
        Ok(false)
    }
}

pub struct PluginRegistry<'a> {
    pub plugins: Vec<Box<dyn Plugin + 'a>>,
    metadata_cache: Vec<PluginMetadata>,
}

impl<'a> PluginRegistry<'a> {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            metadata_cache: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin + 'a>) {
        let name = plugin.name().to_string();
        if self.plugins.iter().any(|p| p.name() == name) {
            return;
        }
        self.metadata_cache.push(plugin.get_metadata());
        self.plugins.push(plugin);
    }

    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }

    pub fn get_action_by_name(&self, action_name: &str) -> Option<PluginAction> {
        for plugin in &self.plugins {
            let metadata = plugin.get_metadata();
            for action in metadata.actions {
                if action.name == action_name {
                    return Some(action);
                }
            }
        }
        None
    }

    pub fn get_all_metadata(&self) -> &Vec<PluginMetadata> {
        &self.metadata_cache
    }

    /// Find similar action names for helpful error messages
    pub fn find_similar_actions(&self, action_name: &str) -> Vec<String> {
        let mut similar = Vec::new();
        let available_actions = self.get_all_action_names();

        for action in available_actions {
            // Check for exact match or simple typos (one character difference)
            if action_name.len() > 2 && action.len() > 2 {
                let distance = strsim::levenshtein(action_name, &action);
                if distance == 0 || distance == 1 || (distance == 2 && action.len() > 4) {
                    similar.push(action);
                }
            }
            // Check for prefix match (e.g., "brew" vs "brew-save")
            else if action.starts_with(action_name) || action_name.starts_with(&action) {
                similar.push(action);
            }
        }

        // Limit to 3 suggestions
        similar.truncate(3);
        similar
    }

    /// Get all available action names
    pub fn get_all_action_names(&self) -> Vec<String> {
        let mut action_names = HashSet::new();
        for metadata in self.get_all_metadata() {
            for action in &metadata.actions {
                action_names.insert(action.name.clone());
            }
        }
        // Add non-plugin commands
        action_names.insert("install".to_string());
        action_names.insert("update".to_string());
        action_names.insert("remove".to_string());
        action_names.insert("install-completions".to_string());
        action_names.insert("schedule enable".to_string());
        action_names.insert("schedule disable".to_string());
        action_names.insert("schedule check".to_string());
        action_names.insert("trim-logfile".to_string());
        action_names.into_iter().collect()
    }

    pub async fn execute_action(
        &self,
        action_name: &str,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        if !action_name.contains('-') {
            if let Some(plugin) = self.get_plugin(action_name) {
                plugin.update(config, insights, logger).await?;
                return Ok(());
            }
        } else if let Some((plugin_name, _)) = action_name.split_once('-')
            && let Some(plugin) = self.get_plugin(plugin_name)
            && let Some(action_meta) = self.get_action_by_name(action_name)
        {
            match action_meta.action_type {
                Some(PluginActionType::Update) => plugin.update(config, insights, logger).await?,
                Some(PluginActionType::Save) => plugin.save(config, insights, logger).await?,
                Some(PluginActionType::Restore) => plugin.restore(config, insights, logger).await?,
                None => {
                    // Custom action - call handle_custom_action
                    if plugin
                        .handle_custom_action(action_name, config, insights, logger)
                        .await?
                    {
                        return Ok(());
                    } else {
                        plugin.update(config, insights, logger).await?
                    }
                }
            }
            return Ok(());
        }
        // Provide helpful suggestions for typos
        let similar = self.find_similar_actions(action_name);
        let suggestion = if similar.is_empty() {
            String::new()
        } else {
            format!("\nDid you mean: {}?", similar.join(", "))
        };

        anyhow::bail!(
            "Invalid action: {}. Run 'updatehauler --help' for available actions.{}",
            action_name,
            suggestion
        )
    }
}

const SUDO_PATH: &str = "/usr/bin/sudo";

fn validate_sudo_path() -> Result<()> {
    if !std::path::Path::new(SUDO_PATH).exists() {
        anyhow::bail!(
            "sudo not found at expected path '{}'. \
             This may indicate a compromised system or unusual PATH configuration.",
            SUDO_PATH
        );
    }
    Ok(())
}

pub fn sudo_command(config: &Config, program: &str, args: &[&str]) -> Result<std::process::Command> {
    if config.no_sudo || std::env::var("UPDATEHAULER_NO_SUDO").is_ok() {
        let mut cmd = std::process::Command::new(program);
        cmd.args(args);
        return Ok(cmd);
    }

    validate_sudo_path()?;

    let mut cmd = std::process::Command::new(SUDO_PATH);
    cmd.arg(program).args(args);
    Ok(cmd)
}

pub(crate) fn run_with_sudo(
    config: &Config,
    logger: &mut Logger,
    show_error: bool,
    command: &str,
    args: &[&str],
) -> Result<()> {
    if config.no_sudo || std::env::var("UPDATEHAULER_NO_SUDO").is_ok() {
        return run_cmd(config, logger, show_error, command, args);
    }

    validate_sudo_path()?;

    let mut sudo_args: Vec<&str> = vec![command];
    sudo_args.extend(args);
    run_cmd(config, logger, show_error, SUDO_PATH, &sudo_args)
}

pub(crate) fn run_cmd(
    config: &Config,
    logger: &mut Logger,
    show_error: bool,
    command: &str,
    args: &[&str],
) -> Result<()> {
    let cmd_str = format!("{} {}", command, args.join(" "));

    let is_sudo = std::path::Path::new(command)
        .file_name()
        .is_some_and(|n| n == "sudo");
    let short_cmd = if is_sudo && args.len() >= 4 {
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

            Ok(())
        }
        Err(e) => {
            if config.show_header {
                if show_error {
                    logger.error(&format!("{} → Error: {}", cmd_str, e));
                } else {
                    logger.log(&format!("{} → Return code {}", cmd_str, 1));
                }
            }
            Err(anyhow::anyhow!("{}", e))
        }
    }
}

impl<'a> Default for PluginRegistry<'a> {
    fn default() -> Self {
        Self::new()
    }
}
