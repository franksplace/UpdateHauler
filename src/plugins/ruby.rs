use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct RubyPlugin;

#[async_trait]
impl Plugin for RubyPlugin {
    fn name(&self) -> &str {
        "ruby"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "ruby".to_string(),
            description: "Update Ruby gems".to_string(),
            actions: vec![
                PluginAction {
                    name: "ruby".to_string(),
                    description: "Update all installed Ruby gems".to_string(),
                    action_type: Some(PluginActionType::Update),
                },
                PluginAction {
                    name: "ruby-save".to_string(),
                    description: "Save installed Ruby gems list to file".to_string(),
                    action_type: Some(PluginActionType::Save),
                },
                PluginAction {
                    name: "ruby-restore".to_string(),
                    description: "Restore Ruby gems from saved list".to_string(),
                    action_type: Some(PluginActionType::Restore),
                },
            ],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_gem
    }

    async fn update(&self, config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
        super::run_cmd(config, logger, true, "gem", &["update", "--system"])?;
        super::run_cmd(config, logger, true, "gem", &["update"])?;
        super::run_cmd(config, logger, true, "gem", &["cleanup"])?;
        Ok(())
    }

    async fn save(&self, config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
        let gem_file = config.gem_file.to_string_lossy().to_string();
        if let Some(parent) = config.gem_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        logger.log(&format!("Saving Ruby gems list to {}", gem_file));
        super::run_cmd(config, logger, true, "gem", &["list", "--local"])?;
        logger.log("Success savefile written");
        Ok(())
    }

    async fn restore(&self, config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
        let gem_file = config.gem_file.to_string_lossy().to_string();
        if !config.gem_file.exists() {
            logger.error(&format!("missing dependency — {} gem's backup file is not found", gem_file));
            return Ok(());
        }
        logger.log(&format!("Restoring Ruby gems from {}", gem_file));
        logger.log("Ruby gems must be reinstalled manually — run: gem install <gemname>");
        Ok(())
    }
}
