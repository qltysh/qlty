//! Run as the runner user on an image with baked PHP runtimes and network access:
//! QLTY_PHP_RUNTIMES_DIR=/opt/qlty/php cargo run -p qlty-check --example php_runner_smoke
//!
//! Composer and PHP_CodeSniffer are downloaded; PHP itself is already installed.
use anyhow::{ensure, Result};
use duct::cmd;
use qlty_check::{
    tool::{php::Php, RuntimeTool},
    Progress, Tool,
};
use qlty_config::config::PluginDef;

fn main() -> Result<()> {
    ensure!(std::env::var_os("QLTY_PHP_RUNTIMES_DIR").is_some());
    let default_php = cmd("/usr/bin/php", ["--version"]).read()?;
    let package_database = std::fs::read("/var/lib/dpkg/status")?;
    let task = Progress::new(true, 1).task("PHP", "Baked runtime smoke check");

    for minor in ["8.1", "8.2", "8.3", "8.4"] {
        let runtime = Php::new(minor);
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
        package.pre_setup(&task)?;
        package.setup(&task)?;
        ensure!(package.installed_version()? == "3.13.6");
        let patch = runtime.installed_version()?;
        ensure!(patch.starts_with(&format!("{minor}.")));
        Php::new(&patch).setup(&task)?;
        println!("Verified PHP {minor} ({patch}), Composer, and PHP_CodeSniffer");
    }

    let error = Php::new("8.0").setup(&task).unwrap_err();
    ensure!(format!("{error:#}").contains("not baked into this runner image"));
    ensure!(cmd("/usr/bin/php", ["--version"]).read()? == default_php);
    ensure!(std::fs::read("/var/lib/dpkg/status")? == package_database);
    Ok(())
}
