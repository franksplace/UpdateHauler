use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct RustupPlugin;

#[async_trait]
impl Plugin for RustupPlugin {
    fn name(&self) -> &str {
        "rustup"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "rustup".to_string(),
            description: "Update Rust toolchains via rustup update".to_string(),
            actions: vec![PluginAction {
                name: "rustup".to_string(),
                description: "Update Rust toolchains".to_string(),
                action_type: Some(PluginActionType::Update),
            }],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_rustup
    }

    async fn update(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        super::run_cmd(config, logger, true, "rustup", &["update"])?;
        Ok(())
    }
}
