use crate::{errors::CommandError, success::CommandSuccess, Arguments, QltyRelease};
use anyhow::Result;
use clap::Args;
use qlty_config::version::LONG_VERSION;

#[derive(Args, Debug)]
pub struct Version {}

impl Version {
    pub fn execute(&self, args: &Arguments) -> Result<CommandSuccess, CommandError> {
        self.print_version();

        if !args.no_upgrade_check {
            QltyRelease::upgrade_check()?;
        }

        CommandSuccess::ok()
    }

    fn print_version(&self) {
        let current_exe = std::env::args()
            .nth(0)
            .expect("Unable to identify current executable");
        let binary_name = std::path::Path::new(&current_exe)
            .file_name()
            .expect("Unable to identify current executable")
            .to_str()
            .unwrap_or("unknown");

        println!("{binary_name} {}", LONG_VERSION.as_str());
    }
}
