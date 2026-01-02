#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::Command;

    fn get_updatehauler_binary() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("target/release/updatehauler");
        path
    }

    #[test]
    fn test_schedule_custom_hour_minute() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args([
                "--sched-minute",
                "30",
                "--sched-hour",
                "14",
                "schedule",
                "enable",
            ])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(output.status.success());

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

        let output = Command::new(&binary)
            .args(["--sched-day-of-month", "15", "schedule", "enable"])
            .output()
            .expect("Failed to execute updatehauler");

        assert!(output.status.success());
    }

    #[test]
    fn test_schedule_custom_month() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["--sched-month", "12", "schedule", "enable"])
            .output()
            .expect("Failed to execute updatehauler");

        assert!(output.status.success());
    }

    #[test]
    fn test_schedule_custom_day_of_week() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args(["--sched-day-of-week", "MWF", "schedule", "enable"])
            .output()
            .expect("Failed to execute updatehauler");

        assert!(output.status.success());
    }

    #[test]
    fn test_schedule_all_custom_values() {
        let binary = get_updatehauler_binary();

        if !binary.exists() {
            return;
        }

        let output = Command::new(&binary)
            .args([
                "--sched-minute",
                "45",
                "--sched-hour",
                "16",
                "--sched-day-of-month",
                "1",
                "--sched-month",
                "1",
                "--sched-day-of-week",
                "M",
                "schedule",
                "enable",
            ])
            .output()
            .expect("Failed to execute updatehauler");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(output.status.success());

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

        let output = Command::new(&binary)
            .args(["schedule", "enable"])
            .output()
            .expect("Failed to execute updatehauler");

        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(output.status.success());

        if cfg!(target_os = "macos") {
            // Verify that enabling schedule doesn't produce "Boot-out failed" error
            // This error occurs when trying to bootout a non-existent service
            assert!(!stderr.contains("Boot-out failed"));
            assert!(!stderr.contains("No such process"));
        }
    }
}
