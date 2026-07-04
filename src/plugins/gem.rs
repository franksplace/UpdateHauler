use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct GemPlugin;

#[async_trait]
impl Plugin for GemPlugin {
    fn name(&self) -> &str {
        "gem"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "gem".to_string(),
            description: "Update Ruby gems".to_string(),
            actions: vec![PluginAction {
                name: "gem".to_string(),
                description: "Update Ruby gems and rubygems system".to_string(),
                action_type: Some(PluginActionType::Update),
            }],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_gem
    }

    async fn update(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        super::run_cmd(config, logger, true, "gem", &["update", "--system"])?;
        super::run_cmd(config, logger, true, "gem", &["update"])?;
        Ok(())
    }

    async fn save(
        &self,
        _config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        logger.log("Ruby gems are managed by gem - no save needed");
        Ok(())
    }

    async fn restore(
        &self,
        _config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        logger.log("Ruby gems are managed by gem - no restore needed");
        Ok(())
    }
}
