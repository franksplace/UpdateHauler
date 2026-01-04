use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    pub debug: Option<bool>,
    pub datetime: Option<bool>,
    pub show_header: Option<bool>,
    pub color: Option<bool>,
    pub use_log: Option<bool>,
    pub dry_run: Option<bool>,
    pub max_log_lines: Option<usize>,
    pub logfile: Option<String>,
    pub installdir: Option<String>,
    pub brew_save_file: Option<String>,
    pub cargo_save_file: Option<String>,
    pub schedule: Option<ScheduleConfig>,
    pub plugins: Option<PluginConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub minute: Option<String>,
    pub hour: Option<String>,
    pub day_of_month: Option<String>,
    pub month: Option<String>,
    pub day_of_week: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    pub brew: Option<bool>,
    pub cargo: Option<bool>,
    pub nvim: Option<bool>,
    pub os: Option<bool>,
}

#[derive(Clone)]
pub struct Config {
    pub app_install_dir: PathBuf,
    pub app_name: String,
    pub brew_save_dir: PathBuf,
    pub cargo_save_dir: PathBuf,
    #[allow(dead_code)]
    pub log_save_dir: PathBuf,
    pub max_log_lines: usize,
    pub sched_minute: String,
    pub sched_hour: String,
    pub sched_day_of_month: String,
    pub sched_month: String,
    pub sched_day_of_week: String,
    pub debug: bool,
    pub datetime: bool,
    pub show_header: bool,
    pub color: bool,
    pub use_log: bool,
    pub dry_run: bool,
    pub log: PathBuf,
    pub brew_file: PathBuf,
    pub cargo_file: PathBuf,
    pub plugins_enabled: PluginConfig,
}

impl Config {
    pub fn new(home: &str) -> Self {
        let home_path = PathBuf::from(home);

        Self {
            app_install_dir: home_path.join(".local/bin"),
            app_name: "updatehauler".to_string(),
            brew_save_dir: home_path.join(".config/brew"),
            cargo_save_dir: home_path.join(".config/cargo"),
            log_save_dir: home_path.join(".local"),
            max_log_lines: 10000,
            sched_minute: "0".to_string(),
            sched_hour: "2".to_string(),
            sched_day_of_month: "*".to_string(),
            sched_month: "*".to_string(),
            sched_day_of_week: "*".to_string(),
            debug: false,
            datetime: true,
            show_header: true,
            color: true,
            use_log: false,
            dry_run: false,
            log: home_path.join(".local/updates.log"),
            brew_file: PathBuf::new(),
            cargo_file: PathBuf::new(),
            plugins_enabled: PluginConfig {
                brew: Some(true),
                cargo: Some(true),
                nvim: Some(false),
                os: Some(true),
            },
        }
    }

    pub fn load_from_yaml(home: &str, config_path: Option<&PathBuf>) -> Result<Self> {
        let mut config = Self::new(home);

        let default_config_path = PathBuf::from(home).join(".config/updatehauler/config.yaml");
        let config_file = config_path.unwrap_or(&default_config_path);

        if !config_file.exists() {
            return Ok(config);
        }

        let config_str = std::fs::read_to_string(config_file)
            .context(format!("Failed to read config file: {:?}", config_file))?;

        let config_file_yaml: ConfigFile =
            serde_yaml::from_str(&config_str).context("Failed to parse YAML configuration")?;

        if let Some(debug) = config_file_yaml.debug {
            config.debug = debug;
        }
        if let Some(datetime) = config_file_yaml.datetime {
            config.datetime = datetime;
        }
        if let Some(show_header) = config_file_yaml.show_header {
            config.show_header = show_header;
        }
        if let Some(color) = config_file_yaml.color {
            config.color = color;
        }
        if let Some(use_log) = config_file_yaml.use_log {
            config.use_log = use_log;
        }
        if let Some(dry_run) = config_file_yaml.dry_run {
            config.dry_run = dry_run;
        }
        if let Some(max_log_lines) = config_file_yaml.max_log_lines {
            config.max_log_lines = max_log_lines;
        }
        if let Some(logfile) = config_file_yaml.logfile {
            config.log = PathBuf::from(logfile);
        }
        if let Some(installdir) = config_file_yaml.installdir {
            config.app_install_dir = PathBuf::from(installdir);
        }
        if let Some(brew_save_file) = config_file_yaml.brew_save_file {
            config.brew_file = PathBuf::from(brew_save_file);
        }
        if let Some(cargo_save_file) = config_file_yaml.cargo_save_file {
            config.cargo_file = PathBuf::from(cargo_save_file);
        }
        if let Some(schedule) = config_file_yaml.schedule {
            if let Some(minute) = schedule.minute {
                config.sched_minute = minute;
            }
            if let Some(hour) = schedule.hour {
                config.sched_hour = hour;
            }
            if let Some(day_of_month) = schedule.day_of_month {
                config.sched_day_of_month = day_of_month;
            }
            if let Some(month) = schedule.month {
                config.sched_month = month;
            }
            if let Some(day_of_week) = schedule.day_of_week {
                config.sched_day_of_week = day_of_week;
            }
        }
        if let Some(plugins) = config_file_yaml.plugins {
            config.plugins_enabled = plugins;
        }

        Ok(config)
    }

    pub fn crontab_timing(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.sched_minute,
            self.sched_hour,
            self.sched_day_of_month,
            self.sched_month,
            self.sched_day_of_week
        )
    }

    pub fn crontab_entry(&self, app_path: &PathBuf) -> String {
        let path_env = self.get_scheduler_path();
        format!(
            "PATH={}; {} {:?} --logfile-only 2>&1",
            path_env,
            self.crontab_timing(),
            app_path
        )
    }

    pub fn get_scheduler_path(&self) -> String {
        let mut path_parts = vec![
            "/usr/local/bin".to_string(),
            "/usr/local/sbin".to_string(),
            "/opt/homebrew/bin".to_string(),
            "/opt/homebrew/sbin".to_string(),
            "/usr/bin".to_string(),
            "/usr/sbin".to_string(),
            "/bin".to_string(),
            "/sbin".to_string(),
        ];

        if let Ok(home) = std::env::var("HOME") {
            path_parts.push(format!("{}/.cargo/bin", home));
        }

        if let Ok(current_path) = std::env::var("PATH") {
            for part in current_path.split(':') {
                if !path_parts.contains(&part.to_string()) {
                    path_parts.push(part.to_string());
                }
            }
        }

        path_parts.join(":")
    }
}
