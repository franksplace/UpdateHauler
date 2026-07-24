// Integration tests for the main application

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::Command;

    fn get_updatehauler_binary() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("target/debug/updatehauler");
        path
    }

    #[test]
    fn test_help_flag() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .arg("--help")
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success());
        assert!(stdout.contains("Usage:"));
        assert!(stdout.contains("OPTIONS"));
        assert!(stdout.contains("brew"));
        assert!(stdout.contains("cargo"));
        assert!(stdout.contains("schedule"));
    }

    #[test]
    fn test_version_or_about() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .arg("--help")
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(stdout.contains("System package update manager"));
    }

    #[test]
    fn test_invalid_action() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .arg("invalid-action")
            .output()
            .expect("Failed to execute updatehauler");

        assert!(!output.status.success());
    }

    #[test]
    fn test_run_command() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["run", "--cmd", "echo", "test"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(output.status.success());
        assert!(stdout.contains("test") || stderr.contains("test"));
    }

    #[test]
    fn test_color_flag() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["--color", "run", "--cmd", "echo", "test"])
            .output()
            .expect("Failed to execute updatehauler");

        assert!(output.status.success());
    }

    #[test]
    fn test_no_color_flag() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["--no-color", "run", "--cmd", "echo", "test"])
            .output()
            .expect("Failed to execute updatehauler");

        assert!(output.status.success());
    }

    #[test]
    fn test_datetime_flag() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["--datetime", "run", "--cmd", "echo", "test"])
            .output()
            .expect("Failed to execute updatehauler");

        assert!(output.status.success());
    }

    #[test]
    fn test_plugin_help_brew() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["brew", "--help"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success());
        assert!(stdout.contains("brew"));
        assert!(stdout.contains("Update, upgrade"));
    }

    #[test]
    fn test_plugin_help_cargo() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["cargo", "--help"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success());
        assert!(stdout.contains("cargo"));
        assert!(stdout.contains("Upgrade cargo"));
    }

    #[test]
    fn test_plugin_help_nvim() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["nvim", "--help"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success());
        assert!(stdout.contains("nvim"));
        assert!(stdout.contains("Neovim"));
    }

    #[test]
    fn test_plugin_help_os() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["os", "--help"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success());
        assert!(stdout.contains("os"));
        assert!(stdout.contains("OS"));
    }

    #[test]
    fn test_plugin_help_invalid() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["invalid-plugin", "--help"])
            .output()
            .expect("Failed to execute updatehauler");

        assert!(!output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            stdout.contains("invalid")
                || stderr.contains("invalid")
                || stdout.contains("error")
                || stderr.contains("error")
        );
    }

    #[test]
    fn test_plugin_subcommand_actions() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        // Test brew subcommand help shows plugin-specific flags
        let output = Command::new(&binary)
            .args(["brew", "--help"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success());
        assert!(stdout.contains("--save-file"));
        assert!(stdout.contains("--sudo"));
        assert!(stdout.contains("[ACTION]"));
    }

    #[test]
    fn test_schedule_subcommand() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["schedule", "--help"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success());
        assert!(stdout.contains("enable"));
        assert!(stdout.contains("disable"));
        assert!(stdout.contains("check"));
    }

    #[test]
    fn test_config_subcommand() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["config", "--help"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success());
        assert!(stdout.contains("init"));
        assert!(stdout.contains("compare"));
        assert!(stdout.contains("merge"));
    }
}
