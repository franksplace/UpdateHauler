#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use update_hauler::config::Config;
    use update_hauler::insights::Insights;
    use update_hauler::logger::Logger;
    #[allow(unused_imports)]
    use update_hauler::plugins::{
        BrewPlugin, CargoPlugin, NvimPlugin, OsPlugin, Plugin, PluginRegistry,
    };

    #[test]
    fn test_plugin_registry_creation() {
        let _registry = PluginRegistry::new();

        // Test passes if no panic
    }

    #[test]
    fn test_plugin_registry_register() {
        let mut registry = PluginRegistry::new();
        let brew_plugin = Box::new(BrewPlugin) as Box<dyn Plugin>;

        registry.register(brew_plugin);

        // Test passes if no panic
    }

    #[test]
    fn test_plugin_registry_get_plugin() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(BrewPlugin));
        registry.register(Box::new(CargoPlugin));
        registry.register(Box::new(NvimPlugin));
        registry.register(Box::new(OsPlugin));

        let brew = registry.get_plugin("brew");
        let cargo = registry.get_plugin("cargo");
        let nvim = registry.get_plugin("nvim");
        let os = registry.get_plugin("os");
        let nonexistent = registry.get_plugin("nonexistent");

        assert!(brew.is_some());
        assert_eq!(brew.unwrap().name(), "brew");
        assert!(cargo.is_some());
        assert_eq!(cargo.unwrap().name(), "cargo");
        assert!(nvim.is_some());
        assert_eq!(nvim.unwrap().name(), "nvim");
        assert!(os.is_some());
        assert_eq!(os.unwrap().name(), "os");
        assert!(nonexistent.is_none());
    }

    #[tokio::test]
    async fn test_brew_plugin_name() {
        let plugin = BrewPlugin;
        assert_eq!(plugin.name(), "brew");
    }

    #[tokio::test]
    async fn test_cargo_plugin_name() {
        let plugin = CargoPlugin;
        assert_eq!(plugin.name(), "cargo");
    }

    #[tokio::test]
    async fn test_nvim_plugin_name() {
        let plugin = NvimPlugin;
        assert_eq!(plugin.name(), "nvim");
    }

    #[tokio::test]
    async fn test_os_plugin_name() {
        let plugin = OsPlugin;
        assert_eq!(plugin.name(), "os");
    }

    #[tokio::test]
    async fn test_brew_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = BrewPlugin;

        let available = plugin.check_available(&config, &insights).await;

        // Should not panic
        let _ = available;
    }

    #[tokio::test]
    async fn test_cargo_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = CargoPlugin;

        let available = plugin.check_available(&config, &insights).await;

        // Should not panic
        let _ = available;
    }

    #[tokio::test]
    async fn test_nvim_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = NvimPlugin;

        let available = plugin.check_available(&config, &insights).await;

        // Should not panic
        let _ = available;
    }

    #[tokio::test]
    async fn test_os_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = OsPlugin;

        let available = plugin.check_available(&config, &insights).await;

        assert!(available, "OS plugin should always be available");
    }

    #[tokio::test]
    async fn test_plugin_dry_run_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.brew_save_dir = temp_dir.path().to_path_buf();
        config.cargo_save_dir = temp_dir.path().to_path_buf();
        config.brew_file = config.brew_save_dir.join("test-Brewfile");
        config.cargo_file = config.cargo_save_dir.join("test-cargo-backup.json");
        config.dry_run = true;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let brew_plugin = BrewPlugin;
        let cargo_plugin = CargoPlugin;
        let os_plugin = OsPlugin;

        // These should not panic in dry-run mode
        let _ = brew_plugin.update(&config, &insights, &mut logger).await;
        let _ = brew_plugin.save(&config, &insights, &mut logger).await;
        let _ = cargo_plugin.update(&config, &insights, &mut logger).await;
        let _ = cargo_plugin.save(&config, &insights, &mut logger).await;
        let _ = os_plugin.update(&config, &insights, &mut logger).await;
    }

    #[tokio::test]
    async fn test_nvim_plugin_save_restore_dry_run() {
        let mut config = Config::new("/tmp/test");
        config.dry_run = true;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let nvim_plugin = NvimPlugin;

        // These should not panic even if nvim is not installed
        let _ = nvim_plugin.save(&config, &insights, &mut logger).await;
        let _ = nvim_plugin.restore(&config, &insights, &mut logger).await;
    }

    #[tokio::test]
    async fn test_plugin_restore_missing_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.brew_save_dir = temp_dir.path().to_path_buf();
        config.cargo_save_dir = temp_dir.path().to_path_buf();
        config.brew_file = config.brew_save_dir.join("nonexistent-Brewfile");
        config.cargo_file = config.cargo_save_dir.join("nonexistent-cargo-backup.json");
        config.dry_run = false;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let brew_plugin = BrewPlugin;
        let cargo_plugin = CargoPlugin;

        // These should not panic even if files don't exist
        let _ = brew_plugin.restore(&config, &insights, &mut logger).await;
        let _ = cargo_plugin.restore(&config, &insights, &mut logger).await;
    }
}
