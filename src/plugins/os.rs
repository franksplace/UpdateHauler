use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct OsPlugin;

#[async_trait]
impl Plugin for OsPlugin {
    fn name(&self) -> &str {
        "os"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "os".to_string(),
            description: "Update OS & app based packages".to_string(),
            actions: vec![PluginAction {
                name: "os".to_string(),
                description: "Update OS & app based packages".to_string(),
                action_type: Some(PluginActionType::Update),
            }],
        }
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
            let softwareupdate_result = super::run_cmd(
                config,
                logger,
                false,
                "/usr/bin/sudo",
                &["/usr/sbin/softwareupdate", "-a", "-i", "--verbose"],
            );

            if softwareupdate_result.is_err() {
                super::run_cmd(
                    config,
                    logger,
                    true,
                    "/usr/sbin/softwareupdate",
                    &["-a", "-i", "--verbose"],
                )?;
            }

            super::run_cmd(config, logger, true, "mas", &["update"])?;

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
                        vec!["apk", "-y", "upgrade"],
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
                    let (program, args) = cmd_args.split_first().unwrap();
                    if insights.is_root {
                        let args: Vec<&str> = args.to_vec();
                        super::run_cmd(config, logger, true, program, &args)?;
                    } else {
                        let mut sudo_args: Vec<&str> = vec![program];
                        sudo_args.extend(args);
                        super::run_cmd(config, logger, true, "/usr/bin/sudo", &sudo_args)?;
                    }
                }
            } else {
                logger.error("OS not supported for updates");
            }
        }

        Ok(())
    }
}
