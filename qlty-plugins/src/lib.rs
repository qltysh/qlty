use rust_embed::Embed;

#[derive(Embed)]
#[folder = "plugins/"]
#[include = "*.toml"]
#[exclude = ".qlty/qlty.toml"]
#[exclude = "*/fixtures/*"]
#[exclude = "node_modules/*"]
pub struct Plugins;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_definitions() {
        let eslint = Plugins::get("linters/eslint/plugin.toml").unwrap();
        assert!(eslint.data.len() > 0);
    }

    #[test]
    fn ignore_node_modules() {
        let has_node_modules = Plugins::iter().any(|path| path.contains("node_modules"));
        assert!(has_node_modules == false);
    }
}
