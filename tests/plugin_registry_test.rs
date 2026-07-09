use updatehauler::config::Config;
use updatehauler::insights::Insights;
use updatehauler::logger::Logger;
use updatehauler::plugins::{
    BrewPlugin, CargoPlugin, DenoPlugin, DockerPlugin, FlatpakPlugin, GemPlugin, GoPlugin,
    NpmPlugin, NvimPlugin, OsPlugin, PipPlugin, Plugin, PluginActionType, PluginRegistry,
    RunPlugin, RustupPlugin, SnapPlugin, UvPlugin, VscodePlugin, YarnPlugin,
};

fn create_test_config() -> Config {
    Config::new("/tmp/test")
}

fn create_test_insights() -> Insights {
    Insights::new().expect("Failed to create Insights")
}

fn create_test_logger(config: &Config) -> Logger {
    Logger::new(config)
}

#[test]
fn test_plugin_registry_creation() {
    let registry = PluginRegistry::new();
    assert_eq!(registry.plugins.len(), 0);
}

#[test]
fn test_plugin_registration() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(BrewPlugin));

    let plugin = registry.get_plugin("brew");
    assert!(plugin.is_some());
    assert_eq!(plugin.unwrap().name(), "brew");
}

#[test]
fn test_get_plugin_nonexistent() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(BrewPlugin));

    let plugin = registry.get_plugin("nonexistent");
    assert!(plugin.is_none());
}

#[test]
fn test_plugin_metadata() {
    let brew = BrewPlugin;
    let metadata = brew.get_metadata();

    assert_eq!(metadata.name, "brew");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_cargo() {
    let cargo = CargoPlugin;
    let metadata = cargo.get_metadata();

    assert_eq!(metadata.name, "cargo");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_deno() {
    let deno = DenoPlugin;
    let metadata = deno.get_metadata();

    assert_eq!(metadata.name, "deno");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_docker() {
    let docker = DockerPlugin;
    let metadata = docker.get_metadata();

    assert_eq!(metadata.name, "docker");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_flatpak() {
    let flatpak = FlatpakPlugin;
    let metadata = flatpak.get_metadata();

    assert_eq!(metadata.name, "flatpak");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_gem() {
    let gem = GemPlugin;
    let metadata = gem.get_metadata();

    assert_eq!(metadata.name, "gem");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
    assert!(metadata.actions.iter().any(|a| a.name == "gem"));
    assert!(metadata.actions.iter().any(|a| a.name == "gem-save"));
    assert!(metadata.actions.iter().any(|a| a.name == "gem-restore"));
}

#[test]
fn test_plugin_metadata_nvim() {
    let nvim = NvimPlugin;
    let metadata = nvim.get_metadata();

    assert_eq!(metadata.name, "nvim");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_os() {
    let os = OsPlugin;
    let metadata = os.get_metadata();

    assert_eq!(metadata.name, "os");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_rustup() {
    let rustup = RustupPlugin;
    let metadata = rustup.get_metadata();

    assert_eq!(metadata.name, "rustup");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_snap() {
    let snap = SnapPlugin;
    let metadata = snap.get_metadata();

    assert_eq!(metadata.name, "snap");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_vscode() {
    let vscode = VscodePlugin;
    let metadata = vscode.get_metadata();

    assert_eq!(metadata.name, "vscode");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_go() {
    let go = GoPlugin;
    let metadata = go.get_metadata();

    assert_eq!(metadata.name, "go");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_npm() {
    let npm = NpmPlugin;
    let metadata = npm.get_metadata();

    assert_eq!(metadata.name, "npm");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_pip() {
    let pip = PipPlugin;
    let metadata = pip.get_metadata();

    assert_eq!(metadata.name, "pip");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_ruby() {
    let gem = GemPlugin;
    let metadata = gem.get_metadata();

    assert_eq!(metadata.name, "gem");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_run() {
    let run = RunPlugin;
    let metadata = run.get_metadata();

    assert_eq!(metadata.name, "run");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_uv() {
    let uv = UvPlugin;
    let metadata = uv.get_metadata();

    assert_eq!(metadata.name, "uv");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_plugin_metadata_yarn() {
    let yarn = YarnPlugin;
    let metadata = yarn.get_metadata();

    assert_eq!(metadata.name, "yarn");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.actions.is_empty());
}

#[test]
fn test_get_action_by_name() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(BrewPlugin));
    registry.register(Box::new(CargoPlugin));

    let brew_action = registry.get_action_by_name("brew");
    assert!(brew_action.is_some());
    assert_eq!(brew_action.unwrap().name, "brew");

    let brew_save = registry.get_action_by_name("brew-save");
    assert!(brew_save.is_some());
    assert_eq!(brew_save.unwrap().name, "brew-save");

    let cargo_action = registry.get_action_by_name("cargo");
    assert!(cargo_action.is_some());

    let nonexistent = registry.get_action_by_name("nonexistent");
    assert!(nonexistent.is_none());
}

#[test]
fn test_get_all_metadata() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(BrewPlugin));
    registry.register(Box::new(CargoPlugin));
    registry.register(Box::new(DenoPlugin));
    registry.register(Box::new(DockerPlugin));
    registry.register(Box::new(FlatpakPlugin));
    registry.register(Box::new(GemPlugin));
    registry.register(Box::new(GoPlugin));
    registry.register(Box::new(NpmPlugin));
    registry.register(Box::new(NvimPlugin));
    registry.register(Box::new(OsPlugin));
    registry.register(Box::new(PipPlugin));
    registry.register(Box::new(RunPlugin));
    registry.register(Box::new(RustupPlugin));
    registry.register(Box::new(SnapPlugin));
    registry.register(Box::new(UvPlugin));
    registry.register(Box::new(VscodePlugin));
    registry.register(Box::new(YarnPlugin));

    let all_metadata = registry.get_all_metadata();
    assert_eq!(all_metadata.len(), 17);

    let names: Vec<&str> = all_metadata.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"brew"));
    assert!(names.contains(&"cargo"));
    assert!(names.contains(&"deno"));
    assert!(names.contains(&"docker"));
    assert!(names.contains(&"flatpak"));
    assert!(names.contains(&"gem"));
    assert!(names.contains(&"go"));
    assert!(names.contains(&"npm"));
    assert!(names.contains(&"nvim"));
    assert!(names.contains(&"os"));
    assert!(names.contains(&"pip"));
    assert!(names.contains(&"run"));
    assert!(names.contains(&"rustup"));
    assert!(names.contains(&"snap"));
    assert!(names.contains(&"uv"));
    assert!(names.contains(&"vscode"));
    assert!(names.contains(&"yarn"));
}

#[test]
fn test_action_types() {
    let brew = BrewPlugin;
    let metadata = brew.get_metadata();

    let update_action = metadata.actions.iter().find(|a| a.name == "brew");
    assert!(update_action.is_some());
    assert_eq!(
        update_action.unwrap().action_type,
        Some(PluginActionType::Update)
    );

    let save_action = metadata.actions.iter().find(|a| a.name == "brew-save");
    assert!(save_action.is_some());
    assert_eq!(
        save_action.unwrap().action_type,
        Some(PluginActionType::Save)
    );

    let restore_action = metadata.actions.iter().find(|a| a.name == "brew-restore");
    assert!(restore_action.is_some());
    assert_eq!(
        restore_action.unwrap().action_type,
        Some(PluginActionType::Restore)
    );
}

#[test]
fn test_find_similar_actions() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(BrewPlugin));
    registry.register(Box::new(CargoPlugin));

    let similar = registry.find_similar_actions("breq");
    assert!(!similar.is_empty());
    assert!(similar.iter().any(|s| s.contains("brew")));

    let similar = registry.find_similar_actions("brew");
    assert!(similar.iter().any(|s| s.starts_with("brew")));

    let similar = registry.find_similar_actions("xyzabc123");
    assert!(similar.is_empty());
}

#[tokio::test]
async fn test_custom_action_handler_default() {
    let brew = BrewPlugin;
    let config = create_test_config();
    let insights = create_test_insights();
    let mut logger = create_test_logger(&config);

    let result: Result<bool, _> = brew
        .handle_custom_action("custom-action", &config, &insights, &mut logger)
        .await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn test_get_all_action_names() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(BrewPlugin));

    let action_names = registry.get_all_action_names();

    assert!(action_names.contains(&"brew".to_string()));
    assert!(action_names.contains(&"brew-save".to_string()));
    assert!(action_names.contains(&"brew-restore".to_string()));

    assert!(action_names.contains(&"install".to_string()));
    assert!(action_names.contains(&"update".to_string()));
    assert!(action_names.contains(&"remove".to_string()));
    assert!(action_names.contains(&"install-completions".to_string()));
    assert!(action_names.contains(&"schedule enable".to_string()));
    assert!(action_names.contains(&"schedule disable".to_string()));
    assert!(action_names.contains(&"schedule check".to_string()));
    assert!(action_names.contains(&"trim-logfile".to_string()));
}
