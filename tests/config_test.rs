#[cfg(test)]
mod tests {
    use std::path::PathBuf;
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
}
