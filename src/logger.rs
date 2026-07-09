use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use crate::config::Config;

struct LoggerConfig {
    datetime: bool,
    use_log: bool,
    color: bool,
    log: PathBuf,
}

impl LoggerConfig {
    fn from_config(config: &Config) -> Self {
        Self {
            datetime: config.datetime,
            use_log: config.use_log,
            color: config.color,
            log: config.log.clone(),
        }
    }
}

pub struct Logger {
    config: LoggerConfig,
    error_triggered: bool,
}

impl Logger {
    pub fn new(config: &Config) -> Self {
        let log_config = LoggerConfig::from_config(config);
        if log_config.use_log
            && let Some(parent) = log_config.log.parent()
        {
            let _ = std::fs::create_dir_all(parent);
        }
        Self {
            config: log_config,
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

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        writeln!(file, "{}", output)?;

        Ok(())
    }
}
