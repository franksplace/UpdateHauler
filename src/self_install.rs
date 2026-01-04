#[allow(unused_imports)]
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

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
            match *shell {
                "bash" => {
                    let completion_path = self.get_completion_dir("bash")?;
                    let bash_completion = "#!/usr/bin/env bash\n_updatehauler_completion() {\n    local cur prev words cword\n    _init_completion || return\n    cur=\"${COMP_WORDS[COMP_CWORD]}\"\n    prev=\"${COMP_WORDS[COMP_CWORD-1]}\"\n    words=(\"${COMP_WORDS[@]}\")\n    cword=$COMP_CWORD\n    case \"$cword\" in\n        0)\n            COMPREPLY=($(compgen -W \"brew cargo nvim os\" -- \"$cur\"))\n            ;;\n        1)\n            case \"$prev\" in\n                brew)\n                    local brew_actions=\"brew brew-save brew-restore\"\n                    COMPREPLY=($(compgen -W \"$brew_actions\" -- \"$cur\"))\n                    ;;\n                cargo)\n                    local cargo_actions=\"cargo cargo-save cargo-restore\"\n                    COMPREPLY=($(compgen -W \"$cargo_actions\" -- \"$cur\"))\n                    ;;\n                nvim)\n                    local nvim_actions=\"nvim nvim-save nvim-restore\"\n                    COMPREPLY=($(compgen -W \"$nvim_actions\" -- \"$cur\"))\n                    ;;\n                os)\n                    local os_actions=\"os\"\n                    COMPREPLY=($(compgen -W \"$os_actions\" -- \"$cur\"))\n                    ;;\n                *)\n                    COMPREPLY=($(compgen -W \"--debug --no-debug --datetime --no-datetime --header --no-header --color --no-color --logfile-only --dry-run --logfile --max-log-lines --installdir --brew-save-file --cargo-save-file --sched-minute --sched-hour --sched-day-of-month --sched-month --sched-day-of-week --config-file --run -h --help -V --version\" -- \"$cur\"))\n                    ;;\n            esac\n            ;;\n        *)\n            COMPREPLY=($(compgen -W \"--debug --no-debug --datetime --no-datetime --header --no-header --color --no-color --logfile-only --dry-run --logfile --max-log-lines --installdir --brew-save-file --cargo-save-file --sched-minute --sched-hour --sched-day-of-month --sched-month --sched-day-of-week --config-file --run -h --help -V --version\" -- \"$cur\"))\n            ;;\n    esac\n}\ncomplete -F _updatehauler_completion updatehauler\n";
                    fs::write(&completion_path, bash_completion)
                        .context("Failed to write bash completion")?;
                    println!("Installed bash completions to {:?}", completion_path);
                    println!("Add to ~/.bashrc: source {:?}", completion_path);
                }
                "zsh" => {
                    let completion_path = self.get_completion_dir("zsh")?;
                    let zsh_completion = "#compdef updatehauler\n_updatehauler() {\n    local -a commands\n    local -a command_actions\n    local -a opts=(\n        '--help[Show help information]'\n        '--debug[Enable debug output]'\n        '--no-debug[Disable debug output]'\n        '--datetime[Enable timestamps]'\n        '--no-datetime[Disable timestamps]'\n        '--header[Enable header output]'\n        '--no-header[Disable header output]'\n        '--color[Enable color output]'\n        '--no-color[Disable color output]'\n        '--logfile-only[Output only to logfile]'\n        '--dry-run[Preview changes without executing]'\n        '--logfile[Logfile path]'\n        '--max-log-lines[Max logfile lines]'\n        '--installdir[Installation directory]'\n        '--brew-save-file[Brew save file path]'\n        '--cargo-save-file[Cargo save file path]'\n        '--sched-minute[Schedule minute]'\n        '--sched-hour[Schedule hour]'\n        '--sched-day-of-month[Schedule day of month]'\n        '--sched-month[Schedule month]'\n        '--sched-day-of-week[Schedule day of week]'\n        '--config-file[Config file path]'\n        '--run[Run arbitrary command]'\n        '*:file:_files'\n    )\n    commands=(\n        'brew:Update, upgrade, and clean brew formulas and casks'\n        'cargo:Upgrade cargo installed packages'\n        'nvim:Update Neovim plugins'\n        'os:Update OS & app based packages'\n    )\n    command_actions=(\n        'brew:Update, upgrade, and clean brew formulas and casks'\n        'brew-save:Save brew bundle to Brewfile'\n        'brew-restore:Restore from brew bundle'\n        'cargo:Upgrade cargo installed packages'\n        'cargo-save:Save cargo packages to backup JSON'\n        'cargo-restore:Restore cargo packages from backup JSON'\n        'nvim:Update Neovim plugins'\n        'nvim-save:Save nvim plugin configuration'\n        'nvim-restore:Restore nvim plugins'\n    )\n    if (( CURRENT == 1 )); then\n        _describe -V commands\n    else\n        case $words[1] in\n            --help|-h)\n                _describe -V command_actions\n                ;;\n            *)\n                _files\n                ;;\n        esac\n    fi\n}\n";
                    fs::write(&completion_path, zsh_completion)
                        .context("Failed to write zsh completion")?;
                    println!("Installed zsh completions to {:?}", completion_path);
                    println!("Add to ~/.zshrc: fpath={:?}", completion_path);
                }
                _ => {
                    println!("Unsupported shell: {}. Supported: bash, zsh", shell);
                }
            }
        }

        Ok(())
    }

    fn get_completion_dir(&self, shell: &str) -> Result<std::path::PathBuf> {
        let mut completion_dir = self.config.app_install_dir.join("completions");
        completion_dir.push(shell);

        if !completion_dir.exists() {
            fs::create_dir_all(&completion_dir).context("Failed to create completion directory")?;
        }

        Ok(completion_dir.join(format!("{}.{}", self.config.app_name, shell)))
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
