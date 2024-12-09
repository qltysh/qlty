use anyhow::Result;
use qlty_types::analysis::v1::Issue;

pub mod actionlint;
pub mod bandit;
pub mod biome;
pub mod clippy;
pub mod coffeelint;
pub mod eslint;
pub mod golangci_lint;
pub mod hadolint;
pub mod knip;
pub mod markdownlint;
pub mod mypy;
pub mod php_codesniffer;
pub mod phpstan;
pub mod pylint;
pub mod radarlint;
pub mod regex;
pub mod ripgrep;
pub mod rubocop;
pub mod ruff;
pub mod sarif;
pub mod shellcheck;
pub mod sqlfluff;
pub mod stylelint;
pub mod taplo;
pub mod trivy_sarif;
pub mod trufflehog;
pub mod tsc;
pub mod reek;

pub trait Parser {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>>;
}
