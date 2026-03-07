#[cfg(test)]
mod tests {
    use crate::config::include_patterns::Pattern;

    #[test]
    fn test_regex_path_only() {
        let input = "/home/user/test";

        let pattern: Pattern = input.parse().unwrap();

        assert_eq!(pattern.path.to_str().unwrap(), "/home/user/test");
        assert_eq!(pattern.max_depth, 1);
    }

    #[test]
    fn test_regex_with_explicit_depth() {
        let input = "/var/log:5";
        let pattern: Pattern = input.parse().unwrap();

        assert_eq!(pattern.path.to_str().unwrap(), "/var/log");
        assert_eq!(pattern.max_depth, 5);
    }

    #[test]
    fn test_regex_edge_cases() {
        let input = "/weird:path:with:colons:10";
        let pattern: Pattern = input.parse().unwrap();

        assert_eq!(pattern.max_depth, 10);
        assert!(pattern.path.to_str().unwrap().contains("colons"));
    }

    #[test]
    fn test_regex_invalid_depth() {
        let input = "/etc:abc";
        let pattern: Pattern = input.parse().unwrap();

        assert_eq!(pattern.max_depth, 1);
    }

    #[test]
    fn test_parse_config() {
        use crate::config::Config;
        use crate::utils::read_config_file;
        use std::fs;

        let test_path = std::env::current_dir().unwrap().join("test_config.toml");

        let default_cfg = Config::default();
        let toml_content = toml::to_string(&default_cfg).expect("Failed to serialize");
        fs::write(&test_path, toml_content).expect("Failed to write temp file");

        let loaded_result = read_config_file(&test_path);

        let _ = fs::remove_file(&test_path);

        assert!(
            loaded_result.is_ok(),
            "The program failed to read its default config from disk!"
        );

        let loaded_cfg = loaded_result.unwrap();
        assert_eq!(loaded_cfg.toggle_hotkey, "ALT+SPACE");
    }
}
