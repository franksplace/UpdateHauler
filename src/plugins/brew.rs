use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct BrewPlugin;

#[async_trait]
impl Plugin for BrewPlugin {
    fn name(&self) -> &str {
        "brew"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "brew".to_string(),
            description: "Update, upgrade, and clean brew formulas and casks".to_string(),
            actions: vec![
                PluginAction {
                    name: "brew".to_string(),
                    description: "Update, upgrade, and clean brew formulas and casks".to_string(),
                    action_type: Some(PluginActionType::Update),
                },
                PluginAction {
                    name: "brew-save".to_string(),
                    description: "Save the brew bundle to Brewfile".to_string(),
                    action_type: Some(PluginActionType::Save),
                },
                PluginAction {
                    name: "brew-restore".to_string(),
                    description: "Restore from the brew bundle".to_string(),
                    action_type: Some(PluginActionType::Restore),
                },
                PluginAction {
                    name: "brew-list".to_string(),
                    description: "List all installed brew formulas".to_string(),
                    action_type: None,
                },
                PluginAction {
                    name: "brew-outdated".to_string(),
                    description: "Show outdated brew formulas and casks".to_string(),
                    action_type: None,
                },
                PluginAction {
                    name: "brew-upgrade-pinned".to_string(),
                    description: "Upgrade only pinned brew formulas".to_string(),
                    action_type: None,
                },
                PluginAction {
                    name: "brew-info".to_string(),
                    description: "Show information about a brew formula or cask".to_string(),
                    action_type: None,
                },
                PluginAction {
                    name: "brew-search".to_string(),
                    description: "Search for brew formulas and casks".to_string(),
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
            "brew-list" => {
                super::run_cmd(config, logger, true, "brew", &["list"])?;
                Ok(true)
            }
            "brew-outdated" => {
                super::run_cmd(config, logger, true, "brew", &["outdated"])?;
                Ok(true)
            }
            "brew-upgrade-pinned" => {
                super::run_cmd(config, logger, true, "brew", &["upgrade", "--pinned"])?;
                Ok(true)
            }
            "brew-info" => {
                logger.log("Usage: updatehauler run --cmd \"brew info <formula>\"");
                logger.log("Or directly: brew info <formula>");
                Ok(true)
            }
            "brew-search" => {
                logger.log("Usage: updatehauler run --cmd \"brew search <term>\"");
                logger.log("Or directly: brew search <term>");
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_brew
    }

    async fn update(
        &self,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        if !insights.has_brew {
            return Ok(());
        }

        super::run_cmd(config, logger, true, "brew", &["update"])?;
        super::run_cmd(config, logger, true, "brew", &["upgrade"])?;
        super::run_cmd(config, logger, true, "brew", &["cleanup", "-q"])?;
        super::run_cmd(config, logger, false, "brew", &["doctor", "-q"])?;
        super::run_cmd(config, logger, true, "brew", &["upgrade", "--cask"])?;
        if duct::cmd("brew", &["cu", "--version"])
            .stdout_null()
            .stderr_null()
            .run()
            .is_ok()
        {
            super::run_cmd(
                config,
                logger,
                true,
                "brew",
                &["cu", "-a", "-f", "--cleanup", "-y"],
            )?;
        }
        super::run_cmd(config, logger, true, "brew", &["cleanup", "-q"])?;
        super::run_cmd(config, logger, false, "brew", &["doctor", "--verbose"])?;

        Ok(())
    }

    async fn save(&self, config: &Config, insights: &Insights, logger: &mut Logger) -> Result<()> {
        if !insights.has_brew {
            return Ok(());
        }

        let brew_file = config.brew_file.to_string_lossy().to_string();

        if let Some(parent) = config.brew_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        logger.log(&format!("Generating brew's {} save file", brew_file));

        super::run_cmd(
            config,
            logger,
            true,
            "brew",
            &["bundle", "dump", "--force", "--file", &brew_file],
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
        if !insights.has_brew {
            return Ok(());
        }

        let brew_file = config.brew_file.to_string_lossy().to_string();

        if !config.brew_file.exists() {
            logger.error(&format!(
                "missing dependency — {} brew's backup file is not found",
                brew_file
            ));
            return Ok(());
        }

        super::run_cmd(
            config,
            logger,
            true,
            "brew",
            &["bundle", "--file", &brew_file],
        )?;

        Ok(())
    }
}
