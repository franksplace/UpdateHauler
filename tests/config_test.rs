#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use tempfile::TempDir;
    use update_hauler::config::Config;

    #[test]
    fn test_config_new() {
        let home = "/home/test";
        let config = Config::new(home);

        assert_eq!(config.app_name, "updatehauler");
        assert_eq!(config.max_log_lines, 10000);
        assert_eq!(config.sched_minute, "0");
        assert_eq!(config.sched_hour, "2");
        assert!(!config.debug);
        assert!(config.datetime);
        assert!(config.show_header);
        assert!(config.color);
        assert!(!config.use_log);
    }

    #[test]
    fn test_config_paths() {
        let home = "/home/test";
        let config = Config::new(home);

        let expected_install_dir: PathBuf = PathBuf::from("/home/test/.local/bin");
        let expected_brew_dir: PathBuf = PathBuf::from("/home/test/.config/brew");
        let expected_cargo_dir: PathBuf = PathBuf::from("/home/test/.config/cargo");
        let expected_log: PathBuf = PathBuf::from("/home/test/.local/updates.log");

        assert_eq!(config.app_install_dir, expected_install_dir);
        assert_eq!(config.brew_save_dir, expected_brew_dir);
        assert_eq!(config.cargo_save_dir, expected_cargo_dir);
        assert_eq!(config.log, expected_log);
    }

    #[test]
    fn test_config_plugins_enabled() {
        let home = "/home/test";
        let config = Config::new(home);

        assert_eq!(config.plugins_enabled.brew, Some(true));
        assert_eq!(config.plugins_enabled.cargo, Some(true));
        assert_eq!(config.plugins_enabled.nvim, Some(false));
        assert_eq!(config.plugins_enabled.os, Some(true));
    }

    #[test]
    fn test_crontab_timing() {
        let config = Config::new("/home/test");
        let timing = config.crontab_timing();

        assert_eq!(timing, "0 2 * * *");
    }

    #[test]
    fn test_crontab_entry() {
        let config = Config::new("/home/test");
        let app_path: PathBuf = PathBuf::from("/usr/local/bin/updatehauler");
        let entry = config.crontab_entry(&app_path);

        assert!(entry.contains("0 2 * * *"));
        assert!(entry.contains("/usr/local/bin/updatehauler"));
        assert!(entry.contains("--logfile-only"));
    }

    #[test]
    fn test_config_load_from_yaml_nonexistent() {
        let home = "/tmp/test";
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.yaml");

        let config =
            Config::load_from_yaml(home, Some(&config_path)).expect("Failed to load config");

        // Should use defaults when file doesn't exist
        assert_eq!(config.app_name, "updatehauler");
        assert!(!config.debug);
        assert!(config.datetime);
    }

    #[test]
    fn test_config_load_from_yaml_basic() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.yaml");

        std::fs::write(
            &config_path,
            r#"
debug: true
datetime: false
show_header: false
color: false
use_log: true
dry_run: true
max_log_lines: 5000
"#,
        )
        .expect("Failed to write config file");

        let config = Config::load_from_yaml("/home/test", Some(&config_path))
            .expect("Failed to load config");

        assert!(config.debug);
        assert!(!config.datetime);
        assert!(!config.show_header);
        assert!(!config.color);
        assert!(config.use_log);
        assert!(config.dry_run);
        assert_eq!(config.max_log_lines, 5000);
    }

    #[test]
    fn test_config_load_from_yaml_schedule() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.yaml");

        std::fs::write(
            &config_path,
            r#"
schedule:
  minute: "30"
  hour: "15"
  day_of_month: "1"
  month: "6"
  day_of_week: "2"
"#,
        )
        .expect("Failed to write config file");

        let config = Config::load_from_yaml("/home/test", Some(&config_path))
            .expect("Failed to load config");

        assert_eq!(config.sched_minute, "30");
        assert_eq!(config.sched_hour, "15");
        assert_eq!(config.sched_day_of_month, "1");
        assert_eq!(config.sched_month, "6");
        assert_eq!(config.sched_day_of_week, "2");
    }

    #[test]
    fn test_config_load_from_yaml_plugins() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.yaml");

        std::fs::write(
            &config_path,
            r#"
plugins:
  brew: false
  cargo: false
  nvim: true
  os: false
"#,
        )
        .expect("Failed to write config file");

        let config = Config::load_from_yaml("/home/test", Some(&config_path))
            .expect("Failed to load config");

        assert_eq!(config.plugins_enabled.brew, Some(false));
        assert_eq!(config.plugins_enabled.cargo, Some(false));
        assert_eq!(config.plugins_enabled.nvim, Some(true));
        assert_eq!(config.plugins_enabled.os, Some(false));
    }

    #[test]
    fn test_config_load_from_yaml_paths() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.yaml");

        std::fs::write(
            &config_path,
            r#"
logfile: "/var/log/test.log"
installdir: "/opt/test/bin"
brew_save_file: "/tmp/test/Brewfile"
cargo_save_file: "/tmp/test/cargo.json"
"#,
        )
        .expect("Failed to write config file");

        let config = Config::load_from_yaml("/home/test", Some(&config_path))
            .expect("Failed to load config");

        assert_eq!(config.log, PathBuf::from("/var/log/test.log"));
        assert_eq!(config.app_install_dir, PathBuf::from("/opt/test/bin"));
        assert_eq!(config.brew_file, PathBuf::from("/tmp/test/Brewfile"));
        assert_eq!(config.cargo_file, PathBuf::from("/tmp/test/cargo.json"));
    }
}
