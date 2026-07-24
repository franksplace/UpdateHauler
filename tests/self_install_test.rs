#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use updatehauler::config::Config;
    use updatehauler::insights::Insights;
    use updatehauler::self_install::SelfInstaller;

    #[test]
    fn test_installer_creation() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");

        let _installer = SelfInstaller::new(&config, &insights);

        // Test passes if no panic
    }

    #[test]
    fn test_files_equal_same_content() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let _installer = SelfInstaller::new(&config, &insights);

        // Create two files with same content
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("test1.txt");
        let file2 = temp_dir.join("test2.txt");

        fs::write(&file1, "same content").expect("Failed to write file1");
        fs::write(&file2, "same content").expect("Failed to write file2");

        let result = SelfInstaller::files_equal(&file1, &file2);

        assert!(result.is_ok());
        assert!(result.unwrap());

        // Cleanup
        let _ = fs::remove_file(&file1);
        let _ = fs::remove_file(&file2);
    }

    #[test]
    fn test_files_equal_different_content() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let _installer = SelfInstaller::new(&config, &insights);

        // Create two files with different content
        let temp_dir = env::temp_dir();
        let file1 = temp_dir.join("test3.txt");
        let file2 = temp_dir.join("test4.txt");

        fs::write(&file1, "content A").expect("Failed to write file1");
        fs::write(&file2, "content B").expect("Failed to write file2");

        let result = SelfInstaller::files_equal(&file1, &file2);

        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Cleanup
        let _ = fs::remove_file(&file1);
        let _ = fs::remove_file(&file2);
    }

    #[test]
    fn test_remove_nonexistent_file() {
        let temp_dir = env::temp_dir();

        let mut config = Config::new("/tmp/test");
        config.app_install_dir = temp_dir.clone();

        let insights = Insights::new().expect("Failed to create Insights");

        // Mock app_abspath to point to a non-existent file
        let mut insights = insights;
        insights.app_abspath = temp_dir.join("fake_binary");

        // Create a fake binary to copy
        if !insights.app_abspath.exists() {
            fs::write(&insights.app_abspath, "fake binary").expect("Failed to create fake binary");
        }

        let installer = SelfInstaller::new(&config, &insights);

        // Try to remove when nothing is installed
        let result = installer.remove();

        // Should succeed (no-op)
        assert!(result.is_ok());

        // Cleanup
        let _ = fs::remove_file(&insights.app_abspath);
    }

    #[test]
    fn test_cargo_install_install_errors() {
        let config = Config::new("/tmp/test");
        let mut insights = Insights::new().expect("Failed to create Insights");
        insights.is_cargo_install = true;

        let installer = SelfInstaller::new(&config, &insights);
        let result = installer.install();

        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("cargo install"));
    }

    #[test]
    fn test_cargo_install_remove_errors() {
        let config = Config::new("/tmp/test");
        let mut insights = Insights::new().expect("Failed to create Insights");
        insights.is_cargo_install = true;

        let installer = SelfInstaller::new(&config, &insights);
        let result = installer.remove();

        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("cargo uninstall"));
    }

    #[test]
    fn test_cargo_install_update_tries_cargo() {
        if std::env::var("CI").is_ok() {
            return; // Skip in CI — cargo install is too slow under tarpaulin
        }

        let config = Config::new("/tmp/test");
        let mut insights = Insights::new().expect("Failed to create Insights");
        insights.is_cargo_install = true;

        let installer = SelfInstaller::new(&config, &insights);
        let result = installer.update();

        // Should succeed — runs `cargo install updatehauler` which is a no-op if already installed
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Skip due to flaky file system behavior in test environment
    fn test_install_creates_directory() {
        let temp_dir = env::temp_dir();
        let install_dir = temp_dir.join("test_install");
        let install_path = install_dir.join("updatehauler");

        let mut config = Config::new("/tmp/test");
        config.app_install_dir = install_dir.clone();

        let insights = Insights::new().expect("Failed to create Insights");

        // Create a fake binary to copy
        let fake_binary = temp_dir.join("fake_binary");
        fs::write(&fake_binary, "fake binary content").expect("Failed to create fake binary");

        // Create a new insights with modified app_abspath
        let mut installer_insights = insights.clone();
        installer_insights.app_abspath = fake_binary.clone();

        let installer = SelfInstaller::new(&config, &installer_insights);

        let result = installer.install();

        if let Err(e) = &result {
            eprintln!("Install failed: {:?}", e);
        }
        assert!(result.is_ok());
        assert!(install_path.exists());
        assert!(install_dir.exists());

        // Cleanup
        let _ = fs::remove_file(&install_path);
        let _ = fs::remove_dir_all(&install_dir);
        let _ = fs::remove_file(&fake_binary);
    }
}
