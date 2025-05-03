use crate::{
    tool::{
        command_builder::Command, download::Download, finalize_installation_from_cmd_result,
        installations::initialize_installation, ruby::PlatformRuby,
    },
    ui::{ProgressBar, ProgressTask},
    Tool,
};
use anyhow::{bail, Context, Result};
use itertools::Itertools;
use qlty_analysis::utils::fs::path_to_native_string;
use sha2::Digest;
use std::{collections::HashMap, env::split_paths};
use tracing::debug;

#[derive(Debug, Clone, Default)]
pub struct RubyWindows {}

impl PlatformRuby for RubyWindows {
    fn version(&self, _version: &String) -> Option<String> {
        None
    }

    fn update_hash(
        &self,
        tool: &dyn Tool,
        sha: &mut sha2::Sha256,
        _download: Download,
    ) -> Result<()> {
        sha.update(tool.name().as_bytes());
        Ok(())
    }

    fn install(&self, tool: &dyn Tool, task: &ProgressTask, _download: Download) -> Result<()> {
        task.set_message("Using system Ruby");
        Self::verify_system_installation(tool)
    }

    fn post_install(&self, _tool: &dyn Tool, _task: &ProgressTask) -> Result<()> {
        Ok(())
    }

    fn extra_env_paths(&self, _tool: &dyn Tool) -> Vec<String> {
        split_paths(
            &std::env::var("PATH")
                .with_context(|| "PATH not found for Ruby runtime")
                .unwrap(),
        )
        .map(path_to_native_string)
        .collect_vec()
    }

    fn extra_env_vars(&self, _tool: &dyn Tool, _env: &mut HashMap<String, String>) -> Result<()> {
        // Windows does not need any extra env vars
        Ok(())
    }

    fn platform_directory(&self, _tool: &dyn Tool) -> String {
        "unused".to_string()
    }
}

impl RubyWindows {
    fn verify_system_installation(tool: &dyn Tool) -> Result<()> {
        let cmd = Command::new(None, tool.version_command().unwrap_or_default())
            .cmd
            .full_env(tool.env()?)
            .unchecked()
            .stderr_to_stdout()
            .stdout_capture();

        let script = format!("{:?}", cmd);
        debug!("Verify system Ruby: {:?}", script);

        let mut installation = initialize_installation(tool)?;
        let result = cmd.run();
        let _ = finalize_installation_from_cmd_result(tool, &result, &mut installation, script);

        let output = result?;
        if !output.status.success() {
            bail!("Ensure `ruby` is installed and in $PATH");
        }

        debug!("Verified system ruby: {:?}", output);

        Ok(())
    }
}
