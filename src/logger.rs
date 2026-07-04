use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use std::fs::OpenOptions;
use std::io::Write;

use crate::config::Config;

pub struct Logger {
    config: Config,
    error_triggered: bool,
}

impl Logger {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            error_triggered: false,
        }
    }

    pub fn log(&mut self, msg: &str) {
        let timestamp = if self.config.datetime {
            Local::now().format("%FT%T%.6f%:z").to_string()
        } else {
            String::new()
        };

        let output = if !timestamp.is_empty() {
            format!("{} {}", timestamp, msg)
        } else {
            msg.to_string()
        };

        if !self.config.use_log {
            if self.error_triggered {
                eprintln!("{}", output);
            } else {
                println!("{}", output);
            }
        } else if let Err(e) = self.write_to_log(&output) {
            eprintln!("Failed to write to log: {}", e);
        }
    }

    pub fn error(&mut self, msg: &str) {
        self.error_triggered = true;

        let colored_msg = if self.config.color {
            msg.red().to_string()
        } else {
            msg.to_string()
        };

        self.log(&format!("ERROR {}", colored_msg));
    }

    fn write_to_log(&self, output: &str) -> Result<()> {
        let log_path = &self.config.log;

        if !log_path.exists()
            && let Some(parent) = log_path.parent()
        {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        writeln!(file, "{}", output)?;

        Ok(())
    }
}
