use super::config_files::PluginConfigFile;
use super::target::Target;
use crate::executor::Driver;
use crate::tool::Tool;
use crate::Settings;
use console::style;
use qlty_analysis::utils::fs::{path_to_native_string, path_to_string};
use qlty_analysis::WorkspaceEntry;
use qlty_config::config::InvocationDirectoryDef;
use qlty_config::config::OutputDestination;
use qlty_config::config::PluginDef;
use qlty_config::config::Runtime;
use qlty_config::Workspace;
use qlty_types::analysis::v1::ExecutionVerb;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::error;

#[cfg(unix)]
use shell_escape::unix::escape;
#[cfg(windows)]
use shell_escape::windows::escape;

#[derive(Debug, Clone)]
pub struct InvocationPlan {
    pub invocation_id: String,
    pub verb: ExecutionVerb,
    pub settings: Settings,
    pub workspace: Workspace,
    pub runtime: Option<Runtime>,
    pub runtime_version: Option<String>,
    pub plugin_name: String,
    pub plugin: PluginDef,
    pub tool: Box<dyn Tool>,
    pub driver_name: String,
    pub driver: Driver,
    pub plugin_configs: Vec<PluginConfigFile>,
    pub target_root: PathBuf,
    pub workspace_entries: Arc<Vec<WorkspaceEntry>>,
    pub targets: Vec<Target>,
    pub invocation_directory: PathBuf,
    pub invocation_directory_def: InvocationDirectoryDef,
    pub edition: Option<String>,
}

impl InvocationPlan {
    pub fn workspace_entry_paths(&self) -> Vec<PathBuf> {
        self.workspace_entries
            .iter()
            .map(|workspace_entry| workspace_entry.path.to_owned())
            .collect()
    }

    pub fn description(&self) -> String {
        let target_name = self.targets.first().unwrap();
        let targets_count = self.targets.len();

        if targets_count > 1 {
            format!(
                "{}",
                style(format!(
                    "{} and {} more",
                    target_name.path_string(),
                    targets_count - 1
                ))
                .dim()
            )
        } else {
            format!("{}", style(target_name.path_string()).dim())
        }
    }

    pub fn invocation_label(&self) -> String {
        if let Some(prefix) = &self.plugin.prefix {
            format!("{}/{}/{}", self.plugin_name, self.driver_name, prefix)
        } else {
            format!("{}/{}", self.plugin_name, self.driver_name)
        }
    }

    pub fn uses_tmpfile(&self) -> bool {
        self.driver.output == OutputDestination::Tmpfile
    }

    pub fn tmpfile_path(&self) -> String {
        let mut tmpdir = std::env::temp_dir();

        if !tmpdir.exists() && std::fs::create_dir_all(&tmpdir).is_err() {
            error!(
                "Failed to create temporary directory: {}",
                path_to_string(&tmpdir)
            );
        }

        tmpdir.push(self.tmpfile_name());
        escape(path_to_native_string(&tmpdir).into()).into()
    }

    fn tmpfile_name(&self) -> String {
        format!("invocation-out-{}.txt", self.invocation_id)
    }
}
