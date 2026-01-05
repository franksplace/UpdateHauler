#[allow(unused_imports)]
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::insights::Insights;

pub struct SelfInstaller {
    config: Config,
    insights: Insights,
}

impl SelfInstaller {
    pub fn new(config: &Config, insights: &Insights) -> Self {
        Self {
            config: config.clone(),
            insights: insights.clone(),
        }
    }

    pub fn install(&self) -> Result<()> {
        self.copy_binary("Installing", "Installed", "to install")
    }

    pub fn update(&self) -> Result<()> {
        self.copy_binary("Updating", "Updated", "to update")
    }

    pub fn remove(&self) -> Result<()> {
        let install_path = self.config.app_install_dir.join(&self.config.app_name);

        if install_path.exists() {
            println!("Removing {:?}", install_path);

            fs::remove_file(&install_path)?;

            println!("Successfully removed {}", self.config.app_name);
        } else {
            println!("{:?} is not installed", install_path);
        }

        Ok(())
    }

    pub fn install_completions(&self, shells: &[&str]) -> Result<()> {
        for shell in shells {
            let completion_path = self.get_completion_dir(shell)?;

            match *shell {
                "bash" => {
                    let bash_completion = self.generate_bash_completion();
                    fs::write(&completion_path, bash_completion)
                        .context("Failed to write bash completion")?;
                    println!("Installed bash completions to {:?}", completion_path);
                    println!("Add to ~/.bashrc: source {:?}", completion_path);
                }
                "zsh" => {
                    let zsh_completion = self.generate_zsh_completion();
                    fs::write(&completion_path, zsh_completion)
                        .context("Failed to write zsh completion")?;
                    println!("Installed zsh completions to {:?}", completion_path);
                    println!(
                        "Add to ~/.zshrc: fpath=({:?} $fpath)",
                        completion_path.parent()
                    );
                }
                _ => {
                    println!("Unsupported shell: {}. Supported: bash, zsh", shell);
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::useless_format)]
    fn generate_bash_completion(&self) -> String {
        format!(
            r#"#!/usr/bin/env bash
_updatehauler_completion() {{
    local cur prev words cword
    _init_completion || return
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"
    words=("${{COMP_WORDS[@]}}")
    cword=$COMP_CWORD
    case "$cword" in
        0)
            COMPREPLY=($(compgen -W "brew cargo nvim os" -- "$cur"))
            ;;
        1)
            case "$prev" in
                brew)
                    local brew_actions="brew brew-save brew-restore"
                    COMPREPLY=($(compgen -W "$brew_actions" -- "$cur"))
                    ;;
                cargo)
                    local cargo_actions="cargo cargo-save cargo-restore"
                    COMPREPLY=($(compgen -W "$cargo_actions" -- "$cur"))
                    ;;
                nvim)
                    local nvim_actions="nvim nvim-save nvim-restore"
                    COMPREPLY=($(compgen -W "$nvim_actions" -- "$cur"))
                    ;;
                os)
                    local os_actions="os"
                    COMPREPLY=($(compgen -W "$os_actions" -- "$cur"))
                    ;;
                *)
                    COMPREPLY=($(compgen -W "--debug --no-debug --datetime --no-datetime --header --no-header --color --no-color --logfile-only --dry-run --logfile --max-log-lines --installdir --brew-save-file --cargo-save-file --sched-minute --sched-hour --sched-day-of-month --sched-month --sched-day-of-week --config-file --run -h --help -V --version" -- "$cur"))
                    ;;
            esac
            ;;
        *)
            COMPREPLY=($(compgen -W "--debug --no-debug --datetime --no-datetime --header --no-header --color --no-color --logfile-only --dry-run --logfile --max-log-lines --installdir --brew-save-file --cargo-save-file --sched-minute --sched-hour --sched-day-of-month --sched-month --sched-day-of-week --config-file --run -h --help -V --version" -- "$cur"))
            ;;
    esac
}}
complete -F _updatehauler_completion updatehauler
"#
        )
    }

    #[allow(clippy::useless_format)]
    fn generate_zsh_completion(&self) -> String {
        format!(
            r#"#compdef _updatehauler
_updatehauler() {{
    local -a commands
    local -a command_actions
    local -a opts=(
        '--help[Show help information]'
        '--debug[Enable debug output]'
        '--no-debug[Disable debug output]'
        '--datetime[Enable timestamps]'
        '--no-datetime[Disable timestamps]'
        '--header[Enable header output]'
        '--no-header[Disable header output]'
        '--color[Enable color output]'
        '--no-color[Disable color output]'
        '--logfile-only[Output only to logfile]'
        '--dry-run[Preview changes without executing]'
        '--logfile[Logfile path]'
        '--max-log-lines[Max logfile lines]'
        '--installdir[Installation directory]'
        '--brew-save-file[Brew save file path]'
        '--cargo-save-file[Cargo save file path]'
        '--sched-minute[Schedule minute]'
        '--sched-hour[Schedule hour]'
        '--sched-day-of-month[Schedule day of month]'
        '--sched-month[Schedule month]'
        '--sched-day-of-week[Schedule day of week]'
        '--config-file[Config file path]'
        '--run[Run arbitrary command]'
        '*:file:_files'
    )
    commands=(
        'brew:Update, upgrade, and clean brew formulas and casks'
        'cargo:Upgrade cargo installed packages'
        'nvim:Update Neovim plugins'
        'os:Update OS & app based packages'
    )
    command_actions=(
        'brew:Update, upgrade, and clean brew formulas and casks'
        'brew-save:Save brew bundle to Brewfile'
        'brew-restore:Restore from brew bundle'
        'cargo:Upgrade cargo installed packages'
        'cargo-save:Save cargo packages to backup JSON'
        'cargo-restore:Restore cargo packages from backup JSON'
        'nvim:Update Neovim plugins'
        'nvim-save:Save nvim plugin configuration'
        'nvim-restore:Restore nvim plugins'
    )
    if (( CURRENT == 1 )); then
        _describe -t commands 'commands' $words[2]
    else
        case $words[2] in
            brew)
                _describe -t command_actions 'brew actions' $words[3]
                ;;
            cargo)
                _describe -t command_actions 'cargo actions' $words[3]
                ;;
            nvim)
                _describe -t command_actions 'nvim actions' $words[3]
                ;;
            *)
                _arguments $opts && ret=0
                ;;
        esac
    fi
}}
"#
        )
    }

    fn get_completion_dir(&self, shell: &str) -> Result<PathBuf> {
        let mut completion_dir = self.config.completions_dir.clone();

        match shell {
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

        let filename = if shell == "zsh" {
            format!("_{}", self.config.app_name)
        } else {
            format!("{}.bash", self.config.app_name)
        };

        Ok(completion_dir.join(filename))
    }

    fn copy_binary(&self, prefix: &str, success_ending: &str, _error_ending: &str) -> Result<()> {
        let install_path = self.config.app_install_dir.join(&self.config.app_name);
        let source_path = &self.insights.app_abspath;

        let needs_install =
            !install_path.exists() || !Self::files_equal(source_path, &install_path)?;

        if needs_install {
            println!("{} {:?}", prefix, install_path);

            if !self.config.app_install_dir.exists() {
                fs::create_dir_all(&self.config.app_install_dir)
                    .context("Failed to create install directory")?;
            }

            fs::copy(source_path, &install_path).context("Failed to copy binary")?;

            println!("Successfully {} {}", success_ending, self.config.app_name);
        } else {
            println!("{:?} is already installed and up to date", install_path);
        }

        Ok(())
    }

    pub fn files_equal(path1: &Path, path2: &Path) -> Result<bool> {
        let content1 = fs::read(path1)?;
        let content2 = fs::read(path2)?;

        Ok(content1 == content2)
    }
}
