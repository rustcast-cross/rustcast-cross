#[cfg(test)]
mod tests {
    use crate::app::apps::AppCommand;
    use crate::app::pages::prelude::SimpleApp;
    use fuzzy_matcher::FuzzyMatcher;
    use fuzzy_matcher::skim::SkimMatcherV2;

    #[test]
    fn test_fuzzy_search_ranking() {
        let app1 = SimpleApp::new_builtin("Firefox", "firefox", "browser", AppCommand::Display);
        let app2 = SimpleApp::new_builtin("File Manager", "files", "explorer", AppCommand::Display);

        let matcher = SkimMatcherV2::default();
        let query = "ffox";

        let score_firefox = matcher.fuzzy_match("firefox", query).unwrap_or(0);
        let score_fileman = matcher.fuzzy_match("files", query).unwrap_or(0);

        assert!(score_firefox > score_fileman);
    }
}
