use async_trait::async_trait;
use duct::cmd;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct CargoPlugin;

#[async_trait]
impl Plugin for CargoPlugin {
    fn name(&self) -> &str {
        "cargo"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "cargo".to_string(),
            description: "Upgrade cargo installed packages (requires cargo-install-update)"
                .to_string(),
            actions: vec![
                PluginAction {
                    name: "cargo".to_string(),
                    description: "Upgrade cargo installed packages (requires cargo-install-update)"
                        .to_string(),
                    action_type: Some(PluginActionType::Update),
                },
                PluginAction {
                    name: "cargo-save".to_string(),
                    description: "Save cargo packages to backup JSON (requires cargo-backup)"
                        .to_string(),
                    action_type: Some(PluginActionType::Save),
                },
                PluginAction {
                    name: "cargo-restore".to_string(),
                    description: "Restore cargo packages from backup JSON (requires cargo-restore)"
                        .to_string(),
                    action_type: Some(PluginActionType::Restore),
                },
                PluginAction {
                    name: "cargo-list".to_string(),
                    description: "List all installed cargo packages".to_string(),
                    action_type: None,
                },
                PluginAction {
                    name: "cargo-outdated".to_string(),
                    description: "Show outdated cargo packages (requires cargo-outdated)"
                        .to_string(),
                    action_type: None,
                },
            ],
        }
    }

    async fn handle_custom_action(
        &self,
        action_name: &str,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<bool> {
        match action_name {
            "cargo-list" => {
                super::run_cmd(config, logger, true, "cargo", &["install", "--list"])?;
                Ok(true)
            }
            "cargo-outdated" => {
                super::run_cmd(config, logger, false, "cargo", &["outdated"])?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_cargo
    }

    async fn update(
        &self,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        if !insights.has_cargo {
            return Ok(());
        }

        let check = cmd("cargo", &["install-update", "--version"]).run();
        if check.is_err() {
            logger.error("cargo-install-update not found — install it with: cargo install cargo-install-update");
            return Ok(());
        }

        super::run_cmd(config, logger, true, "cargo", &["install-update", "-a"])?;

        Ok(())
    }

    async fn save(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()> {
        if !insights.has_cargo {
            return Ok(());
        }

        let cargo_file = config.cargo_file.to_string_lossy().to_string();

        if let Some(parent) = config.cargo_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let check = cmd("cargo", &["backup", "--version"]).run();
        if check.is_err() {
            logger.error("cargo-backup not found — install it with: cargo install cargo-backup");
            return Ok(());
        }

        logger.log(&format!("Generating cargo's {} save file", cargo_file));

        super::run_cmd(
            config,
            logger,
            true,
            "cargo",
            &["backup", "-o", &cargo_file],
        )?;

        logger.log("Success savefile written");

        Ok(())
    }

    async fn restore(
        &self,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        if !insights.has_cargo {
            return Ok(());
        }

        let cargo_file = config.cargo_file.to_string_lossy().to_string();

        if !config.cargo_file.exists() {
            logger.error(&format!(
                "missing dependency — {} cargo's backup json file is not found",
                cargo_file
            ));
            return Ok(());
        }

        let check = cmd("cargo", &["restore", "--version"]).run();
        if check.is_err() {
            logger.error("cargo-restore not found — install it with: cargo install cargo-restore");
            return Ok(());
        }

        super::run_cmd(
            config,
            logger,
            true,
            "cargo",
            &[
                "restore",
                "--yes",
                "--skip-update",
                "--skip-remove",
                "--backup",
                &cargo_file,
            ],
        )?;

        Ok(())
    }
}
