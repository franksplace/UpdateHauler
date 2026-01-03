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
            // Skip test if binary not built
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
        assert!(stdout.contains("ACTION"));
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

        // Should complete but not do anything for invalid actions
        assert!(output.status.success());
    }

    #[test]
    fn test_run_command() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["--run", "echo", "test"])
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
            .args(["--color", "--run", "echo", "test"])
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
            .args(["--no-color", "--run", "echo", "test"])
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
            .args(["--datetime", "--run", "echo", "test"])
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
            .args(["brew", "help"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success());
        assert!(stdout.contains("Plugin: brew"));
        assert!(stdout.contains("Description:"));
        assert!(stdout.contains("Available Actions:"));
        assert!(stdout.contains("brew"));
        assert!(stdout.contains("brew-save"));
        assert!(stdout.contains("brew-restore"));
        assert!(stdout.contains("Examples:"));
    }

    #[test]
    fn test_plugin_help_cargo() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["cargo", "help"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success());
        assert!(stdout.contains("Plugin: cargo"));
        assert!(stdout.contains("Description:"));
        assert!(stdout.contains("Available Actions:"));
        assert!(stdout.contains("cargo"));
        assert!(stdout.contains("cargo-save"));
        assert!(stdout.contains("cargo-restore"));
        assert!(stdout.contains("Examples:"));
    }

    #[test]
    fn test_plugin_help_nvim() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["nvim", "help"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success());
        assert!(stdout.contains("Plugin: nvim"));
        assert!(stdout.contains("Description:"));
        assert!(stdout.contains("Available Actions:"));
        assert!(stdout.contains("nvim"));
        assert!(stdout.contains("nvim-save"));
        assert!(stdout.contains("nvim-restore"));
        assert!(stdout.contains("Examples:"));
    }

    #[test]
    fn test_plugin_help_os() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["os", "help"])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success());
        assert!(stdout.contains("Plugin: os"));
        assert!(stdout.contains("Description:"));
        assert!(stdout.contains("Available Actions:"));
        assert!(stdout.contains("os"));
        assert!(stdout.contains("Examples:"));
    }

    #[test]
    fn test_plugin_help_invalid() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["invalid-plugin", "help"])
            .output()
            .expect("Failed to execute updatehauler");

        assert!(!output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            stdout.contains("Error: Unknown plugin") || stderr.contains("Error: Unknown plugin")
        );
        assert!(
            stdout.contains("Available plugins: brew, cargo, nvim, os")
                || stderr.contains("Available plugins: brew, cargo, nvim, os")
        );
    }
}
