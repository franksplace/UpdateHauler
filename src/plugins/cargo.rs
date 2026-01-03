use async_trait::async_trait;
use duct::cmd;

use super::Plugin;
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct CargoPlugin;

impl CargoPlugin {
    fn run_cmd(
        config: &Config,
        logger: &mut Logger,
        show_error: bool,
        command: &str,
        args: &[&str],
    ) -> Result<()> {
        let cmd_str = format!("{} {}", command, args.join(" "));

        let short_cmd = command;

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
impl Plugin for CargoPlugin {
    fn name(&self) -> &str {
        "cargo"
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

        let _ = cmd("cargo", &["install-update", "--version"])
            .stdout_null()
            .run();

        Self::run_cmd(config, logger, true, "cargo", &["install-update", "-a"])?;

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

        let _ = cmd("cargo", &["backup", "--version"]).stdout_null().run();

        logger.log(&format!("Generating cargo's {} save file", cargo_file));

        Self::run_cmd(
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

        let _ = cmd("cargo", &["restore", "--version"]).stdout_null().run();

        Self::run_cmd(
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
