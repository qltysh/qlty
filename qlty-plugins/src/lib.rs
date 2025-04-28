use rust_embed::Embed;

#[derive(Embed)]
#[folder = "plugins/"]
#[exclude = "**/fixtures/**"]
#[exclude = "**/node_modules/**"]
#[exclude = "**/README.md"]
#[exclude = "**/*.test.ts"]

pub struct Plugins;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_definitions() {
        let shellcheck = Plugins::get("linters/shellcheck/plugin.toml").unwrap();
        assert!(shellcheck.data.len() > 0);
    }

    #[test]
    fn get_plugin_config() {
        let shellcheck_config = Plugins::get("linters/shellcheck/.shellcheckrc").unwrap();
        assert!(shellcheck_config.data.len() > 0);
    }

    #[test]
    fn ignore_node_modules() {
        let has_node_modules = Plugins::iter().any(|path| path.contains("node_modules"));
        assert!(has_node_modules == false);
    }

    #[test]
    fn ignore_fixtures() {
        let has_node_modules = Plugins::iter().any(|path| path.contains("fixtures"));
        assert!(has_node_modules == false);
    }

    #[test]
    fn ignore_readme() {
        let has_node_modules = Plugins::iter().any(|path| path.contains("README.md"));
        assert!(has_node_modules == false);
    }

    #[test]
    fn ignore_test_files() {
        let has_node_modules = Plugins::iter().any(|path| path.contains(".test.ts"));
        assert!(has_node_modules == false);
    }
}
