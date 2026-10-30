//! Run as an unprivileged user on a provisioned Ubuntu 24.04 runner with network access:
//! QLTY_FEATURE_PHP_APT_INSTALL=true cargo run -p qlty-check --example php_apt_smoke
//!
//! This installs PHP, Composer, and PHP_CodeSniffer into the user cache. Keep it
//! separate from unit tests, which CI runs with --include-ignored on every OS.
use anyhow::{ensure, Result};
use duct::cmd;
use qlty_check::{
    tool::{php::Php, RuntimeTool},
    Progress, Tool,
};
use qlty_config::config::PluginDef;

fn main() -> Result<()> {
    let runtime = Php::new("8.4")?;
    ensure!(
        runtime.version_command().is_some(),
        "Set QLTY_FEATURE_PHP_APT_INSTALL=true to exercise managed PHP"
    );
    let system_php = cmd("/usr/bin/php", ["--version"]).read()?;
    let package_database = std::fs::read("/var/lib/dpkg/status")?;
    let package = runtime.package_tool(
        "php-codesniffer",
        &PluginDef {
            package: Some("squizlabs/php_codesniffer".into()),
            version: Some("3.13.6".into()),
            version_command: Some("php ${linter}/vendor/bin/phpcs --version".into()),
            version_regex: r"(\d+\.\d+\.\d+)".into(),
            ..Default::default()
        },
    );
    let task = Progress::new(true, 1).task("PHP", "Rootless installation smoke check");
    package.pre_setup(&task)?;
    package.setup(&task)?;
    ensure!(package.installed_version()? == "3.13.6");
    ensure!(runtime.installed_version()?.starts_with("8.4."));
    ensure!(cmd("/usr/bin/php", ["--version"]).read()? == system_php);
    ensure!(std::fs::read("/var/lib/dpkg/status")? == package_database);
    Ok(())
}
