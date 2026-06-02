use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct DenoPlugin;

#[async_trait]
impl Plugin for DenoPlugin {
    fn name(&self) -> &str {
        "deno"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "deno".to_string(),
            description: "Upgrade the Deno runtime".to_string(),
            actions: vec![PluginAction {
                name: "deno".to_string(),
                description: "Upgrade Deno to the latest version".to_string(),
                action_type: Some(PluginActionType::Update),
            }],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_deno
    }

    async fn update(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        super::run_cmd(config, logger, true, "deno", &["upgrade"])?;
        Ok(())
    }

    async fn save(
        &self,
        _config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        logger.log("Deno is managed by deno upgrade - no save needed");
        Ok(())
    }

    async fn restore(
        &self,
        _config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        logger.log("Deno is managed by deno upgrade - no restore needed");
        Ok(())
    }
}
