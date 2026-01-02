use std::path::PathBuf;

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
        }
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
        format!(
            "{} {:?} --logfile-only 2>&1",
            self.crontab_timing(),
            app_path
        )
    }
}
