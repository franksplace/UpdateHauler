#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use updatehauler::config::Config;
    use updatehauler::insights::Insights;
    use updatehauler::logger::Logger;
    use updatehauler::plugins::{
        BrewPlugin, CargoPlugin, DenoPlugin, DockerPlugin, FlatpakPlugin, GemPlugin, NpmPlugin,
        NvimPlugin, OsPlugin, PipPlugin, Plugin, PluginRegistry, RustupPlugin, SnapPlugin,
        UvPlugin, VscodePlugin,
    };

    #[test]
    fn test_plugin_registry_creation() {
        let _registry = PluginRegistry::new();
    }

    #[test]
    fn test_plugin_registry_register() {
        let mut registry = PluginRegistry::new();
        let brew_plugin = Box::new(BrewPlugin) as Box<dyn Plugin>;
        registry.register(brew_plugin);
    }

    #[test]
    fn test_plugin_registry_get_plugin() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(BrewPlugin));
        registry.register(Box::new(CargoPlugin));
        registry.register(Box::new(DenoPlugin));
        registry.register(Box::new(DockerPlugin));
        registry.register(Box::new(FlatpakPlugin));
        registry.register(Box::new(GemPlugin));
        registry.register(Box::new(NvimPlugin));
        registry.register(Box::new(OsPlugin));
        registry.register(Box::new(RustupPlugin));
        registry.register(Box::new(SnapPlugin));
        registry.register(Box::new(VscodePlugin));

        let brew = registry.get_plugin("brew");
        let cargo = registry.get_plugin("cargo");
        let deno = registry.get_plugin("deno");
        let docker = registry.get_plugin("docker");
        let flatpak = registry.get_plugin("flatpak");
        let gem = registry.get_plugin("gem");
        let nvim = registry.get_plugin("nvim");
        let os = registry.get_plugin("os");
        let rustup = registry.get_plugin("rustup");
        let snap = registry.get_plugin("snap");
        let vscode = registry.get_plugin("vscode");
        let nonexistent = registry.get_plugin("nonexistent");

        assert!(brew.is_some());
        assert_eq!(brew.unwrap().name(), "brew");
        assert!(cargo.is_some());
        assert_eq!(cargo.unwrap().name(), "cargo");
        assert!(deno.is_some());
        assert_eq!(deno.unwrap().name(), "deno");
        assert!(docker.is_some());
        assert_eq!(docker.unwrap().name(), "docker");
        assert!(flatpak.is_some());
        assert_eq!(flatpak.unwrap().name(), "flatpak");
        assert!(gem.is_some());
        assert_eq!(gem.unwrap().name(), "gem");
        assert!(nvim.is_some());
        assert_eq!(nvim.unwrap().name(), "nvim");
        assert!(os.is_some());
        assert_eq!(os.unwrap().name(), "os");
        assert!(rustup.is_some());
        assert_eq!(rustup.unwrap().name(), "rustup");
        assert!(snap.is_some());
        assert_eq!(snap.unwrap().name(), "snap");
        assert!(vscode.is_some());
        assert_eq!(vscode.unwrap().name(), "vscode");
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
    async fn test_deno_plugin_name() {
        let plugin = DenoPlugin;
        assert_eq!(plugin.name(), "deno");
    }

    #[tokio::test]
    async fn test_docker_plugin_name() {
        let plugin = DockerPlugin;
        assert_eq!(plugin.name(), "docker");
    }

    #[tokio::test]
    async fn test_flatpak_plugin_name() {
        let plugin = FlatpakPlugin;
        assert_eq!(plugin.name(), "flatpak");
    }

    #[tokio::test]
    async fn test_gem_plugin_name() {
        let plugin = GemPlugin;
        assert_eq!(plugin.name(), "gem");
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
    async fn test_rustup_plugin_name() {
        let plugin = RustupPlugin;
        assert_eq!(plugin.name(), "rustup");
    }

    #[tokio::test]
    async fn test_snap_plugin_name() {
        let plugin = SnapPlugin;
        assert_eq!(plugin.name(), "snap");
    }

    #[tokio::test]
    async fn test_vscode_plugin_name() {
        let plugin = VscodePlugin;
        assert_eq!(plugin.name(), "vscode");
    }

    #[tokio::test]
    async fn test_brew_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = BrewPlugin;
        let _ = plugin.check_available(&config, &insights).await;
    }

    #[tokio::test]
    async fn test_cargo_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = CargoPlugin;
        let _ = plugin.check_available(&config, &insights).await;
    }

    #[tokio::test]
    async fn test_deno_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = DenoPlugin;
        let _ = plugin.check_available(&config, &insights).await;
    }

    #[tokio::test]
    async fn test_docker_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = DockerPlugin;
        let _ = plugin.check_available(&config, &insights).await;
    }

    #[tokio::test]
    async fn test_flatpak_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = FlatpakPlugin;
        let _ = plugin.check_available(&config, &insights).await;
    }

    #[tokio::test]
    async fn test_gem_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = GemPlugin;
        let _ = plugin.check_available(&config, &insights).await;
    }

    #[tokio::test]
    async fn test_nvim_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = NvimPlugin;
        let _ = plugin.check_available(&config, &insights).await;
    }

    #[tokio::test]
    async fn test_os_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = OsPlugin;
        assert!(plugin.check_available(&config, &insights).await);
    }

    #[tokio::test]
    async fn test_rustup_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = RustupPlugin;
        let _ = plugin.check_available(&config, &insights).await;
    }

    #[tokio::test]
    async fn test_snap_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = SnapPlugin;
        let _ = plugin.check_available(&config, &insights).await;
    }

    #[tokio::test]
    async fn test_vscode_plugin_check_available() {
        let config = Config::new("/tmp/test");
        let insights = Insights::new().expect("Failed to create Insights");
        let plugin = VscodePlugin;
        let _ = plugin.check_available(&config, &insights).await;
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

        let _ = brew_plugin.update(&config, &insights, &mut logger).await;
        let _ = brew_plugin.save(&config, &insights, &mut logger).await;
        let _ = cargo_plugin.update(&config, &insights, &mut logger).await;
        let _ = cargo_plugin.save(&config, &insights, &mut logger).await;
        let _ = os_plugin.update(&config, &insights, &mut logger).await;
    }

    #[tokio::test]
    async fn test_new_plugins_dry_run() {
        let mut config = Config::new("/tmp/test");
        config.dry_run = true;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let deno_plugin = DenoPlugin;
        let docker_plugin = DockerPlugin;
        let flatpak_plugin = FlatpakPlugin;
        let gem_plugin = GemPlugin;
        let rustup_plugin = RustupPlugin;
        let snap_plugin = SnapPlugin;
        let vscode_plugin = VscodePlugin;

        let _ = deno_plugin.update(&config, &insights, &mut logger).await;
        let _ = docker_plugin.update(&config, &insights, &mut logger).await;
        let _ = flatpak_plugin.update(&config, &insights, &mut logger).await;
        let _ = gem_plugin.update(&config, &insights, &mut logger).await;
        let _ = rustup_plugin.update(&config, &insights, &mut logger).await;
        let _ = snap_plugin.update(&config, &insights, &mut logger).await;
        let _ = vscode_plugin.update(&config, &insights, &mut logger).await;
    }

    #[tokio::test]
    async fn test_nvim_plugin_save_restore_dry_run() {
        let mut config = Config::new("/tmp/test");
        config.dry_run = true;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let nvim_plugin = NvimPlugin;

        let _ = nvim_plugin.save(&config, &insights, &mut logger).await;
        let _ = nvim_plugin.restore(&config, &insights, &mut logger).await;
    }

    #[tokio::test]
    async fn test_plugin_save_dry_run() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.brew_save_dir = temp_dir.path().to_path_buf();
        config.cargo_save_dir = temp_dir.path().to_path_buf();
        config.npm_file = temp_dir.path().join("npm-packages.json");
        config.pip_file = temp_dir.path().join("pip-requirements.txt");
        config.uv_file = temp_dir.path().join("uv-tools.json");
        config.brew_file = config.brew_save_dir.join("test-Brewfile");
        config.cargo_file = config.cargo_save_dir.join("test-cargo-backup.json");
        config.dry_run = true;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let plugins: Vec<Box<dyn Plugin>> = vec![
            Box::new(BrewPlugin),
            Box::new(CargoPlugin),
            Box::new(NpmPlugin),
            Box::new(NvimPlugin),
            Box::new(PipPlugin),
            Box::new(UvPlugin),
        ];

        for plugin in &plugins {
            let result = plugin.save(&config, &insights, &mut logger).await;
            assert!(result.is_ok(), "{} save failed in dry-run", plugin.name());
        }
    }

    #[tokio::test]
    async fn test_plugin_restore_dry_run() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.brew_save_dir = temp_dir.path().to_path_buf();
        config.cargo_save_dir = temp_dir.path().to_path_buf();
        config.npm_file = temp_dir.path().join("npm-packages.json");
        config.pip_file = temp_dir.path().join("pip-requirements.txt");
        config.uv_file = temp_dir.path().join("uv-tools.json");
        config.brew_file = config.brew_save_dir.join("test-Brewfile");
        config.cargo_file = config.cargo_save_dir.join("test-cargo-backup.json");
        config.dry_run = true;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let plugins: Vec<Box<dyn Plugin>> = vec![
            Box::new(BrewPlugin),
            Box::new(CargoPlugin),
            Box::new(NpmPlugin),
            Box::new(NvimPlugin),
            Box::new(PipPlugin),
            Box::new(UvPlugin),
        ];

        for plugin in &plugins {
            let result = plugin.restore(&config, &insights, &mut logger).await;
            assert!(
                result.is_ok(),
                "{} restore failed in dry-run",
                plugin.name()
            );
        }
    }

    #[tokio::test]
    async fn test_npm_save_writes_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.npm_file = temp_dir.path().join("npm-packages.json");
        config.dry_run = false;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let npm_plugin = NpmPlugin;

        if insights.has_npm {
            let result = npm_plugin.save(&config, &insights, &mut logger).await;
            assert!(result.is_ok(), "npm save failed");
            let npm_file = temp_dir.path().join("npm-packages.json");
            assert!(npm_file.exists(), "npm save file was not created");
            let content = std::fs::read_to_string(&npm_file).unwrap_or_default();
            assert!(!content.is_empty(), "npm save file is empty");
        }
    }

    #[tokio::test]
    async fn test_pip_save_writes_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.pip_file = temp_dir.path().join("pip-requirements.txt");
        config.dry_run = false;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let pip_plugin = PipPlugin;

        if insights.has_pip || insights.has_uv {
            let result = pip_plugin.save(&config, &insights, &mut logger).await;
            assert!(result.is_ok(), "pip save failed");
            let pip_file = temp_dir.path().join("pip-requirements.txt");
            assert!(pip_file.exists(), "pip save file was not created");
            let content = std::fs::read_to_string(&pip_file).unwrap_or_default();
            assert!(!content.is_empty(), "pip save file is empty");
        }
    }

    #[tokio::test]
    async fn test_uv_save_writes_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.uv_file = temp_dir.path().join("uv-tools.json");
        config.dry_run = false;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let uv_plugin = UvPlugin;

        if insights.has_uv {
            let result = uv_plugin.save(&config, &insights, &mut logger).await;
            assert!(result.is_ok(), "uv save failed");
            let uv_file = temp_dir.path().join("uv-tools.json");
            assert!(uv_file.exists(), "uv save file was not created");
            let content = std::fs::read_to_string(&uv_file).unwrap_or_default();
            assert!(!content.is_empty(), "uv save file is empty");
        }
    }

    #[tokio::test]
    async fn test_npm_restore_with_saved_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.npm_file = temp_dir.path().join("npm-packages.json");
        config.dry_run = true;

        std::fs::write(&config.npm_file, "{\"dependencies\":{}}\n")
            .expect("Failed to write test save file");

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let npm_plugin = NpmPlugin;
        let result = npm_plugin.restore(&config, &insights, &mut logger).await;
        assert!(result.is_ok(), "npm restore with saved file failed");
    }

    #[tokio::test]
    async fn test_pip_restore_with_saved_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.pip_file = temp_dir.path().join("pip-requirements.txt");
        config.dry_run = true;

        std::fs::write(&config.pip_file, "# pip requirements\n")
            .expect("Failed to write test save file");

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let pip_plugin = PipPlugin;
        let result = pip_plugin.restore(&config, &insights, &mut logger).await;
        assert!(result.is_ok(), "pip restore with saved file failed");
    }

    #[tokio::test]
    async fn test_uv_restore_with_saved_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.uv_file = temp_dir.path().join("uv-tools.json");
        config.dry_run = true;

        std::fs::write(&config.uv_file, "ruff\npdm\n").expect("Failed to write test save file");

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let uv_plugin = UvPlugin;
        let result = uv_plugin.restore(&config, &insights, &mut logger).await;
        assert!(result.is_ok(), "uv restore with saved file failed");
    }

    #[tokio::test]
    async fn test_plugin_restore_missing_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut config = Config::new("/tmp/test");
        config.brew_save_dir = temp_dir.path().to_path_buf();
        config.cargo_save_dir = temp_dir.path().to_path_buf();
        config.npm_file = temp_dir.path().join("nonexistent-npm.json");
        config.pip_file = temp_dir.path().join("nonexistent-pip.txt");
        config.uv_file = temp_dir.path().join("nonexistent-uv.txt");
        config.brew_file = config.brew_save_dir.join("nonexistent-Brewfile");
        config.cargo_file = config.cargo_save_dir.join("nonexistent-cargo-backup.json");
        config.dry_run = false;

        let insights = Insights::new().expect("Failed to create Insights");
        let mut logger = Logger::new(&config);

        let brew_plugin = BrewPlugin;
        let cargo_plugin = CargoPlugin;
        let npm_plugin = NpmPlugin;
        let pip_plugin = PipPlugin;
        let uv_plugin = UvPlugin;

        let _ = brew_plugin.restore(&config, &insights, &mut logger).await;
        let _ = cargo_plugin.restore(&config, &insights, &mut logger).await;
        let _ = npm_plugin.restore(&config, &insights, &mut logger).await;
        let _ = pip_plugin.restore(&config, &insights, &mut logger).await;
        let _ = uv_plugin.restore(&config, &insights, &mut logger).await;
    }
}
