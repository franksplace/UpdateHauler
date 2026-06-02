use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct VscodePlugin;

#[async_trait]
impl Plugin for VscodePlugin {
    fn name(&self) -> &str {
        "vscode"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "vscode".to_string(),
            description: "Update VSCode/Cursor extensions".to_string(),
            actions: vec![PluginAction {
                name: "vscode".to_string(),
                description: "Update all VSCode/Cursor extensions".to_string(),
                action_type: Some(PluginActionType::Update),
            }],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_vscode
    }

    async fn update(
        &self,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        if let Some(ref editor) = insights.vscode_bin {
            super::run_cmd(config, logger, true, editor, &["--update-extensions"])?;
        }
        Ok(())
    }

    async fn save(
        &self,
        _config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        logger.log("VSCode extensions are managed by the editor - no save needed");
        Ok(())
    }

    async fn restore(
        &self,
        _config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        logger.log("VSCode extensions are managed by the editor - no restore needed");
        Ok(())
    }
}
