#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::Command;

    fn get_updatehauler_binary() -> PathBuf {
        PathBuf::from(env!("CARGO_BIN_EXE_updatehauler"))
    }

    fn is_ci() -> bool {
        std::env::var("CI").is_ok()
    }

    #[test]
    fn test_schedule_custom_hour_minute() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let args = if cfg!(target_os = "linux") && is_ci() {
            vec![
                "--dry-run",
                "schedule",
                "enable",
                "--minute",
                "30",
                "--hour",
                "14",
            ]
        } else {
            vec!["schedule", "enable", "--minute", "30", "--hour", "14"]
        };

        let output = Command::new(&binary)
            .args(&args)
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "Command failed: {:?}\nstdout: {}\nstderr: {}",
            args,
            stdout,
            stderr
        );

        if cfg!(target_os = "macos") {
            assert!(stdout.contains("pmset") || stderr.contains("pmset"));
            assert!(stdout.contains("14:30:00") || stderr.contains("14:30:00"));
        }
    }

    #[test]
    fn test_schedule_custom_day_of_month() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let args = if cfg!(target_os = "linux") && is_ci() {
            vec!["--dry-run", "schedule", "enable", "--day-of-month", "15"]
        } else {
            vec!["schedule", "enable", "--day-of-month", "15"]
        };

        let output = Command::new(&binary)
            .args(&args)
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "Command failed: {:?}\nstdout: {}\nstderr: {}",
            args,
            stdout,
            stderr
        );
    }

    #[test]
    fn test_schedule_custom_month() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let args = if cfg!(target_os = "linux") && is_ci() {
            vec!["--dry-run", "schedule", "enable", "--month", "12"]
        } else {
            vec!["schedule", "enable", "--month", "12"]
        };

        let output = Command::new(&binary)
            .args(&args)
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "Command failed: {:?}\nstdout: {}\nstderr: {}",
            args,
            stdout,
            stderr
        );
    }

    #[test]
    fn test_schedule_custom_day_of_week() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let args = if cfg!(target_os = "linux") && is_ci() {
            vec!["--dry-run", "schedule", "enable", "--day-of-week", "MWF"]
        } else {
            vec!["schedule", "enable", "--day-of-week", "MWF"]
        };

        let output = Command::new(&binary)
            .args(&args)
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "Command failed: {:?}\nstdout: {}\nstderr: {}",
            args,
            stdout,
            stderr
        );
    }

    #[test]
    fn test_schedule_all_custom_values() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let args = if cfg!(target_os = "linux") && is_ci() {
            vec![
                "--dry-run",
                "schedule",
                "enable",
                "--minute",
                "45",
                "--hour",
                "16",
                "--day-of-month",
                "1",
                "--month",
                "1",
                "--day-of-week",
                "M",
            ]
        } else {
            vec![
                "schedule",
                "enable",
                "--minute",
                "45",
                "--hour",
                "16",
                "--day-of-month",
                "1",
                "--month",
                "1",
                "--day-of-week",
                "M",
            ]
        };

        let output = Command::new(&binary)
            .args(&args)
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "Command failed: {:?}\nstdout: {}\nstderr: {}",
            args,
            stdout,
            stderr
        );

        if cfg!(target_os = "macos") {
            assert!(stdout.contains("16:45:00") || stderr.contains("16:45:00"));
        }
    }

    #[test]
    fn test_schedule_enable_no_bootout_error() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let args = if cfg!(target_os = "linux") && is_ci() {
            vec!["--dry-run", "schedule", "enable"]
        } else {
            vec!["schedule", "enable"]
        };

        let output = Command::new(&binary)
            .args(&args)
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "Command failed: {:?}\nstdout: {}\nstderr: {}",
            args,
            stdout,
            stderr
        );

        if cfg!(target_os = "macos") {
            assert!(!stderr.contains("Boot-out failed"));
            assert!(!stderr.contains("No such process"));
        }
    }
}
