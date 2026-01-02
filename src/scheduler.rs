use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;

pub struct Scheduler<'a> {
    config: &'a Config,
    insights: &'a Insights,
    logger: &'a mut Logger,
}

impl<'a> Scheduler<'a> {
    pub fn new(config: &'a Config, insights: &'a Insights, logger: &'a mut Logger) -> Self {
        Self {
            config,
            insights,
            logger,
        }
    }

    pub fn enable(&mut self) -> Result<()> {
        if self.insights.is_darwin {
            self.darwin_enable()
        } else {
            self.cron_enable()
        }
    }

    pub fn disable(&mut self) -> Result<()> {
        if self.insights.is_darwin {
            self.darwin_disable()
        } else {
            self.cron_disable()
        }
    }

    pub fn check(&mut self) -> Result<()> {
        if self.insights.is_darwin {
            self.darwin_check()
        } else {
            self.cron_check()
        }
    }

    fn cron_enable(&mut self) -> Result<()> {
        let current_tab = self.get_crontab()?;

        let crontab_entry = self.config.crontab_entry(&self.insights.app_abspath);

        if current_tab.contains(&self.insights.app_abspath.to_string_lossy().to_string()) {
            self.logger.log("Cron entry already enabled");
            return Ok(());
        }

        let new_tab = if current_tab.is_empty() {
            crontab_entry
        } else {
            format!("{}\n{}", current_tab, crontab_entry)
        };

        self.set_crontab(&new_tab)?;

        self.logger.log("Crontab successfully updated");

        Ok(())
    }

    fn cron_disable(&mut self) -> Result<()> {
        let current_tab = self.get_crontab()?;

        if current_tab.is_empty() {
            self.logger.log("No crontab what so ever");
            return Ok(());
        }

        let app_path = self.insights.app_abspath.to_string_lossy().to_string();

        if !current_tab.contains(&app_path) {
            self.logger.log("cron entry not found");
            return Ok(());
        }

        let new_tab: String = current_tab
            .lines()
            .filter(|line| !line.contains(&app_path))
            .collect::<Vec<_>>()
            .join("\n");

        self.set_crontab(&new_tab)?;

        self.logger.log(&format!(
            "Successfully disabled {} in cron",
            self.config.app_name
        ));

        Ok(())
    }

    fn cron_check(&mut self) -> Result<()> {
        let current_tab = self.get_crontab()?;

        if self.config.dry_run {
            self.logger.log("Would check crontab status (DRY-RUN)");
            if current_tab.is_empty() {
                self.logger.log("No crontab entries found (DRY-RUN)");
            } else {
                self.logger.log(&format!("Current crontab:\n{} (DRY-RUN)", current_tab));
            }
            return Ok(());
        }

        if current_tab.is_empty() {
            self.logger.error("no crontab at all enabled");
            return Ok(());
        }

        self.logger.log(&current_tab);

        Ok(())
    }

    fn get_crontab(&self) -> Result<String> {
        let output = std::process::Command::new("crontab").arg("-l").output();

        match output {
            Ok(result) => Ok(String::from_utf8_lossy(&result.stdout).to_string()),
            Err(_) => Ok(String::new()),
        }
    }

    fn set_crontab(&self, content: &str) -> Result<()> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new("crontab")
            .arg("-")
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(content.as_bytes())?;
        }

        let status = child.wait()?;

        if !status.success() {
            anyhow::bail!("Failed to update crontab");
        }

        Ok(())
    }

    fn darwin_enable(&mut self) -> Result<()> {
        let label = "net.franksplace.wake-update-hauler";
        let home = std::env::var("HOME")?;
        let launch_agents_dir = format!("{}/Library/LaunchAgents", home);
        let plist_path = format!("{}/{}.plist", launch_agents_dir, label);
        let plist_path = PathBuf::from(&plist_path);

        fs::create_dir_all(&launch_agents_dir)
            .context("Failed to create LaunchAgents directory")?;

        let app_path = self.insights.app_abspath.to_string_lossy().to_string();

        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{}</string>

  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
    <string>--logfile-only</string>
  </array>
{}
</dict>
</plist>
"#,
            label,
            app_path,
            self.build_calendar_interval()
        );

        fs::write(&plist_path, plist_content)?;

        let mut pm_hour = self.config.sched_hour.clone();
        let mut pm_min = self.config.sched_minute.clone();

        if pm_min == "*" {
            pm_min = "0".to_string();
        }
        if pm_hour == "*" {
            pm_hour = "2".to_string();
        }

        let time_str = format!("{}:{}:00", pm_hour, pm_min);

        let _ = std::process::Command::new("sudo")
            .args(["pmset", "repeat", "wakeorpoweron", "MTWRFSU", &time_str])
            .status();

        let uid = nix::unistd::Uid::effective().as_raw();
        let domain_target = format!("gui/{}", uid);
        let service_name = format!("{}/{}", domain_target, label);

        let _ = std::process::Command::new("launchctl")
            .args(["bootout", &service_name])
            .stderr(std::process::Stdio::null())
            .status();

        std::process::Command::new("launchctl")
            .args(["bootstrap", &domain_target, &plist_path.to_string_lossy()])
            .status()?;

        std::process::Command::new("launchctl")
            .args(["kickstart", "-k", &service_name])
            .status()?;

        self.logger.log(&format!(
            "schedule for Darwin enabled {} with StartCalendarInterval and pmset {}",
            label, time_str
        ));

        Ok(())
    }

    fn darwin_disable(&mut self) -> Result<()> {
        let label = "net.franksplace.wake-update-hauler";
        let home = std::env::var("HOME")?;
        let launch_agents_dir = format!("{}/Library/LaunchAgents", home);
        let plist_path = format!("{}/{}.plist", launch_agents_dir, label);
        let plist_path = PathBuf::from(&plist_path);

        let _ = std::process::Command::new("sudo")
            .args(["pmset", "repeat", "cancel"])
            .status();

        let uid = nix::unistd::Uid::effective().as_raw();
        let domain_target = format!("gui/{}", uid);
        let service_name = format!("{}/{}", domain_target, label);

        let _ = std::process::Command::new("launchctl")
            .args(["bootout", &service_name])
            .stderr(std::process::Stdio::null())
            .status();

        if plist_path.exists() {
            fs::remove_file(&plist_path)?;
        }

        self.logger.log(&format!(
            "schedule for Darwin disabled {} and cleared pmset repeat schedule",
            label
        ));

        Ok(())
    }

    fn darwin_check(&mut self) -> Result<()> {
        let label = "net.franksplace.wake-update-hauler";
        let home = std::env::var("HOME")?;
        let launch_agents_dir = format!("{}/Library/LaunchAgents", home);
        let plist_path = format!("{}/{}.plist", launch_agents_dir, label);
        let plist_path = PathBuf::from(&plist_path);

        if self.config.dry_run {
            self.logger.log(&format!("LaunchAgent plist: {:?} (DRY-RUN)", plist_path));
            self.logger.log("Would check plist existence (DRY-RUN)");
            self.logger.log("Would check launchctl status (DRY-RUN)");
            self.logger.log("Would check pmset schedule (DRY-RUN)");
            return Ok(());
        }

        self.logger.log(&format!("LaunchAgent plist: {:?}", plist_path));
        if plist_path.exists() {
            self.logger.log("  - plist exists");
        } else {
            self.logger.log("  - plist missing");
        }

        self.logger.log("launchctl status:");

        let uid = nix::unistd::Uid::effective().as_raw();
        let domain_target = format!("gui/{}", uid);
        let full_service = format!("{}/{}", domain_target, label);

        let result = std::process::Command::new("launchctl")
            .args(["print", &full_service])
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    for line in output_str.lines().take(40) {
                        self.logger.log(&format!("  {}", line));
                    }
                } else {
                    self.logger
                        .log("  - service not loaded (launchctl print failed)");
                }
            }
            Err(_) => {
                self.logger
                    .log("  - service not loaded (launchctl print failed)");
            }
        }

        self.logger.log("\nPower Management Schedule:");

        let result = std::process::Command::new("pmset")
            .args(["-g", "sched"])
            .output();

        if let Ok(output) = result {
            let output_str = String::from_utf8_lossy(&output.stdout);
            self.logger.log(&output_str);
        }

        Ok(())
    }

    fn build_calendar_interval(&self) -> String {
        let mut parts = String::from("  <key>StartCalendarInterval</key>\n  <dict>");

        if self.config.sched_minute != "*" {
            parts.push_str(&format!(
                "\n    <key>Minute</key>\n    <integer>{}</integer>",
                self.config.sched_minute
            ));
        }

        if self.config.sched_hour != "*" {
            parts.push_str(&format!(
                "\n    <key>Hour</key>\n    <integer>{}</integer>",
                self.config.sched_hour
            ));
        }

        if self.config.sched_day_of_month != "*" {
            parts.push_str(&format!(
                "\n    <key>Day</key>\n    <integer>{}</integer>",
                self.config.sched_day_of_month
            ));
        }

        if self.config.sched_month != "*" {
            parts.push_str(&format!(
                "\n    <key>Month</key>\n    <integer>{}</integer>",
                self.config.sched_month
            ));
        }

        if self.config.sched_day_of_week != "*" {
            parts.push_str(&format!(
                "\n    <key>Weekday</key>\n    <integer>{}</integer>",
                self.config.sched_day_of_week
            ));
        }

        parts.push_str("\n  </dict>");

        parts
    }
}
