pub mod command_builder;
mod download;
mod github;
pub mod go;
pub mod java;
pub mod node;
pub mod null_tool;
pub mod php;
pub mod python;
pub mod ruby;
mod ruby_source;
mod runnable_archive;
pub mod rust;
pub mod tool_builder;

use crate::tool::download::Download;
use crate::ui::ProgressBar;
use crate::ui::ProgressTask;
use anyhow::{bail, Context, Result};
use command_builder::Command;
use duct::Expression;
use fslock::LockFile;
use qlty_analysis::join_path_string;
use qlty_analysis::utils::fs::path_to_native_string;
use qlty_analysis::utils::fs::path_to_string;
use qlty_config::config::{PluginDef, PluginEnvironment};
use qlty_config::Library;
use regex::Regex;
use sha2::Digest;
use std::env::join_paths;
use std::env::split_paths;
use std::time::Instant;
use std::{collections::HashMap, fmt::Debug, path::PathBuf};
use tracing::{debug, error, info};

const MAX_TOOL_INSTALL_ATTEMPTS: u32 = 3;

#[cfg(unix)]
const BASE_SHELL_PATH: &[&str] = &["/usr/local/bin", "/usr/bin", "/bin", "/usr/sbin", "/sbin"];
#[cfg(windows)]
const BASE_SHELL_PATH: &[&str] = &[
    r"%SYSTEMROOT%\System32",
    r"%SYSTEMROOT%",
    r"%SYSTEMROOT%\System32\Wbem",
];

#[cfg(unix)]
const SYSTEM_ENV_KEYS: &[&str] = &["HOME"];
#[cfg(windows)]
const SYSTEM_ENV_KEYS: &[&str] = &[
    "SYSTEMROOT",
    "SYSTEMDRIVE",
    "WINDIR",
    "TEMP",
    "TMP",
    "USERPROFILE",
    "COMSPEC",
    "LOCALAPPDATA",
    "APPDATA",
    "CommonProgramFiles",
    "CommonProgramFiles(x86)",
    "CommonProgramW6432",
    "ProgramData",
    "ProgramFiles",
    "ProgramFiles(x86)",
    "ProgramW6432",
    "HOMEDRIVE",
    "HOMEPATH",
];

fn tools_root() -> String {
    path_to_string(
        Library::global_cache_root()
            .expect("Failed to get cache root")
            .join("tools"),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum ToolType {
    Runtime,
    Download,
    RuntimePackage,
    GitHubRelease,
}

pub trait Tool: Debug + Sync + Send {
    fn name(&self) -> String;
    fn version(&self) -> Option<String>;

    fn directory(&self) -> String {
        path_to_string(PathBuf::from(self.parent_directory()).join(self.directory_name()))
    }

    fn directory_name(&self) -> String {
        format!(
            "{}-{}",
            self.version().unwrap_or("generic".to_string()),
            self.fingerprint()
        )
    }

    fn runtime(&self) -> Option<Box<dyn Tool>> {
        None
    }

    fn tool_type(&self) -> ToolType;

    fn parent_directory(&self) -> String {
        path_to_string(PathBuf::from(tools_root()).join(self.name()))
    }

    fn fingerprint(&self) -> String {
        let mut hasher = sha2::Sha256::new();
        self.update_hash(&mut hasher).unwrap();
        let sha_string = format!("{:x}", hasher.finalize());
        sha_string[..12].to_owned()
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        if let Some(runtime) = self.runtime() {
            runtime.update_hash(sha)?;
        }

        if let Some(plugin) = self.plugin() {
            sha.update(plugin.package.as_ref().unwrap());
            sha.update(plugin.version.as_ref().unwrap());

            let mut extra_packages = plugin.extra_packages.clone();
            extra_packages.sort_by(|a, b| a.name.cmp(&b.name));

            for package in extra_packages {
                sha.update(&package.name);
                sha.update(&package.version);
            }

            if let Some(package_file) = plugin.package_file {
                sha.update(std::fs::read_to_string(package_file)?);
            }

            for filter in &plugin.package_filters {
                sha.update(filter);
            }
        }
        Ok(())
    }

    fn pre_setup(&self, task: &ProgressTask) -> Result<()> {
        if let Some(runtime) = self.runtime() {
            runtime.setup(task)?;
        }
        Ok(())
    }

    fn setup(&self, task: &ProgressTask) -> Result<()> {
        std::fs::create_dir_all(self.parent_directory())?;

        task.set_dim_message(&format!("Waiting for lock for {}", self.name()));

        let mut lockfile = LockFile::open(&self.lockfile_path()).with_context(|| {
            format!(
                "Failed to open lockfile: {:?}",
                path_to_string(self.lockfile_path())
            )
        })?;

        lockfile
            .lock_with_pid()
            .with_context(|| format!("Failed to lock file: {:?}", self.lockfile_path()))?;

        debug!(
            "Acquired lock for {} at {}",
            self.name(),
            path_to_string(self.lockfile_path())
        );

        if self.donefile_path().exists() && self.exists() {
            debug!(
                "Tool already installed: {}@{:?}",
                self.name(),
                self.version()
            );
            Ok(())
        } else {
            info!(
                "Setting up tool {}@{:?}. Logging to {}",
                self.name(),
                self.version(),
                self.install_log_path()
            );
            let timer = Instant::now();

            match self.install_and_validate(task) {
                Ok(_) => {
                    std::fs::File::create(self.donefile_path())?;
                    info!(
                        "Set up {}@{:?} in {:.2}s",
                        self.name(),
                        self.version(),
                        timer.elapsed().as_secs_f32()
                    );
                    Ok(())
                }
                Err(e) => {
                    error!(
                        "Failed to set up {}@{:?}: {:?}",
                        self.name(),
                        self.version(),
                        e
                    );

                    Err(e).with_context(|| {
                        format!(
                            "Error installing {}@{}.\n\n    See more: {}",
                            self.name(),
                            self.version().unwrap_or_default(),
                            self.install_log_path()
                        )
                    })
                }
            }
        }
    }

    fn install_and_validate(&self, task: &ProgressTask) -> Result<()> {
        self.internal_pre_install(task)?;
        self.pre_install(task)?;
        self.install_with_retry(task)?;
        self.post_install(task)?;
        self.validate()?;
        Ok(())
    }

    fn donefile_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.done", self.directory()))
    }

    fn lockfile_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.lock", self.directory()))
    }

    fn exists(&self) -> bool {
        PathBuf::from(self.directory()).exists()
    }

    fn internal_pre_install(&self, _task: &ProgressTask) -> Result<()> {
        std::fs::create_dir_all(self.directory()).map_err(|e| e.into())
    }

    fn pre_install(&self, _task: &ProgressTask) -> Result<()> {
        Ok(())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        if let Some(plugin) = self.plugin() {
            self.package_install(task, &plugin.package.unwrap(), &plugin.version.unwrap())?;

            if plugin.package_file.is_some() {
                self.package_file_install(task)?;
            } else {
                for package in &plugin.extra_packages {
                    self.package_install(task, &package.name, &package.version)?;
                }
            }

            Ok(())
        } else {
            bail!("Failed to install {:?}: missing plugin", self.name());
        }
    }

    fn install_with_retry(&self, task: &ProgressTask) -> Result<()> {
        let mut attempts = 0;

        loop {
            match self.install(task) {
                Ok(_) => break,
                Err(e) => {
                    error!("{}: tool installation error: {:?}", self.name(), e);

                    attempts += 1;
                    if attempts >= MAX_TOOL_INSTALL_ATTEMPTS {
                        error!(
                            "Max attempts reached for tool installation: {}",
                            self.name()
                        );
                        return Err(e);
                    }

                    info!(
                        "Attempting retry #{} for tool installation: {}({:?})",
                        attempts,
                        self.name(),
                        self.version()
                    );
                }
            }
        }

        Ok(())
    }

    fn package_install(&self, _task: &ProgressTask, _name: &str, _version: &str) -> Result<()> {
        bail!(
            "Package installation for {} is not implemented",
            self.name()
        );
    }

    fn package_file_install(&self, _task: &ProgressTask) -> Result<()> {
        bail!(
            "Package file installation for {} is not implemented",
            self.name()
        );
    }

    fn post_install(&self, _task: &ProgressTask) -> Result<()> {
        Ok(())
    }

    fn run_command(&self, cmd: Expression) -> Result<()> {
        let cmd = cmd
            .dir(self.directory())
            .full_env(self.env())
            .stderr_to_stdout()
            .stdout_file(self.install_log_file()?);

        debug!("{:?}", cmd);
        cmd.run()?;

        Ok(())
    }

    fn validate(&self) -> Result<()> {
        match self.version_command() {
            Some(_) => {
                let installed_version = self.installed_version()?;

                match self.expected_version()? {
                    Some(ref expected_version) => {
                        if installed_version == *expected_version {
                            info!(
                                "Validated tool: {}: {}",
                                self.version_command().as_ref().unwrap_or(&"".to_string()),
                                expected_version
                            );
                        } else {
                            bail!(
                                "Invalid tool version: {}: {} does not match version {:?} (extracted with regex {:?})",
                                self.version_command().as_ref().unwrap_or(&"".to_string()),
                                installed_version,
                                expected_version,
                                self.version_regex()
                            );
                        }
                    }
                    None => {
                        debug!(
                            "Tool version is {} but nothing to compare to: {:?}",
                            installed_version,
                            self.name()
                        );
                    }
                }
            }
            None => {
                debug!(
                    "Skipping validation, no version command for tool: {:?}",
                    self.name()
                );
            }
        }

        Ok(())
    }

    fn version_command(&self) -> Option<String>;

    fn version_regex(&self) -> String {
        r"v?(\d+\.\d+\.\d+)".to_string()
    }

    fn expected_version(&self) -> Result<Option<String>> {
        let re = Regex::new(&self.version_regex())
            .with_context(|| format!("Invalid regex {:?} for package", self.version_regex()))?;

        if let Some(declared_version) = self.version() {
            if let Some(captures) = re.captures(&declared_version) {
                let captured_version = captures
                    .get(1)
                    .with_context(|| {
                        format!(
                            "No version captured from {:?} using regex {:?}",
                            declared_version,
                            self.version_regex()
                        )
                    })?
                    .as_str();

                Ok(Some(captured_version.to_string()))
            } else {
                bail!(
                    "Package version {:?} does not match regex {:?}",
                    declared_version,
                    self.version_regex()
                );
            }
        } else {
            Ok(None)
        }
    }

    fn installed_version(&self) -> Result<String> {
        if let Some(ref verion_cmd) = self.version_command() {
            let command = Command::new(None, self.interpolate_variables(verion_cmd));
            let env = self.env();
            let cmd_output = command
                .cmd
                .full_env(&env)
                .stdout_capture()
                .stderr_capture()
                .unchecked()
                .run()?;

            if !cmd_output.status.success() {
                error!(
                    "Failed to get version for package {:?}: {:?} {:?}",
                    self.name(),
                    cmd_output,
                    &env,
                );
                bail!(
                    "Failed to get version for package {:?}: (command {} exited with code {})",
                    self.name(),
                    command.script,
                    cmd_output.status.code().unwrap_or_default()
                );
            }

            // ensure stdout appears before stderr in output string
            let output = format!(
                "{} {}",
                String::from_utf8(cmd_output.stdout)?,
                String::from_utf8(cmd_output.stderr)?
            );

            let version_string = output.trim();

            let re = Regex::new(&self.version_regex())
                .with_context(|| format!("Invalid regex {:?} for package", self.version_regex()))?;

            if let Some(captures) = re.captures(version_string) {
                let captured_version = captures
                    .get(1)
                    .with_context(|| {
                        format!(
                            "No version captured from {:?} using regex {:?}",
                            version_string,
                            self.version_regex()
                        )
                    })?
                    .as_str();

                Ok(captured_version.to_string())
            } else {
                bail!(
                    "Package version command {:?} output {:?} does not match regex {:?}",
                    self.version_command(),
                    version_string,
                    self.version_regex()
                );
            }
        } else {
            bail!("No version command for package: {:?}", self.name());
        }
    }

    fn env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        for key in SYSTEM_ENV_KEYS {
            if let Ok(value) = std::env::var(key) {
                env.insert(key.to_string(), value);
            }
        }

        let full_path = path_to_native_string(join_paths(self.env_paths()).unwrap_or_default());

        env.insert("PATH".to_string(), full_path);

        for (key, value) in self.extra_env_vars_with_plugin_env() {
            env.insert(key, value);
        }

        env
    }

    fn extra_env_paths(&self) -> Vec<String> {
        vec![join_path_string!(self.directory(), "bin"), self.directory()]
    }

    fn extra_env_vars(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn install_log_file(&self) -> Result<std::fs::File> {
        let log_path = self.install_log_path();
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        Ok(log_file)
    }

    fn install_log_path(&self) -> String {
        join_path_string!(
            self.parent_directory(),
            format!("{}-install.log", self.directory_name())
        )
    }

    fn clone_box(&self) -> Box<dyn Tool>;

    fn load_environment_paths(&self, plugin_environment: &Vec<PluginEnvironment>) -> Vec<String> {
        let mut paths = vec![];

        for env in plugin_environment {
            if env.name == "PATH" {
                for path in &env.list {
                    paths.extend(
                        split_paths(&self.interpolate_variables(path))
                            .map(path_to_native_string)
                            .collect::<Vec<_>>(),
                    );
                }

                if !env.value.is_empty() {
                    paths.extend(
                        split_paths(&self.interpolate_variables(&env.value))
                            .map(path_to_native_string)
                            .collect::<Vec<_>>(),
                    );
                }
            }
        }

        paths
    }

    fn load_environment_vars(
        &self,
        plugin_environment: &Vec<PluginEnvironment>,
    ) -> HashMap<String, String> {
        let mut env = HashMap::new();

        for plugin_env in plugin_environment {
            if plugin_env.name != "PATH" {
                let value = self.interpolate_variables(&plugin_env.value);
                let value = value.trim();
                if !value.is_empty() {
                    env.insert(plugin_env.name.clone(), value.to_string());
                }
            }
        }

        env
    }

    fn plugin(&self) -> Option<PluginDef> {
        None
    }

    fn env_paths(&self) -> Vec<String> {
        if let Some(plugin) = self.plugin() {
            let plugin_env_paths = self.load_environment_paths(&plugin.environment);
            if !plugin_env_paths.is_empty() {
                return plugin_env_paths;
            }
        }

        let mut paths = self.extra_env_paths();
        if let Some(runtime) = self.runtime() {
            paths.extend(runtime.extra_env_paths());
        }

        if cfg!(windows) {
            paths.extend(BASE_SHELL_PATH.iter().map(|s| {
                s.to_string().replace(
                    "%SYSTEMROOT%",
                    &std::env::var("SYSTEMROOT").unwrap_or_default(),
                )
            }));
        } else {
            paths.extend(BASE_SHELL_PATH.iter().map(|s| s.to_string()));
        }
        paths.iter().map(path_to_native_string).collect()
    }

    fn extra_env_vars_with_plugin_env(&self) -> HashMap<String, String> {
        let mut env = self.extra_env_vars();
        if let Some(plugin) = self.plugin() {
            env.extend(self.load_environment_vars(&plugin.environment));
        }

        env
    }

    fn interpolate_variables(&self, value: &str) -> String {
        let mut result = Regex::new(r"\$\{env\.(.+?)\}")
            .unwrap()
            .replace_all(value, |caps: &regex::Captures| {
                let key = caps.get(1).unwrap().as_str();
                std::env::var(key).unwrap_or_default()
            })
            .replace("${linter}", &path_to_native_string(self.directory()))
            .replace(
                "${cachedir}",
                &path_to_native_string(join_path_string!(
                    std::env::current_dir().unwrap(),
                    ".qlty",
                    "results"
                )),
            );
        if let Some(runtime) = self.runtime() {
            result = result.replace("${runtime}", &path_to_native_string(runtime.directory()));
        }

        result
    }
}

impl Clone for Box<dyn Tool> {
    fn clone(&self) -> Box<dyn Tool> {
        self.clone_box()
    }
}

pub trait RuntimeTool: Tool {
    fn package_tool(&self, name: &str, plugin: &PluginDef) -> Box<dyn Tool>;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Progress;
    use command_builder::test::ENV_LOCK;
    use qlty_analysis::utils::fs::path_to_string;
    use std::env::var;
    use tempfile::tempdir;
    use tracing_test::traced_test;

    pub fn expanded_base_shell_path() -> impl Iterator<Item = std::string::String> {
        BASE_SHELL_PATH.iter().map(|s| {
            s.to_string().replace(
                "%SYSTEMROOT%",
                &std::env::var("SYSTEMROOT").unwrap_or_default(),
            )
        })
    }

    #[derive(Debug, Clone)]
    struct TestTool {
        name: String,
        version: Option<String>,
        tool_type: ToolType,
        version_command: Option<String>,
        runtime: Option<Box<dyn Tool>>,
        plugin: Option<PluginDef>,
        extra_env_vars: HashMap<String, String>,
    }

    impl Default for TestTool {
        fn default() -> Self {
            TestTool {
                name: "test_tool".into(),
                version: Some("1.0.0".into()),
                tool_type: ToolType::Runtime,
                version_command: Some("echo 1.0.0".into()),
                runtime: None,
                plugin: None,
                extra_env_vars: HashMap::new(),
            }
        }
    }

    impl Tool for TestTool {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn version(&self) -> Option<String> {
            self.version.clone()
        }

        fn tool_type(&self) -> ToolType {
            self.tool_type
        }

        fn version_command(&self) -> Option<String> {
            self.version_command.clone()
        }

        fn clone_box(&self) -> Box<dyn Tool> {
            Box::new(self.clone())
        }

        fn runtime(&self) -> Option<Box<dyn Tool>> {
            self.runtime.clone()
        }

        fn plugin(&self) -> Option<PluginDef> {
            self.plugin.clone()
        }

        fn extra_env_vars(&self) -> HashMap<String, String> {
            self.extra_env_vars.clone()
        }

        fn install(&self, _task: &ProgressTask) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    struct TestToolInstallErrors {
        name: String,
        version: Option<String>,
        tool_type: ToolType,
        version_command: Option<String>,
        runtime: Option<Box<dyn Tool>>,
        plugin: Option<PluginDef>,
        extra_env_vars: HashMap<String, String>,
    }

    impl Default for TestToolInstallErrors {
        fn default() -> Self {
            TestToolInstallErrors {
                name: "test_tool_install_errors".into(),
                version: Some("1.0.0".into()),
                tool_type: ToolType::Runtime,
                version_command: Some("echo 1.0.0".into()),
                runtime: None,
                plugin: None,
                extra_env_vars: HashMap::new(),
            }
        }
    }

    impl Tool for TestToolInstallErrors {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn version(&self) -> Option<String> {
            self.version.clone()
        }

        fn tool_type(&self) -> ToolType {
            self.tool_type
        }

        fn version_command(&self) -> Option<String> {
            self.version_command.clone()
        }

        fn clone_box(&self) -> Box<dyn Tool> {
            Box::new(self.clone())
        }

        fn runtime(&self) -> Option<Box<dyn Tool>> {
            self.runtime.clone()
        }

        fn plugin(&self) -> Option<PluginDef> {
            self.plugin.clone()
        }

        fn extra_env_vars(&self) -> HashMap<String, String> {
            self.extra_env_vars.clone()
        }

        fn install(&self, _task: &ProgressTask) -> Result<()> {
            bail!("Error during install");
        }
    }

    #[test]
    fn test_tool_name() {
        let tool = TestTool::default();
        assert_eq!(tool.name(), "test_tool");
    }

    #[test]
    fn test_tool_version() {
        let tool = TestTool::default();
        assert_eq!(tool.version(), Some("1.0.0".to_string()));
    }

    #[test]
    fn test_tool_version_command() {
        let tool = TestTool::default();
        assert_eq!(tool.version_command(), Some("echo 1.0.0".to_string()));
    }

    #[test]
    fn test_tool_tool_type() {
        let tool = TestTool::default();
        assert_eq!(tool.tool_type(), ToolType::Runtime);
    }

    #[test]
    #[traced_test]
    fn test_tool_install_with_retry_ok() {
        let tool = TestTool::default();
        let task = Progress::new(false, 1).task("PREFIX", "message");

        assert!(tool.install_with_retry(&task).is_ok());
        assert!(!logs_contain(
            "Attempting retry #1 for tool installation: test_tool(Some(\"1.0.0\"))"
        ));
    }

    #[test]
    #[traced_test]
    fn test_tool_install_with_retry_errors() {
        let tool = TestToolInstallErrors::default();
        let task = Progress::new(false, 1).task("PREFIX", "message");

        assert!(tool.install_with_retry(&task).is_err());

        assert!(logs_contain(
            "test_tool_install_errors: tool installation error: Error during install"
        ));

        assert!(logs_contain(
            "Attempting retry #1 for tool installation: test_tool_install_errors(Some(\"1.0.0\"))"
        ));

        assert!(logs_contain(
            "Attempting retry #2 for tool installation: test_tool_install_errors(Some(\"1.0.0\"))"
        ));

        assert!(!logs_contain(
            "Attempting retry #3 for tool installation: test_tool_install_errors(Some(\"1.0.0\"))"
        ));
    }

    #[test]
    fn test_tool_fingerprint() {
        let tempdir = tempdir().unwrap();
        std::fs::write(tempdir.path().join("test"), "[package_file]").unwrap();
        let tool = TestTool {
            runtime: Some(Box::new(TestTool {
                name: "[runtime]".into(),
                version: Some("V".into()),
                plugin: Some(PluginDef {
                    package: Some("[runtime_package]".into()),
                    version: Some("V".into()),
                    ..Default::default()
                }),
                ..Default::default()
            })),
            plugin: Some(PluginDef {
                package: Some("[package]".into()),
                version: Some("V".into()),
                extra_packages: vec![
                    qlty_config::config::ExtraPackage {
                        name: "[extra_package1]".into(),
                        version: "V".into(),
                    },
                    qlty_config::config::ExtraPackage {
                        name: "[extra_package2]".into(),
                        version: "V".into(),
                    },
                ],
                package_file: Some(path_to_string(tempdir.path().join("test"))),
                ..Default::default()
            }),
            ..Default::default()
        };

        let hash = "[runtime_package]V[package]V[extra_package1]V[extra_package2]V[package_file]";
        let mut hasher = sha2::Sha256::new();
        hasher.update(hash);
        assert_eq!(tool.fingerprint(), format!("{:x}", hasher.finalize())[..12]);
        drop(tempdir);
    }

    #[test]
    fn test_tool_env() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| {
            ENV_LOCK.clear_poison();
            err.into_inner()
        });
        let tool = TestTool {
            extra_env_vars: [("TEST".into(), "test".into())].iter().cloned().collect(),
            ..TestTool::default()
        };
        let env = tool.env();

        for key in SYSTEM_ENV_KEYS {
            assert_eq!(env.get(*key), Some(&std::env::var(key).unwrap()));
        }

        let mut paths = vec![join_path_string!(tool.directory(), "bin"), tool.directory()];
        paths.extend(expanded_base_shell_path());

        assert_eq!(
            env.get("PATH"),
            Some(&path_to_native_string(
                join_paths(paths).unwrap_or_default()
            ))
        );
        assert_eq!(env.get("TEST"), Some(&"test".to_string()));
    }

    #[test]
    fn test_tool_interpolate_variables() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| {
            ENV_LOCK.clear_poison();
            err.into_inner()
        });
        let tool = TestTool {
            runtime: Some(Box::new(TestTool {
                name: "runtime".into(),
                version: Some("V".into()),
                ..Default::default()
            })),
            ..Default::default()
        };
        assert_eq!(
            tool.interpolate_variables(
                "${linter} ${runtime} ${cachedir} ${env.PATH} ${env.UNKNOWN_VARIABLE}".into()
            ),
            format!(
                "{} {} {} {} ",
                path_to_native_string(tool.directory()),
                path_to_native_string(tool.runtime().unwrap().directory()),
                path_to_native_string(path_to_string(join_path_string!(
                    std::env::current_dir().unwrap(),
                    ".qlty",
                    "results"
                ))),
                var("PATH").unwrap()
            )
        );
    }

    #[test]
    fn test_load_environment_vars() {
        let tool = TestTool::default();
        let plugin_env = vec![
            PluginEnvironment {
                name: "TEST".into(),
                value: "value".into(),
                ..Default::default()
            },
            PluginEnvironment {
                name: "TEST2".into(),
                value: "".into(),
                ..Default::default()
            },
            PluginEnvironment {
                name: "TEST3".into(),
                value: "  ".into(),
                ..Default::default()
            },
        ];

        let env = tool.load_environment_vars(&plugin_env);
        assert_eq!(env.get("TEST"), Some(&"value".to_string()));
        assert_eq!(env.get("TEST2"), None);
        assert_eq!(env.get("TEST3"), None);
    }

    #[test]
    fn test_tool_validate() {
        let tool = TestTool::default();
        assert!(tool.validate().is_ok());
    }

    #[test]
    fn test_tool_validate_incorrect_version() {
        let tool = TestTool {
            version_command: Some("2.0.0".into()),
            ..TestTool::default()
        };
        assert!(tool.validate().is_err());
    }

    #[test]
    fn test_tool_validate_no_version() {
        let tool = TestTool {
            version_command: None,
            ..TestTool::default()
        };
        assert!(tool.validate().is_ok());
    }
}
