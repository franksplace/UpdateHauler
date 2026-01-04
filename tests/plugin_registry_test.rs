use update_hauler::config::Config;
use update_hauler::insights::Insights;
use update_hauler::logger::Logger;
use update_hauler::plugins::levenshtein_distance;
use update_hauler::plugins::{
    BrewPlugin, CargoPlugin, NvimPlugin, OsPlugin, Plugin, PluginActionType, PluginRegistry,
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
fn test_get_action_by_name() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(BrewPlugin));
    registry.register(Box::new(CargoPlugin));

    // Test existing actions
    let brew_action = registry.get_action_by_name("brew");
    assert!(brew_action.is_some());
    assert_eq!(brew_action.unwrap().name, "brew");

    let brew_save = registry.get_action_by_name("brew-save");
    assert!(brew_save.is_some());
    assert_eq!(brew_save.unwrap().name, "brew-save");

    let cargo_action = registry.get_action_by_name("cargo");
    assert!(cargo_action.is_some());

    // Test nonexistent action
    let nonexistent = registry.get_action_by_name("nonexistent");
    assert!(nonexistent.is_none());
}

#[test]
fn test_get_all_metadata() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(BrewPlugin));
    registry.register(Box::new(CargoPlugin));

    let all_metadata = registry.get_all_metadata();
    assert_eq!(all_metadata.len(), 2);

    let names: Vec<&str> = all_metadata.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"brew"));
    assert!(names.contains(&"cargo"));
}

#[test]
fn test_action_types() {
    let brew = BrewPlugin;
    let metadata = brew.get_metadata();

    // Find action by name and check types
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
fn test_levenshtein_distance() {
    // Exact match
    assert_eq!(levenshtein_distance("brew", "brew"), 0);

    // One character off
    assert_eq!(levenshtein_distance("brew", "breq"), 1);
    assert_eq!(levenshtein_distance("brew", "brea"), 1);

    // Two characters off
    assert_eq!(levenshtein_distance("brew", "brxy"), 2);

    // Completely different
    assert!(levenshtein_distance("brew", "cargo") > 3);

    // Empty strings
    assert_eq!(levenshtein_distance("", ""), 0);
    assert_eq!(levenshtein_distance("", "brew"), 4);
    assert_eq!(levenshtein_distance("brew", ""), 4);
}

#[test]
fn test_find_similar_actions() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(BrewPlugin));
    registry.register(Box::new(CargoPlugin));

    // Test typo - one character off
    let similar = registry.find_similar_actions("breq");
    assert!(!similar.is_empty());
    assert!(similar.iter().any(|s| s.contains("brew")));

    // Test prefix match
    let similar = registry.find_similar_actions("brew");
    assert!(similar.iter().any(|s| s.starts_with("brew")));

    // Test non-similar action
    let similar = registry.find_similar_actions("xyzabc123");
    assert!(similar.is_empty());
}

#[tokio::test]
async fn test_custom_action_handler_default() {
    let brew = BrewPlugin;
    let config = create_test_config();
    let insights = create_test_insights();
    let mut logger = create_test_logger(&config);

    // Default implementation should return false (not handled)
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

    // Should include plugin actions
    assert!(action_names.contains(&"brew".to_string()));
    assert!(action_names.contains(&"brew-save".to_string()));
    assert!(action_names.contains(&"brew-restore".to_string()));

    // Should include non-plugin commands
    assert!(action_names.contains(&"install".to_string()));
    assert!(action_names.contains(&"update".to_string()));
    assert!(action_names.contains(&"remove".to_string()));
    assert!(action_names.contains(&"install-completions".to_string()));
    assert!(action_names.contains(&"schedule enable".to_string()));
    assert!(action_names.contains(&"schedule disable".to_string()));
    assert!(action_names.contains(&"schedule check".to_string()));
    assert!(action_names.contains(&"trim-logfile".to_string()));
}
