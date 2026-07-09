use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;
use duct::cmd;

pub struct NpmPlugin;

#[async_trait]
impl Plugin for NpmPlugin {
    fn name(&self) -> &str {
        "npm"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "npm".to_string(),
            description: "Update globally installed npm packages".to_string(),
            actions: vec![
                PluginAction {
                    name: "npm".to_string(),
                    description: "Update globally installed npm packages".to_string(),
                    action_type: Some(PluginActionType::Update),
                },
                PluginAction {
                    name: "npm-save".to_string(),
                    description: "Save globally installed npm packages to JSON".to_string(),
                    action_type: Some(PluginActionType::Save),
                },
                PluginAction {
                    name: "npm-restore".to_string(),
                    description: "Restore globally installed npm packages from JSON".to_string(),
                    action_type: Some(PluginActionType::Restore),
                },
            ],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_npm
    }

    async fn update(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        super::run_cmd(config, logger, true, "npm", &["update", "-g"])?;
        Ok(())
    }

    async fn save(&self, config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
        let npm_file = config.npm_file.to_string_lossy().to_string();

        if let Some(parent) = config.npm_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        logger.log(&format!("Saving npm global packages to {}", npm_file));

        let output = cmd("npm", &["list", "-g", "--depth=0", "--json"])
            .stdout_capture()
            .stderr_capture()
            .run()?;

        if !output.stdout.is_empty() {
            std::fs::write(&npm_file, &output.stdout)?;
        }
        if !output.stderr.is_empty() {
            logger.log(&String::from_utf8_lossy(&output.stderr));
        }

        logger.log("Success savefile written");

        Ok(())
    }

    async fn restore(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        let npm_file = config.npm_file.to_string_lossy().to_string();

        if !config.npm_file.exists() {
            logger.error(&format!(
                "missing dependency — {} npm's backup file is not found",
                npm_file
            ));
            return Ok(());
        }

        logger.log(&format!("Restoring npm global packages from {}", npm_file));

        let content = std::fs::read_to_string(&npm_file)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
            let packages: Vec<&str> = deps.keys().map(|k| k.as_str()).collect();
            if !packages.is_empty() {
                let mut args: Vec<&str> = vec!["install", "-g"];
                args.extend(packages);
                super::run_cmd(config, logger, true, "npm", &args)?;
            } else {
                logger.log("No packages found in save file");
            }
        } else {
            logger.log("No dependencies found in save file");
        }

        Ok(())
    }
}
