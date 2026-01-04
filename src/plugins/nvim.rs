use async_trait::async_trait;
use duct::cmd;
use std::path::PathBuf;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;
use which::which;

pub struct NvimPlugin;

impl NvimPlugin {
    fn get_metadata_internal(&self) -> PluginMetadata {
        PluginMetadata {
            name: "nvim".to_string(),
            description: "Update Neovim plugins (supports lazy.nvim, packer.nvim, vim-plug)"
                .to_string(),
            actions: vec![
                PluginAction {
                    name: "nvim".to_string(),
                    description:
                        "Update Neovim plugins (supports lazy.nvim, packer.nvim, vim-plug)"
                            .to_string(),
                    action_type: Some(PluginActionType::Update),
                },
                PluginAction {
                    name: "nvim-save".to_string(),
                    description:
                        "Save nvim plugin configuration (plugins are defined in your config)"
                            .to_string(),
                    action_type: Some(PluginActionType::Save),
                },
                PluginAction {
                    name: "nvim-restore".to_string(),
                    description: "Restore nvim plugins".to_string(),
                    action_type: Some(PluginActionType::Restore),
                },
            ],
        }
    }

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

        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};
        use std::sync::mpsc;
        use std::thread;

        let mut child = Command::new(command)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", cmd_str, e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let (tx, rx) = mpsc::channel();

        let tx_stdout = tx.clone();
        let tx_stderr = tx.clone();

        let short_cmd1 = short_cmd.to_string();
        let short_cmd2 = short_cmd.to_string();
        let show_header = config.show_header;

        let tx_stdout_clone = tx_stdout.clone();
        let tx_stderr_clone = tx_stderr.clone();

        thread::spawn(move || {
            let reader = stdout_reader;
            for line_result in reader.lines() {
                match line_result {
                    Ok(line) => {
                        let formatted = if show_header {
                            format!("{} → {}", short_cmd1, line)
                        } else {
                            line
                        };
                        let _ = tx_stdout_clone.send((formatted, false));
                    }
                    Err(_) => break,
                }
            }
        });

        thread::spawn(move || {
            let reader = stderr_reader;
            for line_result in reader.lines() {
                match line_result {
                    Ok(line) => {
                        let formatted = if show_header {
                            format!("{} → {}", short_cmd2, line)
                        } else {
                            line
                        };
                        let _ = tx_stderr_clone.send((formatted, true));
                    }
                    Err(_) => break,
                }
            }
        });

        for (line, is_error) in rx {
            if is_error {
                logger.error(&line);
            } else {
                logger.log(&line);
            }
        }

        drop(tx);
        drop(tx_stdout);
        drop(tx_stderr);

        let exit_status = child.wait()?;
        let exit_code = exit_status.code().unwrap_or(1);

        if config.show_header {
            if show_error && exit_code != 0 {
                logger.error(&format!("{} → Return code {}", cmd_str, exit_code));
            } else {
                logger.log(&format!("{} → Return code {}", cmd_str, exit_code));
            }
        }

        Ok(())
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

    fn update_lazy_nvim(config: &Config, logger: &mut Logger) -> Result<()> {
        Self::run_cmd(
            config,
            logger,
            false,
            "nvim",
            &["--headless", "-c", "Lazy! sync", "+qa"],
        )
    }

    fn update_packer_nvim(config: &Config, logger: &mut Logger) -> Result<()> {
        Self::run_cmd(
            config,
            logger,
            false,
            "nvim",
            &["--headless", "-c", "PackerSync", "+qa"],
        )
    }

    fn update_vim_plug(config: &Config, logger: &mut Logger) -> Result<()> {
        Self::run_cmd(
            config,
            logger,
            false,
            "nvim",
            &["--headless", "-c", "PlugUpdate --sync", "+qa"],
        )
    }

    fn save_lazy_nvim(_config: &Config, logger: &mut Logger) -> Result<()> {
        logger.log("lazy.nvim plugins are defined in your lazy.nvim configuration");
        Ok(())
    }

    fn save_packer_nvim(_config: &Config, logger: &mut Logger) -> Result<()> {
        logger.log("packer.nvim plugins are defined in your packer.nvim configuration");
        Ok(())
    }

    fn save_vim_plug(_config: &Config, logger: &mut Logger) -> Result<()> {
        logger.log("vim-plug plugins are defined in your vim-plug configuration");
        Ok(())
    }

    fn restore_lazy_nvim(config: &Config, logger: &mut Logger) -> Result<()> {
        let lazy_install_cmd = "nvim --headless '+Lazy! sync' +qa";
        Self::run_cmd(config, logger, false, "nvim", &["-c", lazy_install_cmd])
    }

    fn restore_packer_nvim(config: &Config, logger: &mut Logger) -> Result<()> {
        let packer_install_cmd = "nvim --headless '+PackerInstall' +qa";
        Self::run_cmd(config, logger, false, "nvim", &["-c", packer_install_cmd])
    }

    fn restore_vim_plug(config: &Config, logger: &mut Logger) -> Result<()> {
        let plug_install_cmd = "nvim --headless '+PlugInstall --sync' +qa";
        Self::run_cmd(config, logger, false, "nvim", &["-c", plug_install_cmd])
    }
}

#[async_trait]
impl Plugin for NvimPlugin {
    fn name(&self) -> &str {
        "nvim"
    }

    fn get_metadata(&self) -> PluginMetadata {
        self.get_metadata_internal()
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
            Some("lazy.nvim") => Self::update_lazy_nvim(config, logger)?,
            Some("packer.nvim") => Self::update_packer_nvim(config, logger)?,
            Some("vim-plug") => Self::update_vim_plug(config, logger)?,
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
            Some("lazy.nvim") => Self::save_lazy_nvim(config, logger)?,
            Some("packer.nvim") => Self::save_packer_nvim(config, logger)?,
            Some("vim-plug") => Self::save_vim_plug(config, logger)?,
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
            Some("lazy.nvim") => Self::restore_lazy_nvim(config, logger)?,
            Some("packer.nvim") => Self::restore_packer_nvim(config, logger)?,
            Some("vim-plug") => Self::restore_vim_plug(config, logger)?,
            _ => {
                logger.log("No supported nvim plugin manager detected");
            }
        }

        Ok(())
    }
}
