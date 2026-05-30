use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct UvPlugin;

#[async_trait]
impl Plugin for UvPlugin {
    fn name(&self) -> &str {
        "uv"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "uv".to_string(),
            description: "Update uv and manage uv-installed tools".to_string(),
            actions: vec![
                PluginAction {
                    name: "uv".to_string(),
                    description: "Update uv itself and upgrade all installed tools".to_string(),
                    action_type: Some(PluginActionType::Update),
                },
                PluginAction {
                    name: "uv-save".to_string(),
                    description: "Save installed uv tools list to JSON".to_string(),
                    action_type: Some(PluginActionType::Save),
                },
                PluginAction {
                    name: "uv-restore".to_string(),
                    description: "Restore uv tools from saved JSON".to_string(),
                    action_type: Some(PluginActionType::Restore),
                },
                PluginAction {
                    name: "uv-list".to_string(),
                    description: "List installed uv tools".to_string(),
                    action_type: None,
                },
                PluginAction {
                    name: "uvx".to_string(),
                    description:
                        "Run a tool with uvx (use: updatehauler run --cmd \"uvx <tool> [args]\")"
                            .to_string(),
                    action_type: None,
                },
            ],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_uv
    }

    async fn handle_custom_action(
        &self,
        action_name: &str,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<bool> {
        match action_name {
            "uv-list" => {
                super::run_cmd(config, logger, true, "uv", &["tool", "list"])?;
                Ok(true)
            }
            "uvx" => {
                logger.log(
                    "Use: updatehauler run --cmd \"uvx <tool> [args]\" to run a tool with uvx",
                );
                logger.log("Or directly:  uvx <tool> [args]");
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    async fn update(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        super::run_cmd(config, logger, true, "uv", &["tool", "upgrade", "--all"])?;
        Ok(())
    }

    async fn save(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()> {
        let uv_file = config.uv_file.to_string_lossy().to_string();

        if let Some(parent) = config.uv_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        logger.log(&format!("Saving uv tools list to {}", uv_file));

        super::run_cmd(config, logger, true, "uv", &["tool", "list"])?;

        self.update(config, insights, logger).await?;

        logger.log("Success savefile written");

        Ok(())
    }

    async fn restore(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        let uv_file = config.uv_file.to_string_lossy().to_string();

        if !config.uv_file.exists() {
            logger.error(&format!(
                "missing dependency — {} uv's backup file is not found",
                uv_file
            ));
            return Ok(());
        }

        logger.log(&format!("Restoring uv tools from {}", uv_file));
        logger.log("uv tools are installed on demand — re-run 'uv tool install' for needed tools");

        Ok(())
    }
}
