use anyhow::{Context, Result};
use std::env;
use std::sync::OnceLock;

use clap::{CommandFactory, Parser};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap_complete::{Shell, generate};
use updatehauler::config::{
    Config, generate_sample_yaml, has_path_traversal, validate_schedule_value,
};
use updatehauler::insights::Insights;
use updatehauler::logger::Logger;
use updatehauler::scheduler::Scheduler;
use updatehauler::self_install::SelfInstaller;
use updatehauler::{
    plugins::BrewPlugin, plugins::CargoPlugin, plugins::DenoPlugin, plugins::DockerPlugin,
    plugins::FlatpakPlugin, plugins::GemPlugin, plugins::GoPlugin, plugins::NpmPlugin,
    plugins::NvimPlugin, plugins::OsPlugin, plugins::PipPlugin, plugins::PluginActionType,
    plugins::PluginRegistry, plugins::RunPlugin, plugins::RustupPlugin, plugins::SnapPlugin,
    plugins::UvPlugin, plugins::VscodePlugin, plugins::YarnPlugin, register_plugins,
};

fn get_help_text() -> &'static str {
    static HELP_TEXT: OnceLock<String> = OnceLock::new();
    HELP_TEXT.get_or_init(build_help_text)
}

fn build_help_text() -> String {
    let mut help = String::from(
        r#"
ACTIONS:

 Update Actions:
"#,
    );

    let registry = create_plugin_registry();

    for metadata in registry.get_all_metadata() {
        help.push_str(&format!(
            "   {:<20} {}\n",
            metadata.name, metadata.description
        ));

        for action in &metadata.actions {
            if action.name != metadata.name {
                help.push_str(&format!("   {:<20} {}\n", action.name, action.description));
            }
        }
    }

    help.push_str(
        r#"
 Run Action:
   run --cmd <command>   Run an arbitrary command (e.g., updatehauler run --cmd "echo hello")

 Schedule Actions:
   schedule enable      Enable scheduled updates (cron on Linux, launchd on macOS)
   schedule disable     Disable scheduled updates
   schedule check       Check current scheduling status

 Maintenance Actions:
   trim-logfile         Trim logfile to max lines

 Self-Installation Actions:
   install              Install this script to system
   update               Update this script on the system
   remove               Remove this script from system
   install-completions  Install shell completions (bash, zsh)

 Plugin Help:
   <plugin> help        Show detailed help for a specific plugin

 Default Actions (when no actions specified):
   Controlled by YAML config (varies by platform and installed tools)

 Examples:
   updatehauler                                           # Run all default actions
   updatehauler os                                        # Update OS packages only
   updatehauler brew brew-save                            # Update and save brew packages
   updatehauler nvim                                      # Update Neovim plugins
   updatehauler --debug                                   # Run with debug output
   updatehauler --no-datetime                             # Run without timestamps
   updatehauler --config ~/.config/updatehauler/config.yaml # Use custom config
   updatehauler schedule enable                           # Enable daily updates at 2 AM
   updatehauler run --cmd "echo hello"                    # Run arbitrary command
   updatehauler --list-plugins                            # List all plugins and their status
   updatehauler --only brew                               # Run only the brew plugin
   updatehauler brew help                                 # Show detailed help for brew plugin
   updatehauler install-completions                       # Install shell completions
   updatehauler --completionsdir ~/.local/share bash zsh  # Custom completion dir
 "#,
    );

    help
}

fn build_plugin_help(plugin_name: &str) -> String {
    let registry = create_plugin_registry();

    if let Some(plugin) = registry.get_plugin(plugin_name) {
        let metadata = plugin.get_metadata();
        let mut help = format!(
            r#"
Plugin: {}

Description: {}

Available Actions:
"#,
            metadata.name, metadata.description
        );

        for action in &metadata.actions {
            let action_type_str = match &action.action_type {
                Some(PluginActionType::Update) => " (update)",
                Some(PluginActionType::Save) => " (save)",
                Some(PluginActionType::Restore) => " (restore)",
                None => " (custom)",
            };
            help.push_str(&format!(
                "  {:<20} {}{}\n",
                action.name, action.description, action_type_str
            ));
        }

        help.push_str("\nExamples:\n");
        help.push_str(&format!(
            "  updatehauler {}                # {}\n",
            metadata.name,
            if !metadata.actions.is_empty() {
                &metadata.actions[0].description
            } else {
                "default action"
            }
        ));

        for action in &metadata.actions {
            if action.name != metadata.name {
                help.push_str(&format!(
                    "  updatehauler {:<20} # {}\n",
                    action.name, action.description
                ));
            }
        }

        help
    } else {
        let available: Vec<String> = registry
            .get_all_metadata()
            .iter()
            .map(|m| m.name.clone())
            .collect();
        format!(
            "Error: Unknown plugin '{}'\n\nAvailable plugins: {}\nRun 'updatehauler --help' for more information.",
            plugin_name,
            available.join(", ")
        )
    }
}

fn create_plugin_registry() -> PluginRegistry<'static> {
    let mut registry = PluginRegistry::new();
    register_plugins!(
        registry,
        BrewPlugin,
        CargoPlugin,
        DenoPlugin,
        DockerPlugin,
        FlatpakPlugin,
        GemPlugin,
        GoPlugin,
        NpmPlugin,
        NvimPlugin,
        OsPlugin,
        PipPlugin,
        RunPlugin,
        RustupPlugin,
        SnapPlugin,
        UvPlugin,
        VscodePlugin,
        YarnPlugin,
    );
    registry
}

fn generate_custom_bash_completion(config: &Config) -> String {
    format!(
        r#"#!/usr/bin/env bash
# Bash completion for {app_name}

_{app_name}() {{
    local cur prev words cword
        local commands="brew brew-save brew-restore brew-list brew-outdated brew-upgrade-pinned cargo cargo-save cargo-restore cargo-list cargo-outdated deno docker flatpak gem gem-save gem-restore go go-save go-restore npm npm-save npm-restore nvim nvim-save nvim-restore nvim-list nvim-clean nvim-health os pip pip-save pip-restore run rustup snap uv uv-save uv-restore uv-list uvx vscode yarn yarn-save yarn-restore init schedule install update remove install-completions trim-logfile"
    local schedule_commands="enable disable check"
    local shell_types="bash zsh fish powershell elvish"
    local flags="--debug --no-debug --datetime --no-datetime --header --no-header --color --no-color --logfile-only --dry-run --no-sudo --confirm-run --notify --logfile --max-log-lines --installdir --brew-save-file --cargo-save-file --npm-save-file --pip-save-file --uv-save-file --completionsdir --sched-minute --sched-hour --sched-day-of-month --sched-month --sched-day-of-week --config-file --cmd --list-plugins --only --enable-plugin --disable-plugin --help --version"

    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"
    words=("${{COMP_WORDS[@]}}")
    cword=$COMP_CWORD

    # If current word starts with -, show flags
    if [[ $cur == -* ]]; then
        COMPREPLY=($(compgen -W "$flags" -- "$cur"))
        return 0
    fi

    case "$cword" in
        1)
            COMPREPLY=($(compgen -W "$commands" -- "$cur"))
            ;;
        *)
            if [[ $prev == "schedule" ]]; then
                COMPREPLY=($(compgen -W "$schedule_commands" -- "$cur"))
            elif [[ $prev == "install-completions" ]]; then
                COMPREPLY=($(compgen -W "$shell_types" -- "$cur"))
            fi
            ;;
    esac
}}

complete -F _{app_name} {app_name}
"#,
        app_name = config.app_name
    )
}

fn generate_custom_zsh_completion(config: &Config) -> String {
    format!(
        r#"#compdef {app_name}

_{app_name}() {{
    local -a commands=(
        'brew:Update, upgrade, and clean brew formulas and casks'
        'brew-save:Save brew bundle to Brewfile'
        'brew-restore:Restore from brew bundle'
        'brew-list:List all installed brew formulas'
        'brew-outdated:Show outdated brew formulas and casks'
        'brew-upgrade-pinned:Upgrade only pinned brew formulas'
        'cargo:Upgrade cargo installed packages'
        'cargo-save:Save cargo packages to backup JSON'
        'cargo-restore:Restore cargo packages from backup JSON'
        'cargo-list:List all installed cargo packages'
        'cargo-outdated:Show outdated cargo packages'
        'deno:Upgrade the Deno runtime'
        'docker:Clean up unused Docker data'
        'flatpak:Update Flatpak applications'
        'gem:Update Ruby gems'
        'gem-save:Save installed Ruby gems list'
        'gem-restore:Restore Ruby gems from saved list'
        'nvim:Update Neovim plugins'
        'nvim-save:Save nvim plugin configuration'
        'nvim-restore:Restore nvim plugins'
        'nvim-list:List installed nvim plugins'
        'nvim-clean:Clean unused nvim plugins'
        'nvim-health:Check nvim plugin health'
        'npm:Update globally installed npm packages'
        'npm-save:Save globally installed npm packages to JSON'
        'npm-restore:Restore globally installed npm packages from JSON'
        'os:Update OS and app-based packages'
        'pip:Update pip packages'
        'pip-save:Save pip packages to requirements file'
        'pip-restore:Restore pip packages from requirements file'
        'run:Run an arbitrary command via --cmd'
        'rustup:Update Rust toolchains'
        'schedule:Manage scheduled updates'
        'snap:Update Snap packages'
        'uv:Update uv tools'
        'uv-save:Save uv tools to JSON'
        'uv-restore:Restore uv tools from JSON'
        'uv-list:List installed uv tools'
        'vscode:Update VSCode/Cursor extensions'
        'install:Install this script to system'
        'update:Update this script on the system'
        'remove:Remove this script from system'
        'install-completions:Install shell completions'
        'trim-logfile:Trim logfile to max lines'
    )

    local -a schedule_subcommands=(
        'enable:Enable scheduled updates (cron on Linux, launchd on macOS)'
        'disable:Disable scheduled updates'
        'check:Check current scheduling status'
    )

    local -a shell_types=(
        'bash:Generate bash completions'
        'zsh:Generate zsh completions'
        'fish:Generate fish completions'
        'powershell:Generate PowerShell completions'
        'elvish:Generate elvish completions'
    )

    _arguments \
        '--debug[Enable debug output]' \
        '--no-debug[Disable debug output (default)]' \
        '--datetime[Enable ISO8601 with microseconds (default)]' \
        '--no-datetime[Disable ISO8601 with microseconds]' \
        '--header[Enable header output (default)]' \
        '--no-header[Disable header output]' \
        '--color[Enable color output (default)]' \
        '--no-color[Disable color output]' \
        '--logfile-only[Enable output to only logfile]' \
        '--dry-run[Dry-run mode - show what would be done without making changes]' \
        '--no-sudo[Skip sudo elevation - run commands as current user]' \
        '--confirm-run[Prompt for confirmation before running arbitrary commands]' \
        '--notify[Send desktop notification when updates complete]' \
        '--logfile+[Logfile to use]:FILE:_files' \
        '--max-log-lines+[Max lines for logfile]:N:_numbers' \
        '--installdir+[Location to install this script]:DIR:_directories' \
        '--brew-save-file+[Brew save file location]:FILE:_files' \
        '--cargo-save-file+[Cargo save file location]:FILE:_files' \
        '--npm-save-file+[NPM save file location]:FILE:_files' \
        '--pip-save-file+[Pip save file location]:FILE:_files' \
        '--uv-save-file+[UV save file location]:FILE:_files' \
        '--completionsdir+[Completion install directory]:DIR:_directories' \
        '--sched-minute+[Schedule minute]:MIN:_numbers' \
        '--sched-hour+[Schedule hour]:HOUR:_numbers' \
        '--sched-day-of-month+[Schedule day of month]:DAY:_numbers' \
        '--sched-month+[Schedule month]:MONTH:_numbers' \
        '--sched-day-of-week+[Schedule day of week]:DAY:_numbers' \
        '--config-file+[YAML configuration file path]:FILE:_files' \
        '*--cmd+[Command to run with the run action]:CMD:_cmdstring' \
        '--list-plugins[List available plugins and their status]' \
        '--only+[Run only the specified plugin]:PLUGIN:(brew cargo deno docker flatpak gem npm nvim os pip rustup snap uv vscode)' \
        '(-h --help)'{{-h,--help}}'[Print help]' \
        '(-V --version)'{{-V,--version}}'[Print version]' \
        '*:: :->action_args'

    if (( CURRENT == 1 )); then
        _describe -t commands 'actions' commands
        return
    fi

    if (( CURRENT > 1 )); then
        local prev_cmd=$words[CURRENT-1]
        case $prev_cmd in
            schedule)
                _describe -t commands 'schedule subcommands' schedule_subcommands
                ;;
            install-completions)
                _describe -t commands 'shell types' shell_types
                ;;
            *)
                _describe -t commands 'actions' commands
                ;;
        esac
    fi
}}

_{app_name} "$@"
"#,
        app_name = config.app_name
    )
}

fn install_completions(config: &Config, shells: &[&str]) -> Result<()> {
    let mut cmd = Args::command();

    for shell in shells {
        let shell_enum = match *shell {
            "bash" => Shell::Bash,
            "zsh" => Shell::Zsh,
            "fish" => Shell::Fish,
            "powershell" => Shell::PowerShell,
            "elvish" => Shell::Elvish,
            _ => anyhow::bail!("Unsupported shell: {}", shell),
        };

        let mut completion_dir = config.completions_dir.clone();
        match *shell {
            "bash" => {
                completion_dir.push("bash-completion");
                completion_dir.push("completions");
            }
            "zsh" => {
                completion_dir.push("zsh");
                completion_dir.push("completions");
            }
            _ => {
                completion_dir.push("completions");
                completion_dir.push(shell);
            }
        }

        if !completion_dir.exists() {
            fs::create_dir_all(&completion_dir).context("Failed to create completion directory")?;
        }

        let filename = if shell == &"zsh" {
            format!("_{}", config.app_name)
        } else {
            format!("{}.bash", config.app_name)
        };

        let completion_path = completion_dir.join(filename);

        if shell == &"zsh" {
            let completion_content = generate_custom_zsh_completion(config);
            fs::write(&completion_path, completion_content)
                .context("Failed to write zsh completion")?;
        } else if shell == &"bash" {
            let completion_content = generate_custom_bash_completion(config);
            fs::write(&completion_path, completion_content)
                .context("Failed to write bash completion")?;
        } else {
            let mut buf = Vec::new();
            generate(shell_enum, &mut cmd, &config.app_name, &mut buf);
            fs::write(&completion_path, buf).context("Failed to write completion")?;
        }
        println!("Installed {} completions to {:?}", shell, completion_path);

        match *shell {
            "bash" => {
                println!("Add to ~/.bashrc: source {:?}", completion_path);
            }
            "zsh" => {
                println!(
                    "Add to ~/.zshrc: fpath=({:?} $fpath)",
                    completion_path.parent()
                );
            }
            _ => {}
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(
    name = "updatehauler",
    version = env!("CARGO_PKG_VERSION"),
    about = "System package update manager for macOS and Linux",
    long_about = None,
    after_help = get_help_text()
)]
struct Args {
    #[arg(long, help = "Enable debug output")]
    debug: bool,

    #[arg(long, action = clap::ArgAction::SetFalse, help = "Disable debug output (default)")]
    no_debug: bool,

    #[arg(
        long,
        default_value = "true",
        help = "Enable ISO8601 with microseconds (default)"
    )]
    datetime: bool,

    #[arg(long, help = "Disable ISO8601 with microseconds")]
    no_datetime: bool,

    #[arg(long, default_value = "true", help = "Enable header output (default)")]
    header: bool,

    #[arg(long, help = "Disable header output")]
    no_header: bool,

    #[arg(long, default_value = "true", help = "Enable color output (default)")]
    color: bool,

    #[arg(long, help = "Disable color output")]
    no_color: bool,

    #[arg(long, help = "Enable output to only logfile")]
    logfile_only: bool,

    #[arg(
        long,
        help = "Dry-run mode - show what would be done without making changes"
    )]
    dry_run: bool,

    #[arg(
        long,
        help = "Skip sudo elevation - run commands as current user"
    )]
    no_sudo: bool,

    #[arg(
        long,
        help = "Prompt for confirmation before running arbitrary commands"
    )]
    confirm_run: bool,

    #[arg(
        long,
        value_name = "FILE",
        help = "Logfile to use (default: ~/.local/updates.log)"
    )]
    logfile: Option<String>,

    #[arg(
        long,
        value_name = "N",
        help = "Max lines for logfile (default: 10000)"
    )]
    max_log_lines: Option<usize>,

    #[arg(
        long,
        value_name = "PATH",
        help = "Location to install this script (default: ~/.local/bin)"
    )]
    installdir: Option<String>,

    #[arg(
        long,
        value_name = "FILE",
        help = "Brew save file location (default: ~/.config/brew/{os}-Brewfile)"
    )]
    brew_save_file: Option<String>,

    #[arg(
        long,
        value_name = "FILE",
        help = "Cargo save file location (default: ~/.config/cargo/{os}-{arch}-cargo-backup.json)"
    )]
    cargo_save_file: Option<String>,

    #[arg(
        long,
        value_name = "FILE",
        help = "NPM save file location (default: ~/.config/npm/{os}-npm-packages.json)"
    )]
    npm_save_file: Option<String>,

    #[arg(
        long,
        value_name = "FILE",
        help = "Pip save file location (default: ~/.config/pip/{os}-pip-requirements.txt)"
    )]
    pip_save_file: Option<String>,

    #[arg(
        long,
        value_name = "FILE",
        help = "UV save file location (default: ~/.config/uv/{os}-uv-tools.json)"
    )]
    uv_save_file: Option<String>,

    #[arg(
        long,
        value_name = "DIR",
        help = "Completion install directory (default: ~/.local/share/bash-completion/completions for bash, ~/.local/share/zsh/completions for zsh)"
    )]
    completionsdir: Option<String>,

    #[arg(long, value_name = "MIN", help = "Schedule minute (default: 0)")]
    sched_minute: Option<String>,

    #[arg(long, value_name = "HOUR", help = "Schedule hour (default: 2)")]
    sched_hour: Option<String>,

    #[arg(long, value_name = "DAY", help = "Schedule day of month (default: *)")]
    sched_day_of_month: Option<String>,

    #[arg(long, value_name = "MONTH", help = "Schedule month (default: *)")]
    sched_month: Option<String>,

    #[arg(
        long,
        value_name = "DAY_OF_WEEK",
        help = "Schedule day of week (default: *)"
    )]
    sched_day_of_week: Option<String>,

    #[arg(
        long,
        value_name = "FILE",
        help = "YAML configuration file path (default: ~/.config/updatehauler/config.yaml)"
    )]
    config_file: Option<String>,

    #[arg(
        long,
        value_name = "CMD",
        num_args = 1..,
        allow_hyphen_values = true,
        help = "Command to run with the run action"
    )]
    cmd: Option<Vec<String>>,

    #[arg(long, help = "List available plugins and their enabled status")]
    list_plugins: bool,

    #[arg(
        long,
        value_name = "PLUGIN",
        help = "Run only the specified plugin (overrides default actions)"
    )]
    only: Option<String>,

    #[arg(
        long,
        value_name = "PLUGIN",
        help = "Enable a specific plugin (overrides config)"
    )]
    enable_plugin: Vec<String>,

    #[arg(
        long,
        value_name = "PLUGIN",
        help = "Disable a specific plugin (overrides config)"
    )]
    disable_plugin: Vec<String>,

    #[arg(long, help = "Send desktop notification when updates complete")]
    notify: bool,

    #[arg(
        value_name = "ACTION",
        help = "Action to perform",
        value_parser = clap::builder::PossibleValuesParser::new([
            "brew",
            "brew-save",
            "brew-restore",
            "brew-list",
            "brew-outdated",
            "brew-upgrade-pinned",
            "cargo",
            "cargo-save",
            "cargo-restore",
            "cargo-list",
            "cargo-outdated",
            "deno",
            "docker",
            "flatpak",
            "gem",
            "gem-save",
            "gem-restore",
            "go",
            "go-save",
            "go-restore",
            "nvim",
            "nvim-save",
            "nvim-restore",
            "nvim-list",
            "nvim-clean",
            "nvim-health",
            "npm",
            "npm-save",
            "npm-restore",
            "os",
            "pip",
            "pip-save",
            "pip-restore",
            "run",
            "rustup",
            "schedule",
            "snap",
            "uv",
            "uv-save",
            "uv-restore",
            "uv-list",
            "uvx",
            "vscode",
            "yarn",
            "yarn-save",
            "yarn-restore",
            "init",
            "install",
            "update",
            "remove",
            "install-completions",
            "trim-logfile",
            "bash",
            "zsh",
            "fish",
            "powershell",
            "elvish",
            "enable",
            "disable",
            "check",
            "help",
        ])
    )]
    actions: Vec<String>,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    let home = env::var("HOME").context("HOME environment variable not set")?;

    let config_path: Option<PathBuf> = args.config_file.as_deref().map(PathBuf::from);
    let mut config = Config::load_from_yaml(&home, config_path.as_ref())?;

    if args.debug {
        config.debug = true;
    }
    config.datetime = args.datetime && !args.no_datetime;
    config.show_header = args.header && !args.no_header;
    config.color = args.color && !args.no_color;
    if args.logfile_only {
        config.use_log = true;
    }
    config.dry_run = args.dry_run;
    config.no_sudo = args.no_sudo;
    config.confirm_run = args.confirm_run;
    config.notify = args.notify;
    if let Some(logfile) = args.logfile {
        let p = PathBuf::from(&logfile);
        if has_path_traversal(&p) {
            anyhow::bail!("--logfile path contains '..' traversal: {}", logfile);
        }
        config.log = p;
    }
    if let Some(max_lines) = args.max_log_lines {
        config.max_log_lines = max_lines;
    }
    if let Some(install_dir) = args.installdir {
        let p = PathBuf::from(&install_dir);
        if has_path_traversal(&p) {
            anyhow::bail!("--installdir path contains '..' traversal: {}", install_dir);
        }
        config.app_install_dir = p;
    }
    if let Some(completions_dir) = args.completionsdir {
        let p = PathBuf::from(&completions_dir);
        if has_path_traversal(&p) {
            anyhow::bail!(
                "--completionsdir path contains '..' traversal: {}",
                completions_dir
            );
        }
        config.completions_dir = p;
    }
    if let Some(sched_minute) = args.sched_minute {
        validate_schedule_value(&sched_minute, "--sched-minute")?;
        config.sched_minute = sched_minute;
    }
    if let Some(sched_hour) = args.sched_hour {
        validate_schedule_value(&sched_hour, "--sched-hour")?;
        config.sched_hour = sched_hour;
    }
    if let Some(sched_day_of_month) = args.sched_day_of_month {
        validate_schedule_value(&sched_day_of_month, "--sched-day-of-month")?;
        config.sched_day_of_month = sched_day_of_month;
    }
    if let Some(sched_month) = args.sched_month {
        validate_schedule_value(&sched_month, "--sched-month")?;
        config.sched_month = sched_month;
    }
    if let Some(sched_day_of_week) = args.sched_day_of_week {
        validate_schedule_value(&sched_day_of_week, "--sched-day-of-week")?;
        config.sched_day_of_week = sched_day_of_week;
    }

    let insights = Insights::new().context("Failed to detect system information")?;

    // Set default brew and cargo save file paths
    config.brew_file = config
        .brew_save_dir
        .join(format!("{}-Brewfile", insights.os));
    config.cargo_file = config.cargo_save_dir.join(format!(
        "{}-{}-cargo-backup.json",
        insights.os, insights.arch
    ));
    config.npm_file = PathBuf::from(&home)
        .join(".config/npm")
        .join(format!("{}-npm-packages.json", insights.os));
    config.pip_file = PathBuf::from(&home)
        .join(".config/pip")
        .join(format!("{}-pip-requirements.txt", insights.os));
    config.uv_file = PathBuf::from(&home)
        .join(".config/uv")
        .join(format!("{}-uv-tools.json", insights.os));

    // Override with command line options if provided
    if let Some(brew_file) = args.brew_save_file {
        let p = PathBuf::from(&brew_file);
        if has_path_traversal(&p) {
            anyhow::bail!(
                "--brew-save-file path contains '..' traversal: {}",
                brew_file
            );
        }
        config.brew_file = p;
    }
    if let Some(cargo_file) = args.cargo_save_file {
        let p = PathBuf::from(&cargo_file);
        if has_path_traversal(&p) {
            anyhow::bail!(
                "--cargo-save-file path contains '..' traversal: {}",
                cargo_file
            );
        }
        config.cargo_file = p;
    }
    if let Some(npm_file) = args.npm_save_file {
        let p = PathBuf::from(&npm_file);
        if has_path_traversal(&p) {
            anyhow::bail!("--npm-save-file path contains '..' traversal: {}", npm_file);
        }
        config.npm_file = p;
    }
    if let Some(pip_file) = args.pip_save_file {
        let p = PathBuf::from(&pip_file);
        if has_path_traversal(&p) {
            anyhow::bail!("--pip-save-file path contains '..' traversal: {}", pip_file);
        }
        config.pip_file = p;
    }
    if let Some(uv_file) = args.uv_save_file {
        let p = PathBuf::from(&uv_file);
        if has_path_traversal(&p) {
            anyhow::bail!("--uv-save-file path contains '..' traversal: {}", uv_file);
        }
        config.uv_file = p;
    }

    let rt = tokio::runtime::Runtime::new()?;

    let mut logger = Logger::new(&config);

    if let Some(cmd) = args.cmd
        && !cmd.is_empty()
    {
        let joined = cmd.join(" ");
        if joined.len() > 4096 {
            anyhow::bail!("--cmd argument too long (max 4096 bytes)");
        }
        if cmd.iter().any(|c| c.contains('\0')) {
            anyhow::bail!("--cmd argument contains null bytes");
        }
        config.cmd_args = cmd;
    }

    let mut actions = args.actions;

    let plugin_registry = create_plugin_registry();

    if args.list_plugins {
        println!(
            "{:<20} {:<10} {:<10}  Description",
            "Plugin", "Enabled", "Available"
        );
        println!("{:-<20} {:-<10} {:-<10}  {:-<40}", "", "", "", "");
        for metadata in plugin_registry.get_all_metadata() {
            let enabled = match metadata.name.as_str() {
                "brew" => config.plugins_enabled.brew.unwrap_or(false),
                "cargo" => config.plugins_enabled.cargo.unwrap_or(false),
                "deno" => config.plugins_enabled.deno.unwrap_or(false),
                "docker" => config.plugins_enabled.docker.unwrap_or(false),
                "flatpak" => config.plugins_enabled.flatpak.unwrap_or(false),
                "gem" => config.plugins_enabled.gem.unwrap_or(false),
                "nvim" => config.plugins_enabled.nvim.unwrap_or(false),
                "npm" => config.plugins_enabled.npm.unwrap_or(false),
                "os" => config.plugins_enabled.os.unwrap_or(false),
                "pip" => config.plugins_enabled.pip.unwrap_or(false),
                "rustup" => config.plugins_enabled.rustup.unwrap_or(false),
                "snap" => config.plugins_enabled.snap.unwrap_or(false),
                "uv" => config.plugins_enabled.uv.unwrap_or(false),
                "vscode" => config.plugins_enabled.vscode.unwrap_or(false),
                "yarn" => config.plugins_enabled.yarn.unwrap_or(false),
                "go" => config.plugins_enabled.go.unwrap_or(false),
                _ => true,
            };
            let plugin = plugin_registry.get_plugin(&metadata.name).unwrap();
            let available = rt.block_on(plugin.check_available(&config, &insights));
            println!(
                "{:<20} {:<10} {:<10}  {}",
                metadata.name,
                if enabled { "yes" } else { "no" },
                if available { "yes" } else { "no" },
                metadata.description
            );
        }
        return Ok(ExitCode::SUCCESS);
    }

    for plugin in &args.enable_plugin {
        if plugin_registry.get_plugin(plugin).is_some() {
            config.apply_plugin_enabled(plugin, true);
        } else {
            anyhow::bail!(
                "Unknown plugin: {} (valid: {})",
                plugin,
                plugin_registry
                    .get_all_metadata()
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    for plugin in &args.disable_plugin {
        if plugin_registry.get_plugin(plugin).is_some() {
            config.apply_plugin_enabled(plugin, false);
        } else {
            anyhow::bail!(
                "Unknown plugin: {} (valid: {})",
                plugin,
                plugin_registry
                    .get_all_metadata()
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    if let Some(ref only_plugin) = args.only {
        if plugin_registry.get_plugin(only_plugin).is_some() {
            config.only_plugin = Some(only_plugin.clone());
        } else {
            anyhow::bail!(
                "Unknown plugin: {} (valid: {})",
                only_plugin,
                plugin_registry
                    .get_all_metadata()
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    if actions.len() == 2 {
        let first = &actions[0];
        let second = &actions[1];

        if second == "--help" || second == "help" {
            if plugin_registry.get_plugin(first).is_some() {
                println!("{}", build_plugin_help(first));
                return Ok(ExitCode::SUCCESS);
            } else {
                eprintln!(
                    "Error: Unknown plugin '{}'\n\nAvailable plugins: {}\nRun 'updatehauler --help' for more information.",
                    first,
                    plugin_registry
                        .get_all_metadata()
                        .iter()
                        .map(|m| m.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return Ok(ExitCode::FAILURE);
            }
        }
    }

    let mut no_action = false;

    for action in &actions.clone() {
        match action.as_str() {
            "install" | "update" | "remove" | "install-completions" => {
                let installer = SelfInstaller::new(&config, &insights);
                match action.as_str() {
                    "install" => installer.install()?,
                    "update" => installer.update()?,
                    "remove" => installer.remove()?,
                    "install-completions" => {
                        let shells: Vec<&str> = actions
                            .iter()
                            .skip_while(|&a| a != "install-completions")
                            .skip(1)
                            .map(|s| s.as_str())
                            .collect();
                        if shells.is_empty() {
                            install_completions(&config, &["bash", "zsh"])?;
                        } else {
                            install_completions(&config, &shells.to_vec())?;
                        }
                        return Ok(ExitCode::SUCCESS);
                    }
                    _ => unreachable!(),
                }
                no_action = true;
            }
            "schedule" => {
                let mut scheduler = Scheduler::new(&config, &insights, &mut logger);
                if actions.len() < 2 {
                    anyhow::bail!("schedule requires qualifier: enable, disable, or check");
                }
                let qualifier = actions.get(1).unwrap().clone();
                match qualifier.as_str() {
                    "enable" => scheduler.enable()?,
                    "disable" => scheduler.disable()?,
                    "check" => scheduler.check()?,
                    _ => anyhow::bail!("Invalid schedule qualifier: {}", qualifier),
                }
                no_action = true;
            }
            "init" => {
                generate_config()?;
                no_action = true;
            }
            _ => {}
        }
    }

    if no_action {
        return Ok(ExitCode::SUCCESS);
    }

    if let Some(ref only_plugin) = config.only_plugin {
        actions.push(only_plugin.clone());
    } else if actions.is_empty() {
        if config.plugins_enabled.os.unwrap_or(true) {
            actions.push("os".to_string());
        }
        if config.plugins_enabled.brew.unwrap_or(false) && insights.has_brew {
            actions.extend_from_slice(&["brew".to_string(), "brew-save".to_string()]);
        }
        if config.plugins_enabled.cargo.unwrap_or(false) && insights.has_cargo {
            actions.extend_from_slice(&["cargo".to_string(), "cargo-save".to_string()]);
        }
        if config.plugins_enabled.nvim.unwrap_or(false) {
            actions.push("nvim".to_string());
        }
        if config.plugins_enabled.npm.unwrap_or(false) && insights.has_npm {
            actions.extend_from_slice(&["npm".to_string(), "npm-save".to_string()]);
        }
        if config.plugins_enabled.pip.unwrap_or(false) && insights.has_pip {
            actions.push("pip".to_string());
        }
        if config.plugins_enabled.rustup.unwrap_or(true) && insights.has_rustup {
            actions.push("rustup".to_string());
        }
        if config.plugins_enabled.flatpak.unwrap_or(false) && insights.has_flatpak {
            actions.push("flatpak".to_string());
        }
        if config.plugins_enabled.snap.unwrap_or(false) && insights.has_snap {
            actions.push("snap".to_string());
        }
        if config.plugins_enabled.vscode.unwrap_or(true) && insights.has_vscode {
            actions.push("vscode".to_string());
        }
        if config.plugins_enabled.docker.unwrap_or(false) && insights.has_docker {
            actions.push("docker".to_string());
        }
        if config.plugins_enabled.gem.unwrap_or(true) && insights.has_gem {
            actions.push("gem".to_string());
        }
        if config.plugins_enabled.deno.unwrap_or(true) && insights.has_deno {
            actions.push("deno".to_string());
        }
        if config.plugins_enabled.uv.unwrap_or(false) && insights.has_uv {
            actions.extend_from_slice(&["uv".to_string(), "uv-save".to_string()]);
        }
        if config.plugins_enabled.yarn.unwrap_or(false) && insights.has_yarn {
            actions.push("yarn".to_string());
        }
        if config.plugins_enabled.go.unwrap_or(false) && insights.has_go {
            actions.push("go".to_string());
        }
        if config.plugins_enabled.gem.unwrap_or(false) && insights.has_gem {
            actions.extend_from_slice(&["gem".to_string(), "gem-save".to_string()]);
        }
        actions.push("trim-logfile".to_string());
    }

    logger.log(&format!("{} Main → Start", config.app_name));

    let mut results: Vec<(&str, bool)> = Vec::new();

    for action in &actions {
        match action.as_str() {
            "trim-logfile" => {
                let r = trim_logfile(&config, &mut logger);
                results.push((action, r.is_ok()));
            }
            _ => {
                let r = rt.block_on(plugin_registry.execute_action(
                    action,
                    &config,
                    &insights,
                    &mut logger,
                ));
                results.push((action, r.is_ok()));
                if let Err(e) = r {
                    logger.error(&e.to_string());
                }
            }
        }
    }

    logger.log(&format!("{} Main → End", config.app_name));

    if config.show_header {
        let success_count = results.iter().filter(|(_, s)| *s).count();
        let fail_count = results.iter().filter(|(_, s)| !*s).count();
        logger.log(&format!(
            "{}   {} succeeded, {} failed",
            config.app_name, success_count, fail_count
        ));
        if !results.is_empty() {
            logger.log(&format!("{}   Summary:", config.app_name));
            for (action, ok) in &results {
                let status = if *ok { "OK" } else { "FAIL" };
                logger.log(&format!(
                    "{}     {:<25} {}",
                    config.app_name, action, status
                ));
            }
        }
    }

    let fail_count = results.iter().filter(|(_, s)| !*s).count();

    if fail_count > 0 {
        notify_result(&config, &insights, fail_count as u32);
        Ok(ExitCode::FAILURE)
    } else {
        notify_result(&config, &insights, 0);
        Ok(ExitCode::SUCCESS)
    }
}

fn notify_result(config: &Config, insights: &Insights, fail_count: u32) {
    if !config.notify {
        return;
    }
    let msg = if fail_count > 0 {
        format!("{}: {} action(s) failed", config.app_name, fail_count)
    } else {
        format!("{}: all actions completed successfully", config.app_name)
    };
    if insights.is_darwin {
        let escaped_msg = msg.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_title = config.app_name.replace('\\', "\\\\").replace('"', "\\\"");
        let _ = std::process::Command::new("/usr/bin/osascript")
            .args([
                "-e",
                &format!(
                    "display notification \"{}\" with title \"{}\"",
                    escaped_msg, escaped_title
                ),
            ])
            .output();
    } else if insights.is_linux {
        let _ = std::process::Command::new("notify-send")
            .args([&config.app_name, &msg])
            .output();
    }
}

fn generate_config() -> Result<()> {
    let home = env::var("HOME").context("HOME not set")?;
    let config_path = PathBuf::from(&home).join(".config/updatehauler/config.yaml");

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let yaml = generate_sample_yaml();
    std::fs::write(&config_path, &yaml)?;

    println!("Generated config file: {}", config_path.display());
    println!("Edit this file to customize updatehauler behavior.");
    Ok(())
}

fn trim_logfile(config: &Config, logger: &mut Logger) -> Result<()> {
    let log_path = &config.log;

    if !log_path.exists() {
        return Ok(());
    }

    let max_lines = config.max_log_lines;

    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
    let line_count = lines.len();

    if line_count <= max_lines {
        return Ok(());
    }

    let start = line_count.saturating_sub(max_lines);
    let trimmed: Vec<&String> = lines.iter().skip(start).collect();

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(log_path)?;

    for line in &trimmed {
        writeln!(file, "{}", line)?;
    }

    logger.log(&format!(
        "Successfully trimmed log {:?} to {}",
        log_path, max_lines
    ));

    Ok(())
}
