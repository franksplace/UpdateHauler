use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::insights::Insights;

pub struct SelfInstaller {
    config: Config,
    insights: Insights,
}

impl SelfInstaller {
    pub fn new(config: &Config, insights: &Insights) -> Self {
        Self {
            config: config.clone(),
            insights: insights.clone(),
        }
    }

    pub fn install(&self) -> Result<()> {
        if self.insights.is_cargo_install {
            bail!(
                "Already installed via cargo. Use `cargo install {}` to reinstall.",
                self.config.app_name
            );
        }
        self.copy_binary("Installing", "Installed", "to install")
    }

    pub fn update(&self) -> Result<()> {
        if self.insights.is_cargo_install {
            println!("Updating {} via cargo...", self.config.app_name);
            let status = Command::new("cargo")
                .args(["install", &self.config.app_name])
                .status()
                .context("Failed to run cargo install. Is cargo installed?")?;
            if status.success() {
                println!("Successfully updated {}", self.config.app_name);
            } else {
                bail!("cargo install failed with exit code {:?}", status.code());
            }
            return Ok(());
        }
        self.copy_binary("Updating", "Updated", "to update")
    }

    pub fn remove(&self) -> Result<()> {
        if self.insights.is_cargo_install {
            bail!(
                "Installed via cargo. Use `cargo uninstall {}` to remove.",
                self.config.app_name
            );
        }
        let install_path = self.config.app_install_dir.join(&self.config.app_name);

        if install_path.exists() {
            println!("Removing {:?}", install_path);

            fs::remove_file(&install_path)?;

            println!("Successfully removed {}", self.config.app_name);
        } else {
            println!("{:?} is not installed", install_path);
        }

        Ok(())
    }

    fn copy_binary(&self, prefix: &str, success_ending: &str, _error_ending: &str) -> Result<()> {
        let install_path = self.config.app_install_dir.join(&self.config.app_name);
        let source_path = &self.insights.app_abspath;

        let needs_install =
            !install_path.exists() || !Self::files_equal(source_path, &install_path)?;

        if needs_install {
            println!("{} {:?}", prefix, install_path);

            if !self.config.app_install_dir.exists() {
                fs::create_dir_all(&self.config.app_install_dir)
                    .context("Failed to create install directory")?;
            }

            fs::copy(source_path, &install_path).context("Failed to copy binary")?;

            println!("Successfully {} {}", success_ending, self.config.app_name);
        } else {
            println!("{:?} is already installed and up to date", install_path);
        }

        Ok(())
    }

    pub fn files_equal(path1: &Path, path2: &Path) -> Result<bool> {
        let meta1 = fs::metadata(path1)?;
        let meta2 = fs::metadata(path2)?;

        if meta1.len() != meta2.len() {
            return Ok(false);
        }

        let content1 = fs::read(path1)?;
        let content2 = fs::read(path2)?;

        Ok(content1 == content2)
    }
}
