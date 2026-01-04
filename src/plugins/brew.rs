use async_trait::async_trait;
use duct::cmd;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct BrewPlugin;

impl BrewPlugin {
    fn run_cmd(
        config: &Config,
        logger: &mut Logger,
        show_error: bool,
        command: &str,
        args: &[&str],
    ) -> Result<()> {
        let cmd_str = format!("{} {}", command, args.join(" "));

        let short_cmd = if command == "sudo" && args.len() >= 4 {
            args[3]
        } else {
            command
        };

        if config.dry_run {
            if config.show_header {
                logger.log(&format!("{} → Start (DRY-RUN)", cmd_str));
            }
            logger.log(&format!("Would execute: {}", cmd_str));
            if config.show_header {
                logger.log(&format!("{} → Return code 0 (DRY-RUN)", cmd_str));
            }
            return Ok(());
        }

        if config.show_header {
            logger.log(&format!("{} → Start", cmd_str));
        }

        let result = cmd(command, args).stdout_capture().stderr_capture().run();

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                for line in stdout.lines() {
                    let formatted = if config.show_header {
                        format!("{} → {}", short_cmd, line)
                    } else {
                        line.to_string()
                    };
                    logger.log(&formatted);
                }

                for line in stderr.lines() {
                    let formatted = if config.show_header {
                        format!("{} → {}", short_cmd, line)
                    } else {
                        line.to_string()
                    };
                    logger.log(&formatted);
                }

                if config.show_header {
                    if show_error && !output.status.success() {
                        logger.error(&format!(
                            "{} → Return code {}",
                            cmd_str,
                            output.status.code().unwrap_or(1)
                        ));
                    } else {
                        logger.log(&format!(
                            "{} → Return code {}",
                            cmd_str,
                            output.status.code().unwrap_or(0)
                        ));
                    }
                }

                Ok(())
            }
            Err(e) => {
                if config.show_header {
                    if show_error {
                        logger.error(&format!("{} → Error: {}", cmd_str, e));
                    } else {
                        logger.log(&format!("{} → Return code {}", cmd_str, 1));
                    }
                }
                Ok(())
            }
        }
    }
}

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
            ],
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

        Self::run_cmd(config, logger, true, "brew", &["update"])?;
        Self::run_cmd(config, logger, true, "brew", &["upgrade"])?;
        Self::run_cmd(config, logger, true, "brew", &["cleanup", "-q"])?;
        Self::run_cmd(config, logger, false, "brew", &["doctor", "-q"])?;
        Self::run_cmd(config, logger, true, "brew", &["upgrade", "--cask"])?;
        Self::run_cmd(
            config,
            logger,
            true,
            "brew",
            &["cu", "-a", "-f", "--cleanup", "-y"],
        )?;
        Self::run_cmd(config, logger, true, "brew", &["cleanup", "-q"])?;
        Self::run_cmd(config, logger, false, "brew", &["doctor", "--verbose"])?;

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

        Self::run_cmd(
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

        Self::run_cmd(
            config,
            logger,
            true,
            "brew",
            &["bundle", "--file", &brew_file],
        )?;

        Ok(())
    }
}
