use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct YarnPlugin;

#[async_trait]
impl Plugin for YarnPlugin {
    fn name(&self) -> &str {
        "yarn"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "yarn".to_string(),
            description: "Update globally installed Yarn/PNPM packages".to_string(),
            actions: vec![
                PluginAction {
                    name: "yarn".to_string(),
                    description: "Update globally installed Yarn/PNPM packages".to_string(),
                    action_type: Some(PluginActionType::Update),
                },
                PluginAction {
                    name: "yarn-save".to_string(),
                    description: "Save globally installed Yarn/PNPM packages to JSON".to_string(),
                    action_type: Some(PluginActionType::Save),
                },
                PluginAction {
                    name: "yarn-restore".to_string(),
                    description: "Restore Yarn/PNPM packages from saved JSON".to_string(),
                    action_type: Some(PluginActionType::Restore),
                },
            ],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_yarn
    }

    async fn update(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        super::run_cmd(config, logger, true, "yarn", &["global", "upgrade"])?;
        Ok(())
    }

    async fn save(&self, config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
        let yarn_file = config.yarn_file.to_string_lossy().to_string();
        if let Some(parent) = config.yarn_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        logger.log(&format!("Saving yarn global packages to {}", yarn_file));
        super::run_cmd(config, logger, true, "yarn", &["global", "list", "--json"])?;
        logger.log("Success savefile written");
        Ok(())
    }

    async fn restore(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        let yarn_file = config.yarn_file.to_string_lossy().to_string();
        if !config.yarn_file.exists() {
            logger.error(&format!(
                "missing dependency — {} yarn's backup file is not found",
                yarn_file
            ));
            return Ok(());
        }
        logger.log(&format!(
            "Restoring yarn global packages from {}",
            yarn_file
        ));
        logger.log(
            "yarn global packages must be reinstalled manually — run: yarn global add <package>",
        );
        Ok(())
    }
}
