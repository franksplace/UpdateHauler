use anyhow::{Context, Result};
use std::env;

use clap::Parser;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::sync::mpsc;
use std::thread;

use update_hauler::config::Config;
use update_hauler::insights::Insights;
use update_hauler::logger::Logger;
use update_hauler::plugins::{
    BrewPlugin, CargoPlugin, NvimPlugin, OsPlugin, PluginActionType, PluginRegistry,
};
use update_hauler::scheduler::Scheduler;
use update_hauler::self_install::SelfInstaller;

fn build_help_text() -> &'static str {
    let mut help = String::from(
        r#"
ACTIONS:

Update Actions:
"#,
    );

    let registry = create_plugin_registry();

    for metadata in registry.get_all_metadata() {
        help.push_str(&format!(
            "  {:<18} {}\n",
            metadata.name, metadata.description
        ));

        for action in &metadata.actions {
            if action.name != metadata.name {
                help.push_str(&format!("  {:<18} {}\n", action.name, action.description));
            }
        }
    }

    help.push_str(
        r#"
Schedule Actions:
  schedule enable    Enable scheduled updates (cron on Linux, launchd on macOS)
  schedule disable   Disable scheduled updates
  schedule check     Check current scheduling status

Maintenance Actions:
  trim-logfile       Trim logfile to max lines

Self-Installation Actions:
  install            Install this script to system
  update             Update this script on the system
  remove             Remove this script from system
  install-completions Install shell completions (bash, zsh)

Plugin Help:
  <plugin> help      Show detailed help for a specific plugin

Default Actions (when no actions specified):
  Controlled by YAML config or: os, brew, cargo, brew-save, cargo-save, trim-logfile

Examples:
  updatehauler                       # Run all default actions
  updatehauler os                    # Update OS packages only
  updatehauler brew brew-save        # Update and save brew packages
  updatehauler nvim                  # Update Neovim plugins
  updatehauler --debug               # Run with debug output
  updatehauler --no-datetime         # Run without timestamps
  updatehauler --config ~/.config/updatehauler/config.yaml  # Use custom config
  updatehauler schedule enable       # Enable daily updates at 2 AM
  updatehauler --run "echo hello"    # Run arbitrary command
  updatehauler brew help            # Show detailed help for brew plugin
  updatehauler install-completions  # Install shell completions
"#,
    );

    Box::leak(help.into_boxed_str())
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
        format!(
            "Error: Unknown plugin '{}'\n\nAvailable plugins: brew, cargo, nvim, os\nRun 'updatehauler --help' for more information.",
            plugin_name
        )
    }
}

fn create_plugin_registry() -> PluginRegistry<'static> {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(BrewPlugin));
    registry.register(Box::new(CargoPlugin));
    registry.register(Box::new(NvimPlugin));
    registry.register(Box::new(OsPlugin));
    registry
}

#[derive(Parser, Debug)]
#[command(
    name = "updatehauler",
    version = env!("CARGO_PKG_VERSION"),
    about = "System package update manager for macOS and Linux",
    long_about = None,
    after_help = build_help_text()
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
        help = "Run arbitrary command"
    )]
    run: Option<Vec<String>>,

    #[arg(value_name = "ACTION", help = "Action to perform")]
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
    if let Some(logfile) = args.logfile {
        config.log = PathBuf::from(logfile);
    }
    if let Some(max_lines) = args.max_log_lines {
        config.max_log_lines = max_lines;
    }
    if let Some(install_dir) = args.installdir {
        config.app_install_dir = PathBuf::from(install_dir);
    }
    if let Some(sched_minute) = args.sched_minute {
        config.sched_minute = sched_minute;
    }
    if let Some(sched_hour) = args.sched_hour {
        config.sched_hour = sched_hour;
    }
    if let Some(sched_day_of_month) = args.sched_day_of_month {
        config.sched_day_of_month = sched_day_of_month;
    }
    if let Some(sched_month) = args.sched_month {
        config.sched_month = sched_month;
    }
    if let Some(sched_day_of_week) = args.sched_day_of_week {
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

    // Override with command line options if provided
    if let Some(brew_file) = args.brew_save_file {
        config.brew_file = PathBuf::from(brew_file);
    }
    if let Some(cargo_file) = args.cargo_save_file {
        config.cargo_file = PathBuf::from(cargo_file);
    }

    let rt = tokio::runtime::Runtime::new()?;

    let mut logger = Logger::new(&config);

    if let Some(run_cmd) = args.run {
        if !run_cmd.is_empty() {
            return execute_run_command(&run_cmd, &mut logger, &config);
        }
    }

    let mut actions = args.actions;

    let plugin_registry = create_plugin_registry();

    if actions.len() == 2 {
        let first = &actions[0];
        let second = &actions[1];

        if second == "--help" || second == "help" {
            if plugin_registry.get_plugin(first).is_some() {
                println!("{}", build_plugin_help(first));
                return Ok(ExitCode::SUCCESS);
            } else {
                eprintln!("Error: Unknown plugin '{}'\n\nAvailable plugins: brew, cargo, nvim, os\nRun 'updatehauler --help' for more information.", first);
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
                        installer.install_completions(&["bash", "zsh"])?;
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
            _ => {}
        }
    }

    if no_action {
        return Ok(ExitCode::SUCCESS);
    }

    if actions.is_empty() {
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
        actions.push("trim-logfile".to_string());
    }

    logger.log(&format!("{} Main → Start", config.app_name));

    for action in &actions {
        match action.as_str() {
            "trim-logfile" => {
                trim_logfile(&config, &mut logger)?;
            }
            _ => {
                if let Err(e) = rt.block_on(plugin_registry.execute_action(
                    action,
                    &config,
                    &insights,
                    &mut logger,
                )) {
                    logger.error(&e.to_string());
                }
            }
        }
    }

    logger.log(&format!("{} Main → End", config.app_name));

    Ok(ExitCode::SUCCESS)
}

fn execute_run_command(
    cmd_vec: &[String],
    logger: &mut Logger,
    config: &Config,
) -> Result<ExitCode> {
    if cmd_vec.is_empty() {
        return Ok(ExitCode::SUCCESS);
    }

    // Parse command and arguments
    // If only one element, split by whitespace (like shell script behavior)
    // If multiple elements, first is program, rest are arguments
    let (program, args) = if cmd_vec.len() == 1 {
        let parts: Vec<&str> = cmd_vec[0].split_whitespace().collect();
        if parts.is_empty() {
            return Ok(ExitCode::SUCCESS);
        }
        let args: Vec<&str> = parts[1..].to_vec();
        (parts[0], args)
    } else {
        let (program, args) = cmd_vec.split_first().unwrap();
        let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        (program.as_str(), args)
    };

    let cmd_str = cmd_vec.join(" ");
    if config.show_header {
        logger.log(&format!("{} → Start", cmd_str));
    }

    // Spawn the process
    let mut child = Command::new(program)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Get the stdout and stderr handles
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // Create a channel for the main thread to receive log messages
    let (sender, receiver) = mpsc::channel::<String>();

    // Clone the necessary values for the threads
    let sender_stdout = sender.clone();
    let sender_stderr = sender;

    // Spawn a thread to read stdout
    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let _ = sender_stdout.send(line);
        }
    });

    // Spawn a thread to read stderr
    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            let _ = sender_stderr.send(line);
        }
    });

    // Read log messages from the channel and log them in real-time
    for received in receiver {
        if !received.is_empty() {
            logger.log(&received);
        }
    }

    // Wait for both threads to complete
    let _ = stdout_thread.join();
    let _ = stderr_thread.join();

    // Wait for the process to complete and get the exit code
    let exit_code = child.wait()?.code().unwrap_or(0);

    if config.show_header {
        logger.log(&format!("{} → Return code {}", cmd_str, exit_code));
    }

    Ok(ExitCode::from(exit_code as u8))
}

fn trim_logfile(config: &Config, logger: &mut Logger) -> Result<()> {
    let log_path = &config.log;

    if !log_path.exists() {
        return Ok(());
    }

    let max_lines = config.max_log_lines;

    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let line_count = reader.lines().count();

    if line_count <= max_lines {
        return Ok(());
    }

    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

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
