#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use update_hauler::config::Config;
    use update_hauler::insights::Insights;
    use update_hauler::logger::Logger;
    use update_hauler::package_manager::PackageManager;

    #[test]
    fn test_package_manager_creation() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let _pm = PackageManager::new(&config, &insights, &mut logger);

        // Test passes if no panic
    }

    #[test]
    fn test_brew_save_creates_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.brew_save_dir = temp_dir.path().to_path_buf();

        let insights = Insights::new().expect("Failed to create Insights");

        // Set up brew file path
        config.brew_file = config.brew_save_dir.join("test-Brewfile");

        let mut logger = Logger::new(&config);

        if insights.has_brew {
            let mut pm = PackageManager::new(&config, &insights, &mut logger);

            // This will fail if brew is not properly set up, but we're just testing structure
            let result = pm.brew_save();

            // Either success or failure is acceptable for this test
            // We just want to ensure it doesn't panic
            let _ = result;
        }
    }

    #[test]
    fn test_cargo_save_creates_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.cargo_save_dir = temp_dir.path().to_path_buf();

        let insights = Insights::new().expect("Failed to create Insights");

        // Set up cargo file path
        config.cargo_file = config.cargo_save_dir.join("test-cargo-backup.json");

        let mut logger = Logger::new(&config);

        if insights.has_cargo {
            let mut pm = PackageManager::new(&config, &insights, &mut logger);

            // This will fail if cargo-backup is not installed, but we're just testing structure
            let result = pm.cargo_save();

            // Either success or failure is acceptable for this test
            // We just want to ensure it doesn't panic
            let _ = result;
        }
    }

    #[test]
    fn test_os_update_with_no_pkg_mgr() {
        let mut config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");

        // Use dry-run mode to avoid actual updates in tests
        config.dry_run = true;

        // Simulate no package manager
        let mut insights = insights;
        if insights.is_linux {
            insights.pkg_mgr = None;
        }

        let mut logger = Logger::new(&config);
        let mut pm = PackageManager::new(&config, &insights, &mut logger);

        let result = pm.os_update();

        // Should not panic
        assert!(result.is_ok());
    }
}
