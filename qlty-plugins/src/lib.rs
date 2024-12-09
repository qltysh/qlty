use rust_embed::Embed;

#[derive(Embed)]
#[folder = "plugins/"]
#[include = "*.toml"]
#[exclude = ".qlty/qlty.toml"]
#[exclude = "*/fixtures/*"]
pub struct Plugins;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_definitions() {
        let eslint = Plugins::get("linters/eslint/plugin.toml").unwrap();
        assert!(eslint.data.len() > 0);
    }
}
