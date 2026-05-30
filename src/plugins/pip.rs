use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct PipPlugin;

#[async_trait]
impl Plugin for PipPlugin {
    fn name(&self) -> &str {
        "pip"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "pip".to_string(),
            description: "Update pip packages (auto-detects uv if available)".to_string(),
            actions: vec![
                PluginAction {
                    name: "pip".to_string(),
                    description: "Update pip packages".to_string(),
                    action_type: Some(PluginActionType::Update),
                },
                PluginAction {
                    name: "pip-save".to_string(),
                    description: "Save pip packages to requirements file".to_string(),
                    action_type: Some(PluginActionType::Save),
                },
                PluginAction {
                    name: "pip-restore".to_string(),
                    description: "Restore pip packages from requirements file".to_string(),
                    action_type: Some(PluginActionType::Restore),
                },
            ],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_pip || insights.has_uv
    }

    async fn update(
        &self,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        if insights.has_uv {
            super::run_cmd(config, logger, true, "uv", &["pip", "list", "--outdated"])?;
            super::run_cmd(
                config,
                logger,
                false,
                "sh",
                &[
                    "-c",
                    "uv pip list --outdated --format=columns 2>/dev/null | tail -n +3 | grep -v '^Using' | awk '{print $1}' | xargs -r uv pip install --upgrade --system --break-system-packages 2>&1 || true",
                ],
            )?;
        } else {
            super::run_cmd(
                config,
                logger,
                true,
                "sh",
                &[
                    "-c",
                    "pip list --outdated --format=freeze 2>/dev/null | cut -d= -f1 | xargs pip install --upgrade 2>&1 || true",
                ],
            )?;
        }

        Ok(())
    }

    async fn save(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()> {
        let pip_file = config.pip_file.to_string_lossy().to_string();

        if let Some(parent) = config.pip_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        logger.log(&format!("Saving pip packages to {}", pip_file));
        if insights.has_uv {
            super::run_cmd(config, logger, true, "uv", &["pip", "freeze"])?;
        } else {
            super::run_cmd(config, logger, true, "pip", &["freeze"])?;
        }
        logger.log("Success savefile written");

        Ok(())
    }

    async fn restore(
        &self,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        let pip_file = config.pip_file.to_string_lossy().to_string();

        if !config.pip_file.exists() {
            logger.error(&format!(
                "missing dependency — {} pip's requirements file is not found",
                pip_file
            ));
            return Ok(());
        }

        logger.log(&format!("Restoring pip packages from {}", pip_file));
        if insights.has_uv {
            super::run_cmd(
                config,
                logger,
                true,
                "uv",
                &["pip", "install", "-r", &pip_file],
            )?;
        } else {
            super::run_cmd(config, logger, true, "pip", &["install", "-r", &pip_file])?;
        }

        Ok(())
    }
}
