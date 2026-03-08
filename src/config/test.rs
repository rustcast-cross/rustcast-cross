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

        let original_config = Config::default();

        let toml_string =
            toml::to_string(&original_config).expect("Should be able to turn config into a string");

        let recovered_config: Config = toml::from_str(&toml_string)
            .expect("Should be able to read that string back into a Config");

        assert_eq!(original_config, recovered_config);
    }
}
