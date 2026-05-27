use async_trait::async_trait;
use std::path::PathBuf;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;
use duct::cmd;
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
                PluginAction {
                    name: "nvim-list".to_string(),
                    description: "List installed nvim plugins".to_string(),
                    action_type: None,
                },
                PluginAction {
                    name: "nvim-clean".to_string(),
                    description: "Clean unused nvim plugins".to_string(),
                    action_type: None,
                },
                PluginAction {
                    name: "nvim-health".to_string(),
                    description: "Check nvim plugin health".to_string(),
                    action_type: None,
                },
            ],
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

    fn update_lazy_nvim(config: &Config, logger: &mut Logger) -> Result<()> {
        super::run_cmd(
            config,
            logger,
            false,
            "nvim",
            &["--headless", "-c", "Lazy! sync", "+qa"],
        )
    }

    fn update_packer_nvim(config: &Config, logger: &mut Logger) -> Result<()> {
        super::run_cmd(
            config,
            logger,
            false,
            "nvim",
            &["--headless", "-c", "PackerSync", "+qa"],
        )
    }

    fn update_vim_plug(config: &Config, logger: &mut Logger) -> Result<()> {
        super::run_cmd(
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
        super::run_cmd(
            config,
            logger,
            false,
            "nvim",
            &["--headless", "-c", "Lazy! sync", "+qa"],
        )
    }

    fn restore_packer_nvim(config: &Config, logger: &mut Logger) -> Result<()> {
        super::run_cmd(
            config,
            logger,
            false,
            "nvim",
            &["--headless", "-c", "PackerInstall", "+qa"],
        )
    }

    fn restore_vim_plug(config: &Config, logger: &mut Logger) -> Result<()> {
        super::run_cmd(
            config,
            logger,
            false,
            "nvim",
            &["--headless", "-c", "PlugInstall --sync", "+qa"],
        )
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

    async fn handle_custom_action(
        &self,
        action_name: &str,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<bool> {
        match action_name {
            "nvim-list" => {
                logger
                    .log("Installed nvim plugins are listed in your plugin manager configuration");
                Ok(true)
            }
            "nvim-clean" => {
                let plugin_manager = Self::detect_plugin_manager();
                match plugin_manager.as_deref() {
                    Some("lazy.nvim") => {
                        super::run_cmd(
                            config,
                            logger,
                            false,
                            "nvim",
                            &["--headless", "-c", "Lazy! clean", "+qa"],
                        )?;
                    }
                    Some("packer.nvim") => {
                        super::run_cmd(
                            config,
                            logger,
                            false,
                            "nvim",
                            &["--headless", "-c", "PackerClean", "+qa"],
                        )?;
                    }
                    Some("vim-plug") => {
                        super::run_cmd(
                            config,
                            logger,
                            false,
                            "nvim",
                            &["--headless", "-c", "PlugClean!", "+qa"],
                        )?;
                    }
                    _ => {
                        logger.log("No supported nvim plugin manager detected");
                    }
                }
                Ok(true)
            }
            "nvim-health" => {
                let lua_script = r#"
local results = {}
local h = vim.health

local function hook(name)
  local orig = h[name]
  h[name] = function(msg, details)
    if msg then
      table.insert(results, "[" .. name:upper() .. "] " .. msg .. (details and (" -- " .. details) or ""))
    end
    if orig then return orig(msg, details) end
  end
end

hook("start")
hook("ok")
hook("warn")
hook("error")

vim.cmd("checkhealth")

for _, r in ipairs(results) do
  print(r)
end
"#;

                let script_path = std::env::temp_dir().join("updatehauler_nvim_health.lua");
                std::fs::write(&script_path, lua_script)?;

                let output = cmd("nvim", &["--headless", "-c", &format!("luafile {}", script_path.display()), "-c", "qa"])
                    .stdout_capture()
                    .stderr_capture()
                    .run();

                let _ = std::fs::remove_file(&script_path);

                match output {
                    Ok(output) => {
                        let combined = format!(
                            "{}{}",
                            String::from_utf8_lossy(&output.stdout),
                            String::from_utf8_lossy(&output.stderr),
                        );
                        for line in combined.lines() {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() && !trimmed.contains("checkhealth:") {
                                logger.log(trimmed);
                            }
                        }
                        Ok(true)
                    }
                    Err(e) => {
                        logger.error(&format!("Health check failed: {}", e));
                        Ok(true)
                    }
                }
            }
            _ => Ok(false),
        }
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
