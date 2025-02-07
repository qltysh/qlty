use crate::planner::InvocationPlan;
use anyhow::Result;
use itertools::Itertools;
use qlty_analysis::utils::fs::path_to_string;
use qlty_analysis::{join_path_string, utils::fs::path_to_native_string};
use qlty_config::config::InvocationDirectoryType;
use tracing::{error, trace};

#[cfg(unix)]
use shell_escape::unix::escape;
#[cfg(windows)]
use shell_escape::windows::escape;

pub fn compute_invocation_script(plan: &InvocationPlan) -> Result<String> {
    trace!("Driver script (original): {}", plan.driver.script);
    let mut base_script = plan.driver.script.clone();

    // Autoload script first in case it has variables that need to be interpolated
    base_script = replace_autoload_script(plan, base_script);
    base_script = replace_config_script(plan, base_script);
    base_script = plan.tool.interpolate_variables(&base_script);
    base_script = replace_target_variable(plan, base_script);
    base_script = replace_tmpfile_variable(plan, base_script);
    base_script = replace_config_file(plan, base_script);

    trace!("Driver script (interpolated): {}", base_script);
    Ok(base_script)
}

fn replace_autoload_script(plan: &InvocationPlan, script: String) -> String {
    if script.contains("${autoload_script}") {
        if plan.plugin.package_file.is_some() {
            let autoload_script = plan.driver.autoload_script.as_deref().unwrap_or("");
            script.replace("${autoload_script}", autoload_script)
        } else {
            script.replace("${autoload_script}", "")
        }
    } else {
        script
    }
}

fn replace_config_script(plan: &InvocationPlan, script: String) -> String {
    if script.contains("${config_script}") {
        if !plan.plugin_configs.is_empty() {
            let config_script = plan.driver.config_script.as_deref().unwrap_or("");
            script.replace("${config_script}", config_script)
        } else {
            script.replace("${config_script}", "")
        }
    } else {
        script
    }
}

fn replace_target_variable(plan: &InvocationPlan, script: String) -> String {
    if script.contains("${target}") {
        let targets_list = plan_target_list(plan);
        script.replace("${target}", &targets_list)
    } else {
        script
    }
}

fn replace_tmpfile_variable(plan: &InvocationPlan, script: String) -> String {
    if plan.uses_tmpfile() {
        script.replace("${tmpfile}", &plan.tmpfile_path())
    } else {
        script
    }
}

fn replace_config_file(plan: &InvocationPlan, script: String) -> String {
    if !script.contains("${config_file}") {
        return script;
    }

    let config_file_paths = get_config_file_paths(plan);
    let all_configs = config_file_paths.join(":");

    if config_file_paths.len() > 1 {
        error!(
            "{} has more than one config file, but only one is supported, {}",
            plan.invocation_label(),
            all_configs
        );
    }

    script.replace("${config_file}", &all_configs)
}

fn get_config_file_paths(plan: &InvocationPlan) -> Vec<String> {
    plan.plugin_configs
        .iter()
        .map(|config| {
            if plan.driver.copy_configs_into_tool_install {
                let config_file_name = config.path.file_name().unwrap().to_str().unwrap();
                let config_file = path_to_native_string(join_path_string!(
                    plan.tool.directory(),
                    config_file_name
                ));
                escape(config_file.into()).into_owned()
            } else {
                let config_file = plan.workspace.root.join(&config.path);
                escape(path_to_native_string(config_file).into()).into_owned()
            }
        })
        .collect()
}

pub fn plan_target_list(plan: &InvocationPlan) -> String {
    plan.targets
        .iter()
        .map(|target| {
            let target_path = match plan.invocation_directory_def.kind {
                InvocationDirectoryType::Root => target.path_string(),
                InvocationDirectoryType::ToolDir => plan
                    .target_root // resolves to staging_area for formatters
                    .join(&target.path)
                    .to_str()
                    .unwrap()
                    .to_string(),
                _ => {
                    if let Ok(relative_path) =
                        plan.invocation_directory.strip_prefix(&plan.target_root)
                    {
                        path_to_string(
                            target
                                .path
                                .strip_prefix(relative_path)
                                .unwrap_or(&target.path),
                        )
                    } else {
                        target.path_string()
                    }
                }
            };

            escape(path_to_native_string(target_path).into())
        })
        .collect_vec()
        .join(" ")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{executor::driver::test::build_driver, planner::target::Target, tool::ruby::Ruby};
    use qlty_analysis::{WorkspaceEntry, WorkspaceEntryKind};
    use qlty_config::{
        config::{InvocationDirectoryDef, PluginDef},
        Workspace,
    };
    use qlty_types::analysis::v1::ExecutionVerb;
    use std::{path::PathBuf, sync::Arc, time::SystemTime};

    #[test]
    fn test_target_list() {
        let workspace_root = PathBuf::from("/var/root");

        let plan = InvocationPlan {
            target_root: workspace_root.clone(),
            workspace_entries: Arc::new(vec![WorkspaceEntry {
                path: PathBuf::from("nested_dir/basic.in.py"),
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }]),
            invocation_id: "".to_string(),
            verb: ExecutionVerb::Check,
            workspace: Workspace {
                root: workspace_root,
            },
            settings: Default::default(),
            runtime: None,
            runtime_version: None,
            plugin_name: "test".to_string(),
            plugin: PluginDef::default(),
            tool: Ruby::new_tool(""),
            driver_name: "test".to_string(),
            driver: build_driver(vec![], vec![]),
            plugin_configs: vec![],
            targets: vec![Target {
                path: PathBuf::from("nested_dir"),
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::Directory,
            }],
            invocation_directory: PathBuf::from("/var/root/nested_dir"),
            invocation_directory_def: InvocationDirectoryDef {
                kind: InvocationDirectoryType::RootOrParentWith,
                path: Some("nested_dir".to_string()),
            },
        };

        let target_list = plan_target_list(&plan);

        #[cfg(unix)]
        assert_eq!(target_list, "''");

        #[cfg(windows)]
        assert_eq!(target_list, "\"\"");
    }

    #[test]
    fn test_target_list_default_invocation_directory() {
        let invocation_directory = PathBuf::from("/var/root/nested_dir");

        let plan = InvocationPlan {
            target_root: PathBuf::from(invocation_directory.clone()),
            workspace_entries: Arc::new(vec![WorkspaceEntry {
                path: PathBuf::from("nested_dir/basic.in.py"),
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }]),
            invocation_id: "".to_string(),
            verb: ExecutionVerb::Check,
            workspace: Workspace {
                root: PathBuf::from("/var/root"),
            },
            settings: Default::default(),
            runtime: None,
            runtime_version: None,
            plugin_name: "test".to_string(),
            plugin: PluginDef::default(),
            tool: Ruby::new_tool(""),
            driver_name: "test".to_string(),
            driver: build_driver(vec![], vec![]),
            plugin_configs: vec![],
            targets: vec![Target {
                path: PathBuf::from("nested_dir"),
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::Directory,
            }],
            invocation_directory,
            invocation_directory_def: InvocationDirectoryDef::default(),
        };

        let target_list = plan_target_list(&plan);

        assert_eq!(target_list, "nested_dir");
    }

    #[test]
    fn test_target_list_default_invocation_directory_and_default_target() {
        let invocation_directory = PathBuf::from("/var/root/nested_dir");
        let target_path = PathBuf::from("nested_dir/basic.in.py");

        let plan = InvocationPlan {
            target_root: PathBuf::from(invocation_directory.clone()),
            workspace_entries: Arc::new(vec![WorkspaceEntry {
                path: target_path.clone(),
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }]),
            invocation_id: "".to_string(),
            verb: ExecutionVerb::Check,
            workspace: Workspace {
                root: PathBuf::from("/var/root"),
            },
            settings: Default::default(),
            runtime: None,
            runtime_version: None,
            plugin_name: "test".to_string(),
            plugin: PluginDef::default(),
            tool: Ruby::new_tool(""),
            driver_name: "test".to_string(),
            driver: build_driver(vec![], vec![]),
            plugin_configs: vec![],
            targets: vec![Target {
                path: target_path.clone(),
                content_modified: SystemTime::now(),
                language_name: None,
                contents_size: 0,
                kind: WorkspaceEntryKind::File,
            }],
            invocation_directory,
            invocation_directory_def: InvocationDirectoryDef::default(),
        };

        let target_list = plan_target_list(&plan);

        assert_eq!(
            target_list,
            path_to_native_string(target_path.to_str().unwrap())
        );
    }

    #[test]
    fn test_replace_autoload_script() {
        let mut driver = build_driver(vec![], vec![]);
        driver.autoload_script = Some("autoload.php".to_string());

        let mut plan = InvocationPlan {
            target_root: PathBuf::from("/var/root"),
            workspace_entries: Arc::new(vec![]),
            invocation_id: "".to_string(),
            verb: ExecutionVerb::Check,
            workspace: Workspace {
                root: PathBuf::from("/var/root"),
            },
            settings: Default::default(),
            runtime: None,
            runtime_version: None,
            plugin_name: "test".to_string(),
            plugin: PluginDef::default(),
            tool: Ruby::new_tool(""),
            driver_name: "test".to_string(),
            driver,
            plugin_configs: vec![],
            targets: vec![],
            invocation_directory: PathBuf::from("/var/root"),
            invocation_directory_def: InvocationDirectoryDef::default(),
        };

        let script = "autoload_script: ${autoload_script}".to_string();
        let script = replace_autoload_script(&plan, script);

        assert_eq!(script, "autoload_script: ");

        plan.plugin.package_file = Some("composer.json".to_string());
        let script = "autoload_script: ${autoload_script}".to_string();
        let script = replace_autoload_script(&plan, script);

        assert_eq!(script, "autoload_script: autoload.php");
    }
}
