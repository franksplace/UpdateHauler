use anyhow::{Context, Result};
use std::env;
use std::sync::OnceLock;

use clap::{CommandFactory, Parser, Subcommand};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap_complete::{Shell, generate};
use updatehauler::config::{Config, has_path_traversal, validate_schedule_value};
use updatehauler::insights::Insights;
use updatehauler::logger::Logger;
use updatehauler::scheduler::Scheduler;
use updatehauler::self_install::SelfInstaller;
use updatehauler::{
    plugins::BrewPlugin, plugins::CargoPlugin, plugins::DenoPlugin, plugins::DockerPlugin,
    plugins::FlatpakPlugin, plugins::GemPlugin, plugins::GoPlugin, plugins::NpmPlugin,
    plugins::NvimPlugin, plugins::OsPlugin, plugins::PipPlugin, plugins::PluginRegistry,
    plugins::RunPlugin, plugins::RustupPlugin, plugins::SnapPlugin, plugins::UvPlugin,
    plugins::VscodePlugin, plugins::YarnPlugin, register_plugins,
};

fn get_help_text() -> &'static str {
    static HELP_TEXT: OnceLock<String> = OnceLock::new();
    HELP_TEXT.get_or_init(build_help_text)
}

fn build_help_text() -> String {
    r#"
 EXAMPLES:
   updatehauler                                            # Run all default actions
   updatehauler brew save                                  # Update and save brew packages
   updatehauler brew save --sudo --save-file ./Brewfile    # Brew with options
   updatehauler cargo save                                 # Update and save cargo packages
   updatehauler npm save                                   # Update and save npm packages
   updatehauler run --cmd "echo hello"                     # Run arbitrary command
   updatehauler schedule enable                            # Enable daily updates at 2 AM
   updatehauler schedule enable --hour 3 --minute 30       # Enable at custom time
   updatehauler config init                                # Generate config file
   updatehauler config compare                             # Compare config with defaults
   updatehauler config merge                               # Interactive merge config
   updatehauler --debug brew save                          # Run with debug output
   updatehauler --dry-run brew save                        # Preview changes
   updatehauler --list-plugins                             # List all plugins and status
   updatehauler install-completions bash zsh               # Install shell completions
"#
    .to_string()
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
    local plugins="brew cargo deno docker flatpak gem go npm nvim os pip run rustup snap uv vscode yarn"
    local plugin_actions_brew="update save restore list outdated upgrade-pinned"
    local plugin_actions_cargo="update save restore list outdated"
    local plugin_actions_npm="update save restore"
    local plugin_actions_pip="update save restore"
    local plugin_actions_uv="update save restore list uvx"
    local plugin_actions_nvim="update save restore list clean health"
    local plugin_actions_gem="update save restore"
    local plugin_actions_go="update save restore"
    local plugin_actions_yarn="update save restore"
    local plugin_actions_default="update"
    local global_commands="schedule config install update remove install-completions trim-logfile"
    local schedule_actions="enable disable check"
    local config_actions="init compare merge"
    local shell_types="bash zsh fish powershell elvish"
    local global_flags="--debug --no-debug --datetime --no-datetime --header --no-header --color --no-color --logfile-only --dry-run --no-sudo --confirm-run --notify --logfile --max-log-lines --installdir --completionsdir --config-file --list-plugins --only --enable-plugin --disable-plugin --help --version"
    local brew_flags="--save-file --sudo --info --search"
    local cargo_flags="--save-file"
    local npm_flags="--save-file"
    local pip_flags="--save-file"
    local uv_flags="--save-file"
    local run_flags="--cmd"

    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"
    words=("${{COMP_WORDS[@]}}")
    cword=$COMP_CWORD

    if [[ $cur == -* ]]; then
        if [[ $prev == "brew" ]]; then
            COMPREPLY=($(compgen -W "$global_flags $brew_flags" -- "$cur"))
        elif [[ $prev == "cargo" ]]; then
            COMPREPLY=($(compgen -W "$global_flags $cargo_flags" -- "$cur"))
        elif [[ $prev == "npm" ]]; then
            COMPREPLY=($(compgen -W "$global_flags $npm_flags" -- "$cur"))
        elif [[ $prev == "pip" ]]; then
            COMPREPLY=($(compgen -W "$global_flags $pip_flags" -- "$cur"))
        elif [[ $prev == "uv" ]]; then
            COMPREPLY=($(compgen -W "$global_flags $uv_flags" -- "$cur"))
        elif [[ $prev == "run" ]]; then
            COMPREPLY=($(compgen -W "$global_flags $run_flags" -- "$cur"))
        elif [[ $prev == "schedule" ]]; then
            COMPREPLY=($(compgen -W "--hour --minute --day-of-month --month --day-of-week" -- "$cur"))
        else
            COMPREPLY=($(compgen -W "$global_flags" -- "$cur"))
        fi
        return 0
    fi

    case "$cword" in
        1)
            COMPREPLY=($(compgen -W "$plugins $global_commands" -- "$cur"))
            ;;
        2)
            case $prev in
                brew) COMPREPLY=($(compgen -W "$plugin_actions_brew" -- "$cur")) ;;
                cargo) COMPREPLY=($(compgen -W "$plugin_actions_cargo" -- "$cur")) ;;
                npm) COMPREPLY=($(compgen -W "$plugin_actions_npm" -- "$cur")) ;;
                pip) COMPREPLY=($(compgen -W "$plugin_actions_pip" -- "$cur")) ;;
                uv) COMPREPLY=($(compgen -W "$plugin_actions_uv" -- "$cur")) ;;
                nvim) COMPREPLY=($(compgen -W "$plugin_actions_nvim" -- "$cur")) ;;
                gem) COMPREPLY=($(compgen -W "$plugin_actions_gem" -- "$cur")) ;;
                go) COMPREPLY=($(compgen -W "$plugin_actions_go" -- "$cur")) ;;
                yarn) COMPREPLY=($(compgen -W "$plugin_actions_yarn" -- "$cur")) ;;
                deno|docker|flatpak|os|rustup|snap|vscode)
                    COMPREPLY=($(compgen -W "$plugin_actions_default" -- "$cur")) ;;
                schedule) COMPREPLY=($(compgen -W "$schedule_actions" -- "$cur")) ;;
                config) COMPREPLY=($(compgen -W "$config_actions" -- "$cur")) ;;
                install-completions) COMPREPLY=($(compgen -W "$shell_types" -- "$cur")) ;;
            esac
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
    local -a plugins=(
        'brew:Update, upgrade, and clean brew formulas and casks'
        'cargo:Upgrade cargo installed packages'
        'deno:Upgrade the Deno runtime'
        'docker:Clean up unused Docker data'
        'flatpak:Update Flatpak applications'
        'gem:Update Ruby gems'
        'go:Update Go modules and tools'
        'npm:Update globally installed npm packages'
        'nvim:Update Neovim plugins'
        'os:Update OS and app-based packages'
        'pip:Update pip packages'
        'run:Run an arbitrary command via --cmd'
        'rustup:Update Rust toolchains'
        'snap:Update Snap packages'
        'uv:Update uv tools'
        'vscode:Update VSCode/Cursor extensions'
        'yarn:Update globally installed Yarn/PNPM packages'
    )

    local -a global_commands=(
        'schedule:Manage scheduled updates'
        'config:Manage configuration files'
        'install:Install this script to system'
        'update:Update this script on the system'
        'remove:Remove this script from system'
        'install-completions:Install shell completions'
        'trim-logfile:Trim logfile to max lines'
    )

    local -a brew_actions=(
        'update:Update and upgrade brew formulas and casks'
        'save:Save brew bundle to Brewfile'
        'restore:Restore from brew bundle'
        'list:List all installed brew formulas'
        'outdated:Show outdated brew formulas and casks'
        'upgrade-pinned:Upgrade only pinned brew formulas'
    )

    local -a cargo_actions=(
        'update:Upgrade cargo installed packages'
        'save:Save cargo packages to backup JSON'
        'restore:Restore cargo packages from backup JSON'
        'list:List all installed cargo packages'
        'outdated:Show outdated cargo packages'
    )

    local -a npm_actions=(
        'update:Update globally installed npm packages'
        'save:Save globally installed npm packages to JSON'
        'restore:Restore globally installed npm packages from JSON'
    )

    local -a pip_actions=(
        'update:Update pip packages'
        'save:Save pip packages to requirements file'
        'restore:Restore pip packages from requirements file'
    )

    local -a uv_actions=(
        'update:Update uv tools'
        'save:Save uv tools to JSON'
        'restore:Restore uv tools from JSON'
        'list:List installed uv tools'
        'uvx:Run a tool with uvx'
    )

    local -a nvim_actions=(
        'update:Update Neovim plugins'
        'save:Save nvim plugin configuration'
        'restore:Restore nvim plugins'
        'list:List installed nvim plugins'
        'clean:Clean unused nvim plugins'
        'health:Check nvim plugin health'
    )

    local -a gem_actions=(
        'update:Update Ruby gems'
        'save:Save installed Ruby gems list'
        'restore:Restore Ruby gems from saved list'
    )

    local -a go_actions=(
        'update:Update Go modules and tools'
        'save:Save Go modules list'
        'restore:Restore Go modules from saved list'
    )

    local -a yarn_actions=(
        'update:Update Yarn packages'
        'save:Save Yarn packages'
        'restore:Restore Yarn packages'
    )

    local -a simple_actions=(
        'update:Update packages'
    )

    local -a schedule_actions=(
        'enable:Enable scheduled updates (cron on Linux, launchd on macOS)'
        'disable:Disable scheduled updates'
        'check:Check current scheduling status'
    )

    local -a config_actions=(
        'init:Generate default config file'
        'compare:Compare default config with local config'
        'merge:Interactive merge from defaults into local config'
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
        '--completionsdir+[Completion install directory]:DIR:_directories' \
        '--config-file+[YAML configuration file path]:FILE:_files' \
        '--list-plugins[List available plugins and their status]' \
        '--only+[Run only the specified plugin]:PLUGIN:(brew cargo deno docker flatpak gem npm nvim os pip rustup snap uv vscode yarn)' \
        '*--enable-plugin+[Enable a specific plugin]:PLUGIN:(brew cargo deno docker flatpak gem npm nvim os pip rustup snap uv vscode yarn)' \
        '*--disable-plugin+[Disable a specific plugin]:PLUGIN:(brew cargo deno docker flatpak gem npm nvim os pip rustup snap uv vscode yarn)' \
        '(-h --help)'{{-h,--help}}'[Print help]' \
        '(-V --version)'{{-V,--version}}'[Print version]' \
        '*:: :->args'

    if (( CURRENT == 1 )); then
        local -a all_commands=("${{plugins[@]}}" "${{global_commands[@]}}")
        _describe -t commands 'command' all_commands
        return
    fi

    if (( CURRENT > 1 )); then
        local prev_cmd=$words[2]
        case $prev_cmd in
            brew) _describe -t actions 'action' brew_actions ;;
            cargo) _describe -t actions 'action' cargo_actions ;;
            npm) _describe -t actions 'action' npm_actions ;;
            pip) _describe -t actions 'action' pip_actions ;;
            uv) _describe -t actions 'action' uv_actions ;;
            nvim) _describe -t actions 'action' nvim_actions ;;
            gem) _describe -t actions 'action' gem_actions ;;
            go) _describe -t actions 'action' go_actions ;;
            yarn) _describe -t actions 'action' yarn_actions ;;
            deno|docker|flatpak|os|rustup|snap|vscode)
                _describe -t actions 'action' simple_actions ;;
            schedule) _describe -t actions 'action' schedule_actions ;;
            config) _describe -t actions 'action' config_actions ;;
            install-completions) _describe -t shells 'shell' shell_types ;;
        esac
    fi

    # Plugin-specific flags
    if (( CURRENT > 2 )); then
        local plugin=$words[2]
        local action=$words[3]
        case $plugin in
            brew)
                _arguments \
                    '--save-file+[Brew save file location]:FILE:_files' \
                    '--sudo[Use sudo for brew upgrade commands]' \
                    '--info[Show information about a brew formula]' \
                    '--search[Search for brew formulas]:query:'
                ;;
            cargo)
                _arguments '--save-file+[Cargo save file location]:FILE:_files' ;;
            npm)
                _arguments '--save-file+[NPM save file location]:FILE:_files' ;;
            pip)
                _arguments '--save-file+[Pip save file location]:FILE:_files' ;;
            uv)
                _arguments '--save-file+[UV save file location]:FILE:_files' ;;
            run)
                _arguments '*--cmd+[Command to run]:CMD:_cmdstring' ;;
            schedule)
                _arguments \
                    '--hour+[Schedule hour (0-23)]:HOUR:_numbers' \
                    '--minute+[Schedule minute (0-59)]:MIN:_numbers' \
                    '--day-of-month+[Schedule day of month]:DAY:_numbers' \
                    '--month+[Schedule month]:MONTH:_numbers' \
                    '--day-of-week+[Schedule day of week]:DAY:_numbers'
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

// ---------------------------------------------------------------------------
// CLI Definition
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(
    name = "updatehauler",
    version = env!("CARGO_PKG_VERSION"),
    about = "System package update manager for macOS and Linux",
    long_about = None,
    after_help = get_help_text()
)]
struct Args {
    // -- Global flags --
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

    #[arg(long, help = "Skip sudo elevation - run commands as current user")]
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

    #[arg(long, value_name = "DIR", help = "Completion install directory")]
    completionsdir: Option<String>,

    #[arg(long, value_name = "FILE", help = "YAML configuration file path")]
    config_file: Option<String>,

    #[arg(long, help = "List available plugins and their enabled status")]
    list_plugins: bool,

    #[arg(long, value_name = "PLUGIN", help = "Run only the specified plugin")]
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

    // -- Subcommand --
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // -- Plugin subcommands --
    #[command(about = "Update, upgrade, and clean brew formulas and casks")]
    Brew {
        /// Action to perform
        action: Option<String>,

        /// Brew save file location
        #[arg(long, value_name = "FILE")]
        save_file: Option<String>,

        /// Use sudo for brew upgrade commands
        #[arg(long)]
        sudo: bool,

        /// Show information about a brew formula or cask
        #[arg(long, value_name = "FORMULA")]
        info: Option<String>,

        /// Search for brew formulas and casks
        #[arg(long, value_name = "QUERY")]
        search: Option<String>,
    },

    #[command(about = "Upgrade cargo installed packages")]
    Cargo {
        /// Action to perform
        action: Option<String>,

        /// Cargo save file location
        #[arg(long, value_name = "FILE")]
        save_file: Option<String>,
    },

    #[command(about = "Upgrade the Deno runtime")]
    Deno {
        /// Action to perform
        action: Option<String>,
    },

    #[command(about = "Clean up unused Docker data")]
    Docker {
        /// Action to perform
        action: Option<String>,
    },

    #[command(about = "Update Flatpak applications")]
    Flatpak {
        /// Action to perform
        action: Option<String>,
    },

    #[command(about = "Update Ruby gems")]
    Gem {
        /// Action to perform
        action: Option<String>,
    },

    #[command(about = "Update Go modules and tools")]
    Go {
        /// Action to perform
        action: Option<String>,
    },

    #[command(about = "Update globally installed npm packages")]
    Npm {
        /// Action to perform
        action: Option<String>,

        /// NPM save file location
        #[arg(long, value_name = "FILE")]
        save_file: Option<String>,
    },

    #[command(about = "Update Neovim plugins")]
    Nvim {
        /// Action to perform
        action: Option<String>,
    },

    #[command(about = "Update OS and app-based packages")]
    Os {
        /// Action to perform
        action: Option<String>,
    },

    #[command(about = "Update pip packages")]
    Pip {
        /// Action to perform
        action: Option<String>,

        /// Pip save file location
        #[arg(long, value_name = "FILE")]
        save_file: Option<String>,
    },

    #[command(about = "Run an arbitrary command")]
    Run {
        /// Command to run
        #[arg(long, value_name = "CMD", num_args = 1.., allow_hyphen_values = true)]
        cmd: Vec<String>,
    },

    #[command(about = "Update Rust toolchains via rustup")]
    Rustup {
        /// Action to perform
        action: Option<String>,
    },

    #[command(about = "Update Snap packages")]
    Snap {
        /// Action to perform
        action: Option<String>,
    },

    #[command(about = "Update uv itself and upgrade all installed tools")]
    Uv {
        /// Action to perform
        action: Option<String>,

        /// UV save file location
        #[arg(long, value_name = "FILE")]
        save_file: Option<String>,
    },

    #[command(about = "Update VSCode/Cursor extensions")]
    Vscode {
        /// Action to perform
        action: Option<String>,
    },

    #[command(about = "Update globally installed Yarn/PNPM packages")]
    Yarn {
        /// Action to perform
        action: Option<String>,
    },

    // -- Non-plugin subcommands --
    #[command(about = "Manage scheduled updates")]
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },

    #[command(about = "Manage configuration files")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(about = "Install updatehauler to system")]
    Install,

    #[command(about = "Update installed updatehauler binary")]
    Update,

    #[command(about = "Remove updatehauler from system")]
    Remove,

    #[command(about = "Install shell completions")]
    InstallCompletions {
        /// Shell types to install completions for
        shells: Vec<String>,
    },

    #[command(about = "Trim logfile to maximum lines")]
    TrimLogfile,
}

#[derive(Subcommand, Debug)]
enum ScheduleAction {
    #[command(about = "Enable scheduled updates")]
    Enable {
        /// Schedule hour (0-23, default: 2)
        #[arg(long, value_name = "HOUR")]
        hour: Option<String>,

        /// Schedule minute (0-59, default: 0)
        #[arg(long, value_name = "MIN")]
        minute: Option<String>,

        /// Schedule day of month (1-31 or *, default: *)
        #[arg(long, value_name = "DAY")]
        day_of_month: Option<String>,

        /// Schedule month (1-12 or *, default: *)
        #[arg(long, value_name = "MONTH")]
        month: Option<String>,

        /// Schedule day of week (0-7 or *, default: *)
        #[arg(long, value_name = "DAY_OF_WEEK")]
        day_of_week: Option<String>,
    },
    #[command(about = "Disable scheduled updates")]
    Disable,
    #[command(about = "Check current scheduling status")]
    Check,
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    #[command(about = "Generate default config file")]
    Init,
    #[command(about = "Compare default config with local config")]
    Compare,
    #[command(about = "Interactive merge from defaults into local config")]
    Merge,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    let home = env::var("HOME").context("HOME environment variable not set")?;

    let config_path: Option<PathBuf> = args.config_file.as_deref().map(PathBuf::from);
    let mut config = Config::load_from_yaml(&home, config_path.as_ref())?;

    // Apply global flags to config
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
    if let Some(ref logfile) = args.logfile {
        let p = PathBuf::from(&logfile);
        if has_path_traversal(&p) {
            anyhow::bail!("--logfile path contains '..' traversal: {}", logfile);
        }
        config.log = p;
    }
    if let Some(max_lines) = args.max_log_lines {
        config.max_log_lines = max_lines;
    }
    if let Some(ref install_dir) = args.installdir {
        let p = PathBuf::from(&install_dir);
        if has_path_traversal(&p) {
            anyhow::bail!("--installdir path contains '..' traversal: {}", install_dir);
        }
        config.app_install_dir = p;
    }
    if let Some(ref completions_dir) = args.completionsdir {
        let p = PathBuf::from(&completions_dir);
        if has_path_traversal(&p) {
            anyhow::bail!(
                "--completionsdir path contains '..' traversal: {}",
                completions_dir
            );
        }
        config.completions_dir = p;
    }

    let insights = Insights::new().context("Failed to detect system information")?;

    if insights.is_cargo_install && args.installdir.is_some() {
        eprintln!("Warning: --installdir is ignored for cargo-installed binaries. Use `cargo install updatehauler` to manage installation.");
    }

    // Set default save file paths
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

    let rt = tokio::runtime::Runtime::new()?;
    let mut logger = Logger::new(&config);

    let plugin_registry = create_plugin_registry();

    // -- Handle subcommands --
    let mut actions: Vec<String> = Vec::new();

    match args.command {
        // No subcommand: populate default actions from config
        None => {
            // Handle --list-plugins
            if args.list_plugins {
                print_plugin_list(&config, &insights, &rt, &plugin_registry);
                return Ok(ExitCode::SUCCESS);
            }

            // Handle --enable-plugin / --disable-plugin
            apply_plugin_overrides(&args, &plugin_registry, &mut config)?;

            // Handle --only
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

            // Populate default actions from config (same as original)
            if let Some(ref only_plugin) = config.only_plugin {
                actions.push(only_plugin.clone());
            } else {
                populate_default_actions(&config, &insights, &mut actions);
            }
        }

        // Plugin subcommands
        Some(Commands::Brew {
            action,
            save_file,
            sudo,
            info,
            search,
        }) => {
            config.brew_sudo = sudo;
            apply_save_file(&save_file, &mut config.brew_file)?;
            if let Some(query) = search {
                actions.push(format!("brew-search:{}", query));
            } else if let Some(formula) = info {
                actions.push(format!("brew-info:{}", formula));
            } else {
                let act = action.as_deref().unwrap_or("update");
                actions.push(map_plugin_action("brew", act)?);
            }
        }
        Some(Commands::Cargo { action, save_file }) => {
            apply_save_file(&save_file, &mut config.cargo_file)?;
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("cargo", act)?);
        }
        Some(Commands::Deno { action }) => {
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("deno", act)?);
        }
        Some(Commands::Docker { action }) => {
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("docker", act)?);
        }
        Some(Commands::Flatpak { action }) => {
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("flatpak", act)?);
        }
        Some(Commands::Gem { action }) => {
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("gem", act)?);
        }
        Some(Commands::Go { action }) => {
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("go", act)?);
        }
        Some(Commands::Npm { action, save_file }) => {
            apply_save_file(&save_file, &mut config.npm_file)?;
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("npm", act)?);
        }
        Some(Commands::Nvim { action }) => {
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("nvim", act)?);
        }
        Some(Commands::Os { action }) => {
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("os", act)?);
        }
        Some(Commands::Pip { action, save_file }) => {
            apply_save_file(&save_file, &mut config.pip_file)?;
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("pip", act)?);
        }
        Some(Commands::Run { cmd }) => {
            if cmd.is_empty() {
                anyhow::bail!("run requires --cmd argument");
            }
            let joined = cmd.join(" ");
            if joined.len() > 4096 {
                anyhow::bail!("--cmd argument too long (max 4096 bytes)");
            }
            if cmd.iter().any(|c| c.contains('\0')) {
                anyhow::bail!("--cmd argument contains null bytes");
            }
            config.cmd_args = cmd;
            actions.push("run".to_string());
        }
        Some(Commands::Rustup { action }) => {
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("rustup", act)?);
        }
        Some(Commands::Snap { action }) => {
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("snap", act)?);
        }
        Some(Commands::Uv { action, save_file }) => {
            apply_save_file(&save_file, &mut config.uv_file)?;
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("uv", act)?);
        }
        Some(Commands::Vscode { action }) => {
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("vscode", act)?);
        }
        Some(Commands::Yarn { action }) => {
            let act = action.as_deref().unwrap_or("update");
            actions.push(map_plugin_action("yarn", act)?);
        }

        // Schedule subcommand
        Some(Commands::Schedule { action }) => {
            match action {
                ScheduleAction::Enable {
                    hour,
                    minute,
                    day_of_month,
                    month,
                    day_of_week,
                } => {
                    if let Some(v) = hour {
                        validate_schedule_value(&v, "--hour")?;
                        config.sched_hour = v;
                    }
                    if let Some(v) = minute {
                        validate_schedule_value(&v, "--minute")?;
                        config.sched_minute = v;
                    }
                    if let Some(v) = day_of_month {
                        validate_schedule_value(&v, "--day-of-month")?;
                        config.sched_day_of_month = v;
                    }
                    if let Some(v) = month {
                        validate_schedule_value(&v, "--month")?;
                        config.sched_month = v;
                    }
                    if let Some(v) = day_of_week {
                        validate_schedule_value(&v, "--day-of-week")?;
                        config.sched_day_of_week = v;
                    }
                    let mut scheduler = Scheduler::new(&config, &insights, &mut logger);
                    scheduler.enable()?
                }
                ScheduleAction::Disable => {
                    let mut scheduler = Scheduler::new(&config, &insights, &mut logger);
                    scheduler.disable()?
                }
                ScheduleAction::Check => {
                    let mut scheduler = Scheduler::new(&config, &insights, &mut logger);
                    scheduler.check()?
                }
            }
            return Ok(ExitCode::SUCCESS);
        }

        // Config subcommand
        Some(Commands::Config { action }) => {
            match action {
                ConfigAction::Init => Config::config_init(config_path.as_ref())?,
                ConfigAction::Compare => Config::config_compare(config_path.as_ref())?,
                ConfigAction::Merge => Config::config_merge(config_path.as_ref())?,
            }
            return Ok(ExitCode::SUCCESS);
        }

        // Self-install subcommands
        Some(Commands::Install) => {
            let installer = SelfInstaller::new(&config, &insights);
            installer.install()?;
            return Ok(ExitCode::SUCCESS);
        }
        Some(Commands::Update) => {
            let installer = SelfInstaller::new(&config, &insights);
            installer.update()?;
            return Ok(ExitCode::SUCCESS);
        }
        Some(Commands::Remove) => {
            let installer = SelfInstaller::new(&config, &insights);
            installer.remove()?;
            return Ok(ExitCode::SUCCESS);
        }

        // Install completions
        Some(Commands::InstallCompletions { shells }) => {
            if shells.is_empty() {
                install_completions(&config, &["bash", "zsh"])?;
            } else {
                install_completions(
                    &config,
                    &shells.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                )?;
            }
            return Ok(ExitCode::SUCCESS);
        }

        // Trim logfile
        Some(Commands::TrimLogfile) => {
            trim_logfile(&config, &mut logger)?;
            return Ok(ExitCode::SUCCESS);
        }
    }

    // -- Execute plugin actions --
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn map_plugin_action(plugin: &str, action: &str) -> Result<String> {
    match action {
        "update" => Ok(plugin.to_string()),
        "save" => Ok(format!("{}-save", plugin)),
        "restore" => Ok(format!("{}-restore", plugin)),
        "list" => Ok(format!("{}-list", plugin)),
        "outdated" => Ok(format!("{}-outdated", plugin)),
        "upgrade-pinned" => Ok(format!("{}-upgrade-pinned", plugin)),
        "clean" => Ok(format!("{}-clean", plugin)),
        "health" => Ok(format!("{}-health", plugin)),
        "uvx" => Ok("uvx".to_string()),
        other => {
            // Check if it's a valid action for this plugin
            let registry = create_plugin_registry();
            let full_name = format!("{}-{}", plugin, other);
            if registry.get_action_by_name(&full_name).is_some() {
                Ok(full_name)
            } else if registry.get_action_by_name(other).is_some() {
                Ok(other.to_string())
            } else {
                anyhow::bail!(
                    "Unknown action '{}' for plugin '{}'. Valid actions: update, save, restore, list, outdated",
                    other,
                    plugin
                );
            }
        }
    }
}

fn apply_save_file(file: &Option<String>, target: &mut PathBuf) -> Result<()> {
    if let Some(f) = file {
        let p = PathBuf::from(f);
        if has_path_traversal(&p) {
            anyhow::bail!("--save-file path contains '..' traversal: {}", f);
        }
        *target = p;
    }
    Ok(())
}

fn apply_plugin_overrides(
    args: &Args,
    registry: &PluginRegistry,
    config: &mut Config,
) -> Result<()> {
    for plugin in &args.enable_plugin {
        if registry.get_plugin(plugin).is_some() {
            config.apply_plugin_enabled(plugin, true);
        } else {
            anyhow::bail!(
                "Unknown plugin: {} (valid: {})",
                plugin,
                registry
                    .get_all_metadata()
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    for plugin in &args.disable_plugin {
        if registry.get_plugin(plugin).is_some() {
            config.apply_plugin_enabled(plugin, false);
        } else {
            anyhow::bail!(
                "Unknown plugin: {} (valid: {})",
                plugin,
                registry
                    .get_all_metadata()
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    Ok(())
}

fn populate_default_actions(config: &Config, insights: &Insights, actions: &mut Vec<String>) {
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

fn print_plugin_list(
    config: &Config,
    insights: &Insights,
    rt: &tokio::runtime::Runtime,
    registry: &PluginRegistry,
) {
    println!(
        "{:<20} {:<10} {:<10}  Description",
        "Plugin", "Enabled", "Available"
    );
    println!("{:-<20} {:-<10} {:-<10}  {:-<40}", "", "", "", "");
    for metadata in registry.get_all_metadata() {
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
        let plugin = registry.get_plugin(&metadata.name).unwrap();
        let available = rt.block_on(plugin.check_available(config, insights));
        println!(
            "{:<20} {:<10} {:<10}  {}",
            metadata.name,
            if enabled { "yes" } else { "no" },
            if available { "yes" } else { "no" },
            metadata.description
        );
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
