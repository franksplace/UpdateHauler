#[cfg(test)]
mod tests {
    use update_hauler::utils;

    #[test]
    fn test_run_cmd_echo() {
        let result = utils::run_cmd("echo", &["hello"]);

        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello"));
    }

    #[test]
    fn test_run_cmd_exit_only_success() {
        let result = utils::run_cmd_exit_only("echo", &["test"]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_run_cmd_exit_only_failure() {
        let result = utils::run_cmd_exit_only("false", &[]);

        assert!(result.is_err());
    }
}
