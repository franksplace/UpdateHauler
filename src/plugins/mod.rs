#![allow(unused_imports)]

pub mod brew;
pub mod cargo;
pub mod nvim;
pub mod os;

pub use brew::BrewPlugin;
pub use cargo::CargoPlugin;
pub use nvim::NvimPlugin;
pub use os::OsPlugin;

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashSet;

use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;

/// Simple character difference for fuzzy matching
#[allow(clippy::needless_range_loop)]
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 0..=m {
        dp[i][0] = i;
    }

    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[m][n]
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

    async fn save(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()>;

    async fn restore(
        &self,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()>;

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
}

impl<'a> PluginRegistry<'a> {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin + 'a>) {
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

    pub fn get_all_metadata(&self) -> Vec<PluginMetadata> {
        self.plugins.iter().map(|p| p.get_metadata()).collect()
    }

    /// Find similar action names for helpful error messages
    pub fn find_similar_actions(&self, action_name: &str) -> Vec<String> {
        let mut similar = Vec::new();
        let available_actions = self.get_all_action_names();

        for action in available_actions {
            // Check for exact match or simple typos (one character difference)
            if action_name.len() > 2 && action.len() > 2 {
                let distance = levenshtein_distance(action_name, &action);
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
            for action in metadata.actions {
                action_names.insert(action.name.clone());
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
        }
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
        } else if let Some((plugin_name, _)) = action_name.split_once('-') {
            if let Some(plugin) = self.get_plugin(plugin_name) {
                if let Some(action_meta) = self.get_action_by_name(action_name) {
                    match action_meta.action_type {
                        Some(PluginActionType::Update) => {
                            plugin.update(config, insights, logger).await?
                        }
                        Some(PluginActionType::Save) => {
                            plugin.save(config, insights, logger).await?
                        }
                        Some(PluginActionType::Restore) => {
                            plugin.restore(config, insights, logger).await?
                        }
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
            }
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

    #[allow(dead_code)]
    pub async fn run_available_plugins(
        &self,
        action: &str,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        for plugin in &self.plugins {
            if plugin.check_available(config, insights).await {
                match action {
                    "update" => {
                        plugin.update(config, insights, logger).await?;
                    }
                    "save" => {
                        plugin.save(config, insights, logger).await?;
                    }
                    "restore" => {
                        plugin.restore(config, insights, logger).await?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl<'a> Default for PluginRegistry<'a> {
    fn default() -> Self {
        Self::new()
    }
}
