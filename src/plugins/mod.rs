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

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;

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
