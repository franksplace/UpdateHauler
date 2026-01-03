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

use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;

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
}

pub struct PluginRegistry<'a> {
    plugins: Vec<Box<dyn Plugin + 'a>>,
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
                        None => plugin.update(config, insights, logger).await?,
                    }
                    return Ok(());
                }
            }
        }
        anyhow::bail!(
            "Invalid action: {}. Run 'updatehauler --help' for available actions.",
            action_name
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
