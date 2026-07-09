use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct FlatpakPlugin;

#[async_trait]
impl Plugin for FlatpakPlugin {
    fn name(&self) -> &str {
        "flatpak"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "flatpak".to_string(),
            description: "Update Flatpak applications".to_string(),
            actions: vec![PluginAction {
                name: "flatpak".to_string(),
                description: "Update all Flatpak applications".to_string(),
                action_type: Some(PluginActionType::Update),
            }],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_flatpak
    }

    async fn update(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        super::run_cmd(config, logger, true, "flatpak", &["update", "-y"])?;
        Ok(())
    }
}
