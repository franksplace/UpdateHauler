use anyhow::Result;
use duct::cmd;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;

pub struct PackageManager<'a> {
    config: &'a Config,
    insights: &'a Insights,
    logger: &'a mut Logger,
}

impl<'a> PackageManager<'a> {
    pub fn new(config: &'a Config, insights: &'a Insights, logger: &'a mut Logger) -> Self {
        Self {
            config,
            insights,
            logger,
        }
    }

    fn run_cmd(&mut self, show_error: bool, command: &str, args: &[&str]) -> Result<()> {
        let cmd_str = format!("{} {}", command, args.join(" "));

        // Get short command name for prefixing output
        let short_cmd = if command == "sudo" && args.len() >= 4 {
            // For sudo sh -c "command", extract the actual command
            args[3]
        } else {
            command
        };

        if self.config.show_header {
            self.logger.log(&format!("{} → Start", cmd_str));
        }

        // Spawn the process
        let mut child = Command::new(command)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Get the stdout and stderr handles
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to stderr");

        // Create a channel for the main thread to receive log messages
        let (sender, receiver) = mpsc::channel::<String>();

        // Clone the necessary values for the threads
        let sender_stdout = sender.clone();
        let sender_stderr = sender;

        // Spawn a thread to read stdout
        let stdout_thread = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                let _ = sender_stdout.send(line);
            }
        });

        // Spawn a thread to read stderr
        let stderr_thread = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                let _ = sender_stderr.send(line);
            }
        });

        // Read log messages from the channel and log them in real-time
        for received in receiver {
            if !received.is_empty() {
                let formatted = if self.config.show_header {
                    format!("{} → {}", short_cmd, received)
                } else {
                    received
                };
                self.logger.log(&formatted);
            }
        }

        // Wait for both threads to complete
        let _ = stdout_thread.join();
        let _ = stderr_thread.join();

        // Wait for the process to complete and get the exit code
        let exit_code = child.wait()?.code().unwrap_or(0);

        if self.config.show_header {
            if show_error && exit_code != 0 {
                self.logger
                    .error(&format!("{} → Return code {}", cmd_str, exit_code));
            } else {
                self.logger
                    .log(&format!("{} → Return code {}", cmd_str, exit_code));
            }
        }

        Ok(())
    }

    pub fn brew_update(&mut self) -> Result<()> {
        if !self.insights.has_brew {
            return Ok(());
        }

        self.run_cmd(true, "brew", &["update"])?;
        self.run_cmd(true, "brew", &["upgrade"])?;
        self.run_cmd(true, "brew", &["cleanup", "-q"])?;
        self.run_cmd(false, "brew", &["doctor", "-q"])?;
        self.run_cmd(true, "brew", &["upgrade", "--cask"])?;
        self.run_cmd(true, "brew", &["cu", "-a", "-f", "--cleanup", "-y"])?;
        self.run_cmd(true, "brew", &["cleanup", "-q"])?;
        self.run_cmd(false, "brew", &["doctor", "--verbose"])?;

        Ok(())
    }

    pub fn brew_save(&mut self) -> Result<()> {
        if !self.insights.has_brew {
            return Ok(());
        }

        let brew_file = self.config.brew_file.to_string_lossy().to_string();

        if let Some(parent) = self.config.brew_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        self.logger
            .log(&format!("Generating brew's {} save file", brew_file));

        self.run_cmd(
            true,
            "brew",
            &["bundle", "dump", "--force", "--file", &brew_file],
        )?;

        self.logger.log("Success savefile written");

        Ok(())
    }

    pub fn brew_restore(&mut self) -> Result<()> {
        if !self.insights.has_brew {
            return Ok(());
        }

        let brew_file = self.config.brew_file.to_string_lossy().to_string();

        if !self.config.brew_file.exists() {
            self.logger.error(&format!(
                "missing dependency — {} brew's backup file is not found",
                brew_file
            ));
            return Ok(());
        }

        self.run_cmd(true, "brew", &["bundle", "--file", &brew_file])?;

        Ok(())
    }

    pub fn cargo_update(&mut self) -> Result<()> {
        if !self.insights.has_cargo {
            return Ok(());
        }

        // Check if cargo-update is installed (silent check - to stdout)
        let _ = cmd("cargo", &["install-update", "--version"])
            .stdout_null()
            .run();

        self.run_cmd(true, "cargo", &["install-update", "-a"])?;

        Ok(())
    }

    pub fn cargo_save(&mut self) -> Result<()> {
        if !self.insights.has_cargo {
            return Ok(());
        }

        let cargo_file = self.config.cargo_file.to_string_lossy().to_string();

        if let Some(parent) = self.config.cargo_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Check if cargo-backup is installed (silent check - to stdout)
        let _ = cmd("cargo", &["backup", "--version"]).stdout_null().run();

        self.logger
            .log(&format!("Generating cargo's {} save file", cargo_file));

        self.run_cmd(true, "cargo", &["backup", "-o", &cargo_file])?;

        self.logger.log("Success savefile written");

        Ok(())
    }

    pub fn cargo_restore(&mut self) -> Result<()> {
        if !self.insights.has_cargo {
            return Ok(());
        }

        let cargo_file = self.config.cargo_file.to_string_lossy().to_string();

        if !self.config.cargo_file.exists() {
            self.logger.error(&format!(
                "missing dependency — {} cargo's backup json file is not found",
                cargo_file
            ));
            return Ok(());
        }

        // Check if cargo-restore is installed (silent check - to stdout)
        let _ = cmd("cargo", &["restore", "--version"]).stdout_null().run();

        self.run_cmd(
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

    pub fn os_update(&mut self) -> Result<()> {
        if self.insights.is_darwin {
            self.run_cmd(true, "softwareupdate", &["-a", "-i", "--verbose"])?;

            self.run_cmd(true, "mas", &["update"])?;

            return Ok(());
        }

        if self.insights.is_linux {
            if let Some(ref pkg_mgr) = self.insights.pkg_mgr {
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
                        self.logger.error("OS not supported for updates");
                        return Ok(());
                    }
                };

                for cmd_args in commands {
                    if self.insights.is_root {
                        let (program, args) = cmd_args.split_first().unwrap();
                        let args: Vec<&str> = args.to_vec();
                        self.run_cmd(true, program, &args)?;
                    } else {
                        let shell_cmd = cmd_args.join(" ");
                        self.run_cmd(true, "sudo", &["sh", "-c", &shell_cmd])?;
                    }
                }
            } else {
                self.logger.error("OS not supported for updates");
            }
        }

        Ok(())
    }
}
