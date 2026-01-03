use async_trait::async_trait;
use duct::cmd;
use std::path::PathBuf;

use super::Plugin;
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;
use which::which;

pub struct NvimPlugin;

impl NvimPlugin {
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

    fn get_nvim_config_path() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".config/nvim"))
    }

    fn detect_plugin_manager() -> Option<String> {
        let nvim_config = Self::get_nvim_config_path()?;
        let lazy_lock = nvim_config.join("lazy-lock.json");
        let packer_lock = nvim_config.join("packer_compiled.lua");
        let plug_vim = nvim_config.join("autoload/plug.vim");

        if lazy_lock.exists() {
            Some("lazy.nvim".to_string())
        } else if packer_lock.exists() {
            Some("packer.nvim".to_string())
        } else if plug_vim.exists() {
            Some("vim-plug".to_string())
        } else {
            None
        }
    }

    async fn update_lazy_nvim(config: &Config, logger: &mut Logger) -> Result<()> {
        let lazy_update_cmd = "nvim --headless '+Lazy! sync' +qa";
        let args = vec!["-c", lazy_update_cmd];

        logger.log("Updating lazy.nvim plugins");
        Self::run_cmd(config, logger, true, "nvim", &args)?;

        Ok(())
    }

    async fn update_packer_nvim(config: &Config, logger: &mut Logger) -> Result<()> {
        let packer_update_cmd = "nvim --headless '+PackerSync' +qa";
        let args = vec!["-c", packer_update_cmd];

        logger.log("Updating packer.nvim plugins");
        Self::run_cmd(config, logger, true, "nvim", &args)?;

        Ok(())
    }

    async fn update_vim_plug(config: &Config, logger: &mut Logger) -> Result<()> {
        let plug_update_cmd = "nvim --headless '+PlugUpdate --sync' +qa";
        let args = vec!["-c", plug_update_cmd];

        logger.log("Updating vim-plug plugins");
        Self::run_cmd(config, logger, true, "nvim", &args)?;

        Ok(())
    }

    async fn save_lazy_nvim(_config: &Config, logger: &mut Logger) -> Result<()> {
        logger.log("lazy.nvim plugins are defined in your lazy.nvim configuration");
        Ok(())
    }

    async fn save_packer_nvim(_config: &Config, logger: &mut Logger) -> Result<()> {
        logger.log("packer.nvim plugins are defined in your packer.nvim configuration");
        Ok(())
    }

    async fn save_vim_plug(_config: &Config, logger: &mut Logger) -> Result<()> {
        logger.log("vim-plug plugins are defined in your vim-plug configuration");
        Ok(())
    }

    async fn restore_lazy_nvim(config: &Config, logger: &mut Logger) -> Result<()> {
        let lazy_install_cmd = "nvim --headless '+Lazy! sync' +qa";
        let args = vec!["-c", lazy_install_cmd];

        logger.log("Restoring lazy.nvim plugins");
        Self::run_cmd(config, logger, true, "nvim", &args)?;

        Ok(())
    }

    async fn restore_packer_nvim(config: &Config, logger: &mut Logger) -> Result<()> {
        let packer_install_cmd = "nvim --headless '+PackerInstall' +qa";
        let args = vec!["-c", packer_install_cmd];

        logger.log("Restoring packer.nvim plugins");
        Self::run_cmd(config, logger, true, "nvim", &args)?;

        Ok(())
    }

    async fn restore_vim_plug(config: &Config, logger: &mut Logger) -> Result<()> {
        let plug_install_cmd = "nvim --headless '+PlugInstall --sync' +qa";
        let args = vec!["-c", plug_install_cmd];

        logger.log("Restoring vim-plug plugins");
        Self::run_cmd(config, logger, true, "nvim", &args)?;

        Ok(())
    }
}

#[async_trait]
impl Plugin for NvimPlugin {
    fn name(&self) -> &str {
        "nvim"
    }

    async fn check_available(&self, _config: &Config, _insights: &Insights) -> bool {
        which("nvim").is_ok() && Self::detect_plugin_manager().is_some()
    }

    async fn update(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        let plugin_manager = Self::detect_plugin_manager();

        match plugin_manager.as_deref() {
            Some("lazy.nvim") => Self::update_lazy_nvim(config, logger).await?,
            Some("packer.nvim") => Self::update_packer_nvim(config, logger).await?,
            Some("vim-plug") => Self::update_vim_plug(config, logger).await?,
            _ => {
                logger.log("No supported nvim plugin manager detected (lazy.nvim, packer.nvim, or vim-plug)");
                return Ok(());
            }
        }

        Ok(())
    }

    async fn save(&self, config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
        let plugin_manager = Self::detect_plugin_manager();

        match plugin_manager.as_deref() {
            Some("lazy.nvim") => Self::save_lazy_nvim(config, logger).await?,
            Some("packer.nvim") => Self::save_packer_nvim(config, logger).await?,
            Some("vim-plug") => Self::save_vim_plug(config, logger).await?,
            _ => {
                logger.log("No supported nvim plugin manager detected");
            }
        }

        Ok(())
    }

    async fn restore(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        let plugin_manager = Self::detect_plugin_manager();

        match plugin_manager.as_deref() {
            Some("lazy.nvim") => Self::restore_lazy_nvim(config, logger).await?,
            Some("packer.nvim") => Self::restore_packer_nvim(config, logger).await?,
            Some("vim-plug") => Self::restore_vim_plug(config, logger).await?,
            _ => {
                logger.log("No supported nvim plugin manager detected");
            }
        }

        Ok(())
    }
}
