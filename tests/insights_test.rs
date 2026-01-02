#[cfg(test)]
mod tests {
    use update_hauler::insights::Insights;

    #[test]
    fn test_insights_creation() {
        let insights = Insights::new().expect("Failed to create Insights");

        assert_eq!(insights.os, std::env::consts::OS);
        assert!(insights.os == "linux" || insights.os == "macos");
    }

    #[test]
    fn test_brew_detection() {
        let insights = Insights::new().expect("Failed to create Insights");

        // Test that has_brew is a boolean (may be true or false depending on system)
        let _has_brew = insights.has_brew;
    }

    #[test]
    fn test_cargo_detection() {
        let insights = Insights::new().expect("Failed to create Insights");

        // Test that has_cargo is a boolean (may be true or false depending on system)
        let _has_cargo = insights.has_cargo;
    }

    #[test]
    fn test_os_flags() {
        let insights = Insights::new().expect("Failed to create Insights");

        // On any system, exactly one of these should be true
        assert!(insights.is_linux || insights.is_darwin);
        assert!(!(insights.is_linux && insights.is_darwin));
    }

    #[test]
    fn test_app_abspath() {
        let insights = Insights::new().expect("Failed to create Insights");

        assert!(insights.app_abspath.exists());
    }

    #[test]
    fn test_arch_not_empty() {
        let insights = Insights::new().expect("Failed to create Insights");

        assert!(!insights.arch.is_empty());
    }
}
