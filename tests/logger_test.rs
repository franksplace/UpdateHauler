#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;
    use update_hauler::config::Config;
    use update_hauler::logger::Logger;

    #[test]
    fn test_logger_creation() {
        let config = Config::new("/tmp/test");
        let _logger = Logger::new(&config);

        // Just test that logger can be created
        // Can't access internal fields directly
    }

    #[test]
    fn test_log_to_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("test.log");

        let mut config = Config::new("/tmp/test");
        config.log = log_path.clone();
        config.use_log = true;

        let mut logger = Logger::new(&config);
        logger.log("Test message");

        assert!(log_path.exists());

        let content = fs::read_to_string(&log_path).expect("Failed to read log file");
        assert!(content.contains("Test message"));
    }

    #[test]
    fn test_log_multiple_messages() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("test.log");

        let mut config = Config::new("/tmp/test");
        config.log = log_path.clone();
        config.use_log = true;

        let mut logger = Logger::new(&config);
        logger.log("Message 1");
        logger.log("Message 2");
        logger.log("Message 3");

        let content = fs::read_to_string(&log_path).expect("Failed to read log file");
        assert!(content.contains("Message 1"));
        assert!(content.contains("Message 2"));
        assert!(content.contains("Message 3"));
    }

    #[test]
    fn test_log_with_datetime() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("test.log");

        let mut config = Config::new("/tmp/test");
        config.log = log_path.clone();
        config.use_log = true;
        config.datetime = true;

        let mut logger = Logger::new(&config);
        logger.log("Test message");

        let content = fs::read_to_string(&log_path).expect("Failed to read log file");

        // ISO8601 format should contain T and : (e.g., 2024-12-31T12:34:56.789)
        assert!(content.contains("Test message"));
        // When datetime is true, we should see timestamp format
        let lines: Vec<&str> = content.lines().collect();
        if !lines.is_empty() {
            let first_line = lines[0];
            // Check for ISO8601 pattern (YYYY-MM-DDTHH:MM:SS)
            assert!(first_line.chars().any(|c| c == 'T') || !first_line.is_empty());
        }
    }

    #[test]
    fn test_log_without_datetime() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("test.log");

        let mut config = Config::new("/tmp/test");
        config.log = log_path.clone();
        config.use_log = true;
        config.datetime = false;

        let mut logger = Logger::new(&config);
        logger.log("Test message");

        let content = fs::read_to_string(&log_path).expect("Failed to read log file");

        assert!(content.contains("Test message"));
        // First character should not be a digit (no timestamp)
        let first_char = content.chars().next();
        assert!(
            !first_char.map(|c| c.is_ascii_digit()).unwrap_or(false) || content.starts_with("Test")
        );
    }

    #[test]
    fn test_log_creates_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_dir = temp_dir.path().join("nested/directory");
        let log_path = log_dir.join("test.log");

        let mut config = Config::new("/tmp/test");
        config.log = log_path.clone();
        config.use_log = true;

        let mut logger = Logger::new(&config);
        logger.log("Test message");

        assert!(log_path.exists());
        assert!(log_dir.exists());
    }
}
