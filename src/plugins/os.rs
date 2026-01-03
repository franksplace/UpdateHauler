use async_trait::async_trait;
use duct::cmd;

use super::Plugin;
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct OsPlugin;

impl OsPlugin {
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
impl Plugin for OsPlugin {
    fn name(&self) -> &str {
        "os"
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.is_darwin || insights.is_linux
    }

    async fn update(
        &self,
        config: &Config,
        insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        if insights.is_darwin {
            let softwareupdate_result = Self::run_cmd(
                config,
                logger,
                false,
                "sudo",
                &["softwareupdate", "-a", "-i", "--verbose"],
            );

            if softwareupdate_result.is_err() {
                Self::run_cmd(
                    config,
                    logger,
                    true,
                    "softwareupdate",
                    &["-a", "-i", "--verbose"],
                )?;
            }

            let mas_result = cmd("mas", &["version"]).stdout_null().run();
            if mas_result.is_ok() {
                Self::run_cmd(config, logger, true, "mas", &["update"])?;
            }

            return Ok(());
        }

        if insights.is_linux {
            if let Some(ref pkg_mgr) = insights.pkg_mgr {
                let commands = match pkg_mgr.as_str() {
                    "dnf" => vec![
                        vec!["dnf", "-y", "update"],
                        vec!["dnf", "-y", "upgrade"],
                        vec!["dnf", "-y", "update"],
                    ],
                    "apt-get" => vec![
                        vec!["apt-get", "-y", "update"],
                        vec!["apt-get", "-y", "upgrade"],
                        vec!["apt-get", "-y", "update"],
                    ],
                    "apk" => vec![
                        vec!["apk", "update"],
                        vec!["sh", "-c", "yes | apk -U upgrade"],
                        vec!["apk", "update"],
                    ],
                    "nix-env" => vec![vec!["nix-channel", "--update"], vec!["nix-env", "-u", "*"]],
                    "arch" => vec![vec!["pacman", "-Syu", "--noconfirm"]],
                    _ => {
                        logger.error("OS not supported for updates");
                        return Ok(());
                    }
                };

                for cmd_args in commands {
                    if insights.is_root {
                        let (program, args) = cmd_args.split_first().unwrap();
                        let args: Vec<&str> = args.to_vec();
                        Self::run_cmd(config, logger, true, program, &args)?;
                    } else {
                        let shell_cmd = cmd_args.join(" ");
                        Self::run_cmd(config, logger, true, "sudo", &["sh", "-c", &shell_cmd])?;
                    }
                }
            } else {
                logger.error("OS not supported for updates");
            }
        }

        Ok(())
    }

    async fn save(
        &self,
        _config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        logger.log("OS packages are managed by the system package manager - no save needed");
        Ok(())
    }

    async fn restore(
        &self,
        _config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        logger.log("OS packages are managed by the system package manager - no restore needed");
        Ok(())
    }
}
