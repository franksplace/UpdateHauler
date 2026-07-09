use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct SnapPlugin;

#[async_trait]
impl Plugin for SnapPlugin {
    fn name(&self) -> &str {
        "snap"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "snap".to_string(),
            description: "Update Snap packages".to_string(),
            actions: vec![PluginAction {
                name: "snap".to_string(),
                description: "Refresh all Snap packages".to_string(),
                action_type: Some(PluginActionType::Update),
            }],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_snap
    }

    async fn update(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        super::run_cmd(config, logger, true, "snap", &["refresh"])?;
        Ok(())
    }
}
