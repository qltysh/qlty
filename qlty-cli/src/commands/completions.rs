use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::{Args, CommandFactory};
use clap_complete::Shell;

#[derive(Args, Debug)]
pub struct Completions {
    /// Shell to generate completions for
    #[arg(long, value_enum)]
    shell: Option<Shell>,

    /// Install completions for the given shell
    #[arg(long)]
    install: bool,
}

impl Completions {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let bin_name = self.bin_name();

        if let Some(mut shell) = crate::shell::build(self.shell) {
            let mut command = Arguments::command();
            let completions = shell.generate(&bin_name, &mut command)?;

            if self.install {
                let path = shell.install(&completions)?;
                shell.update_rc(&path)?;
                eprintln!("Completions installed in {}", path.display());
            } else {
                println!("{}", completions.completions);
            }

            CommandSuccess::ok()
        } else {
            eprintln!("Unsupported shell: {:?}", self.shell);
            Err(CommandError::InvalidOptions {
                message: "Unsupported shell".to_owned(),
            })
        }
    }

    fn bin_name(&self) -> String {
        if let Some(bin_name) = Arguments::command().get_bin_name() {
            bin_name.to_owned()
        } else {
            "qlty".to_owned()
        }
    }
}
