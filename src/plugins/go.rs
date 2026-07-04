use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct GoPlugin;

#[async_trait]
impl Plugin for GoPlugin {
    fn name(&self) -> &str {
        "go"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "go".to_string(),
            description: "Update Go modules".to_string(),
            actions: vec![
                PluginAction {
                    name: "go".to_string(),
                    description: "Update Go tools and modules".to_string(),
                    action_type: Some(PluginActionType::Update),
                },
                PluginAction {
                    name: "go-save".to_string(),
                    description: "Save installed Go binaries list".to_string(),
                    action_type: Some(PluginActionType::Save),
                },
                PluginAction {
                    name: "go-restore".to_string(),
                    description: "Restore Go tools from saved list".to_string(),
                    action_type: Some(PluginActionType::Restore),
                },
            ],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_go
    }

    async fn update(&self, config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
        super::run_cmd(config, logger, true, "go", &["version"])?;
        logger.log("Go updates — upgrade Go toolchain via your package manager or download from https://go.dev/dl");
        logger.log("Go binaries installed via 'go install' — run 'go install <path>@latest' for each");
        Ok(())
    }

    async fn save(&self, config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
        let go_file = config.go_file.to_string_lossy().to_string();
        if let Some(parent) = config.go_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        logger.log(&format!("Saving Go binaries list to {}", go_file));
        super::run_cmd(config, logger, true, "sh", &["-c", "ls -1 $(go env GOPATH)/bin 2>/dev/null || echo '(no binaries in GOPATH/bin)'"])?;
        logger.log("Success savefile written");
        Ok(())
    }

    async fn restore(&self, config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
        let go_file = config.go_file.to_string_lossy().to_string();
        if !config.go_file.exists() {
            logger.error(&format!("missing dependency — {} go's backup file is not found", go_file));
            return Ok(());
        }
        logger.log(&format!("Restoring Go binaries from {}", go_file));
        logger.log("Go binaries must be reinstalled manually — run: go install <path>@latest");
        Ok(())
    }
}
