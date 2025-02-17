pub mod gemfile;
mod sys;

use super::command_builder::{default_command_builder, CommandBuilder};
use super::ruby_source::RubySource;
use super::{Tool, ToolType};
use crate::tool::download::Download;
use crate::tool::RuntimeTool;
use crate::ui::{ProgressBar, ProgressTask};
use anyhow::Result;
use itertools::Itertools;
use qlty_analysis::join_path_string;
use qlty_analysis::utils::fs::{path_to_native_string, path_to_string};
use qlty_config::config::{Cpu, DownloadDef, System};
use qlty_config::config::{OperatingSystem, PluginDef};
use std::collections::HashMap;
use std::env::join_paths;
use std::fmt::Debug;
use std::fs::read_dir;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct Ruby {
    pub version: String,
    platform_tool: sys::platform::Ruby,
}

pub trait PlatformRuby {
    fn post_install(&self, tool: &dyn Tool, task: &ProgressTask) -> Result<()>;
    fn extra_env_paths(&self, tool: &dyn Tool) -> Vec<String>;
    fn extra_env_vars(&self, tool: &dyn Tool, env: &mut HashMap<String, String>);
    fn platform_directory(&self, tool: &dyn Tool) -> String;

    fn version(&self, version: &String) -> Option<String> {
        Some(version.to_string())
    }

    fn install(&self, tool: &dyn Tool, task: &ProgressTask, download: Download) -> Result<()> {
        task.set_message("Installing Ruby");
        download.install(tool.directory(), tool.name())
    }

    fn insert_rubylib_env(&self, tool: &dyn Tool, env: &mut HashMap<String, String>) {
        let major_version = self.major_version(tool);
        let platform_directory = self.platform_directory(tool);
        let lib_prefix = join_path_string!(tool.directory(), "lib", "ruby");
        env.insert(
            "RUBYLIB".to_string(),
            join_paths(
                ["site_ruby", "vendor_ruby", ""]
                    .iter()
                    .flat_map(|dir| {
                        let mut major_version = major_version.clone();
                        let entries_path = join_path_string!(&lib_prefix, dir);
                        if let Ok(entries) = read_dir(entries_path) {
                            for entry in entries.flatten().filter(|entry| entry.path().is_dir()) {
                                if path_to_string(entry.file_name()).starts_with(&major_version) {
                                    major_version = path_to_string(entry.file_name());
                                    break;
                                }
                            }
                        }

                        [
                            join_path_string!(dir, &major_version),
                            join_path_string!(dir, &major_version, &platform_directory),
                            join_path_string!(dir),
                        ]
                        .iter()
                        .map(|path| join_path_string!(&lib_prefix, path))
                        .collect_vec()
                    })
                    .collect_vec(),
            )
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        );
    }

    fn update_hash(
        &self,
        tool: &dyn Tool,
        sha: &mut sha2::Sha256,
        download: Download,
    ) -> Result<()> {
        download.update_hash(sha, &tool.name());
        Ok(())
    }

    fn major_version(&self, tool: &dyn Tool) -> String {
        if let Some(version) = tool.version() {
            format!("{}.0", version.split('.').take(2).join("."))
        } else {
            "unknown".to_string()
        }
    }

    fn rewrite_binstubs(&self, tool: &dyn Tool) -> Result<()> {
        let bin_dir = join_path_string!(tool.directory(), "bin");
        for entry in read_dir(&bin_dir)?
            .flatten()
            .filter(|entry| entry.path().is_file() && entry.file_name() != "ruby")
        {
            let contents = std::fs::read_to_string(&entry.path())?;
            let mut lines = contents.lines().map(String::from).collect_vec();
            if lines[0].starts_with("#!") {
                let new_line = format!("#!{}", join_path_string!(&bin_dir, "ruby"));
                debug!(
                    "Rewriting binstub: {:?}: {} -> {}",
                    entry.path(),
                    lines[0],
                    new_line
                );
                lines[0] = new_line;
                std::fs::write(entry.path(), lines.join("\n"))?;
            }
        }

        Ok(())
    }
}

impl Tool for Ruby {
    fn name(&self) -> String {
        "ruby".to_string()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Runtime
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        self.platform_tool.update_hash(self, sha, self.download())
    }

    fn version(&self) -> Option<String> {
        self.platform_tool.version(&self.version)
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        self.platform_tool.install(self, task, self.download())
    }

    fn post_install(&self, task: &ProgressTask) -> Result<()> {
        self.platform_tool.post_install(self, task)
    }

    fn extra_env_paths(&self) -> Vec<String> {
        self.platform_tool.extra_env_paths(self)
    }

    fn extra_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        self.platform_tool.extra_env_vars(self, &mut env);
        env
    }

    fn version_command(&self) -> Option<String> {
        Some("ruby --version".to_string())
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

impl Ruby {
    pub fn new_tool(version: &str) -> Box<dyn Tool> {
        if Self::binary_install_enabled() {
            Box::new(Self {
                version: version.to_string(),
                platform_tool: sys::platform::Ruby::default(),
            })
        } else {
            Box::new(RubySource {
                version: version.to_string(),
            })
        }
    }

    // because Rust doesn't support trait upcasting in stable releases
    pub fn new_runtime(version: &str) -> Box<dyn RuntimeTool> {
        if Self::binary_install_enabled() {
            Box::new(Self {
                version: version.to_string(),
                platform_tool: sys::platform::Ruby::default(),
            })
        } else {
            Box::new(RubySource {
                version: version.to_string(),
            })
        }
    }

    fn binary_install_enabled() -> bool {
        if cfg!(windows) {
            return true; // always use the binary install code path on Windows
        }
        if let Ok(value) = std::env::var("QLTY_FEATURE_RUBY_BINARY_INSTALL") {
            if ["false", "off", "0"].contains(&value.to_ascii_lowercase().as_str()) {
                return false;
            }
        }
        true
    }

    fn download(&self) -> Download {
        Download::new(
            &DownloadDef {
                strip_components: 1,
                systems: vec![
                    System {
                        url:
                            "https://github.com/ruby/ruby-builder/releases/download/toolcache/ruby-${version}-macos-13-arm64.tar.gz"
                                .to_string(),
                        cpu: Cpu::Aarch64,
                        os: OperatingSystem::MacOS,
                    },
                    System {
                        url:
                            "https://github.com/ruby/ruby-builder/releases/download/toolcache/ruby-${version}-macos-latest.tar.gz"
                                .to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::MacOS,
                    },
                    System {
                        url:
                            "https://github.com/ruby/ruby-builder/releases/download/toolcache/ruby-${version}-ubuntu-20.04.tar.gz"
                                .to_string(),
                        cpu: Cpu::Aarch64,
                        os: OperatingSystem::Linux,
                    },
                    System {
                        url:
                            "https://github.com/ruby/ruby-builder/releases/download/toolcache/ruby-${version}-ubuntu-20.04.tar.gz"
                                .to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::Linux,
                    },
                ],
                ..Default::default()
            },
            &self.name(),
            &self.version,
        )
    }
}

impl RuntimeTool for Ruby {
    fn package_tool(&self, name: &str, plugin: &PluginDef) -> Box<dyn Tool> {
        Box::new(RubygemsPackage {
            name: name.to_owned(),
            plugin: plugin.clone(),
            runtime: self.clone_box(),
            cmd: default_command_builder(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct RubygemsPackage {
    pub name: String,
    pub plugin: PluginDef,
    pub runtime: Box<dyn Tool>,
    pub cmd: Box<dyn CommandBuilder>,
}

impl Tool for RubygemsPackage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::RuntimePackage
    }

    fn runtime(&self) -> Option<Box<dyn Tool>> {
        Some(self.runtime.clone())
    }

    fn version(&self) -> Option<String> {
        if self.plugin.package_file.is_some() {
            Some("bundled".to_string())
        } else {
            self.plugin.version.clone()
        }
    }

    fn version_command(&self) -> Option<String> {
        if self.plugin.package_file.is_some() {
            None // version should come from package_file
        } else {
            self.plugin.version_command.clone()
        }
    }

    fn version_regex(&self) -> String {
        self.plugin.version_regex.clone()
    }

    fn package_install(&self, task: &ProgressTask, name: &str, version: &str) -> Result<()> {
        if self.plugin.package_file.is_some() {
            return Ok(()); // tool needs to be installed in Gemfile when bundler is used
        }

        task.set_message(&format!("gem install {}@{}", name, version));
        self.run_command(self.cmd.build(
            "ruby",
            vec![
                "-S",
                "gem",
                "install",
                name,
                "--no-document",
                "--version",
                version,
                "--install-dir",
                &path_to_native_string(self.directory()),
            ],
        ))
    }

    fn package_file_install(&self, task: &ProgressTask) -> Result<()> {
        self.gemfile_install(task)
    }

    fn extra_env_vars(&self) -> HashMap<String, String> {
        let mut env = self.runtime.extra_env_vars();
        env.insert(
            "GEM_HOME".to_string(),
            path_to_native_string(self.directory()),
        );
        env.insert(
            "GEM_PATH".to_string(),
            path_to_native_string(self.directory()),
        );

        self.package_file_envs(&mut env);

        env
    }

    fn extra_env_paths(&self) -> Vec<String> {
        if self.plugin.package_file.is_some() {
            if let Ok(version_paths) = read_dir(join_path_string!(self.directory(), "ruby")) {
                let paths = version_paths
                    .into_iter()
                    .flatten()
                    .filter(|entry| entry.path().is_dir())
                    .map(|entry| path_to_native_string(entry.path().join("bin")))
                    .collect_vec();
                if !paths.is_empty() {
                    return paths;
                }
            }
        }
        vec![join_path_string!(self.directory(), "bin"), self.directory()]
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }
}

#[cfg(test)]
pub mod test {
    use super::{Ruby, RubygemsPackage};
    use crate::{
        tool::{
            command_builder::test::{reroute_tools_root, stub_cmd, ENV_LOCK},
            ruby::sys::platform,
            ruby_source::RubySource,
        },
        ui::ProgressTask,
        Progress, Tool,
    };
    use itertools::Itertools;
    use qlty_analysis::{join_path_string, utils::fs::path_to_native_string};
    use qlty_config::config::PluginDef;
    use std::sync::{Arc, Mutex};
    use tempfile::{tempdir, TempDir};

    pub fn with_rubygems_package(
        callback: impl Fn(
            &mut RubygemsPackage,
            &TempDir,
            &Arc<Mutex<Vec<Vec<String>>>>,
        ) -> anyhow::Result<()>,
    ) {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| {
            ENV_LOCK.clear_poison();
            err.into_inner()
        });
        let list = Arc::new(Mutex::new(Vec::<Vec<String>>::new()));
        let temp_path = tempdir().unwrap();
        let mut pkg = RubygemsPackage {
            cmd: stub_cmd(list.clone()),
            name: "tool".into(),
            plugin: PluginDef {
                package: Some("test".to_string()),
                version: Some("1.0.0".to_string()),
                ..Default::default()
            },
            runtime: super::Ruby::new_tool("1.0.0"),
        };
        reroute_tools_root(&temp_path, &pkg);
        callback(&mut pkg, &temp_path, &list).unwrap();
        drop(temp_path);
    }

    pub fn new_task() -> ProgressTask {
        Progress::new(false, 1).task("PREFIX", "message")
    }

    #[test]
    fn test_ruby_binary_install_feature_flag() {
        let version = "1.0.0".to_string();

        let ruby_fingerprint = Ruby {
            platform_tool: platform::Ruby::default(),
            version: version.clone(),
        }
        .fingerprint();
        let ruby_source_fingerprint = if cfg!(windows) {
            ruby_fingerprint.clone() // don't try to load RubySource on Windows
        } else {
            RubySource {
                version: version.clone(),
            }
            .fingerprint()
        };

        let tests = [
            ("", &ruby_fingerprint),
            ("true", &ruby_fingerprint),
            ("anything", &ruby_fingerprint),
            ("1", &ruby_fingerprint),
            ("off", &ruby_source_fingerprint),
            ("false", &ruby_source_fingerprint),
            ("0", &ruby_source_fingerprint),
            ("FALSE", &ruby_source_fingerprint),
            ("OFF", &ruby_source_fingerprint),
            ("fAlSe", &ruby_source_fingerprint),
        ];
        for (flag, expected) in tests.iter() {
            std::env::set_var("QLTY_FEATURE_RUBY_BINARY_INSTALL", flag);
            assert_eq!(Ruby::new_tool(&version).fingerprint(), **expected);
        }

        std::env::remove_var("QLTY_FEATURE_RUBY_BINARY_INSTALL");
        assert_eq!(Ruby::new_tool(&version).fingerprint(), ruby_fingerprint);
    }

    #[test]
    fn test_rubygems_package_install_and_validate() {
        with_rubygems_package(|pkg, _, list| {
            pkg.install_and_validate(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                vec![vec![
                    "ruby",
                    "-S",
                    "gem",
                    "install",
                    "test",
                    "--no-document",
                    "--version",
                    "1.0.0",
                    "--install-dir",
                    &path_to_native_string(pkg.directory())
                ]]
            );
            Ok(())
        });
    }

    #[test]
    fn test_rubygems_package_env() {
        with_rubygems_package(|pkg, temp_path, _| {
            let env = pkg.env();

            if !cfg!(windows) {
                assert_eq!(
                    env.get("PATH")
                        .unwrap()
                        .split(':')
                        .filter(|x| x.starts_with(temp_path.path().to_str().unwrap()))
                        .sorted()
                        .collect::<Vec<_>>(),
                    vec![
                        // make sure that Ruby runtime is in PATH
                        join_path_string!(pkg.runtime().unwrap().directory(), "bin"),
                        pkg.directory(),
                        join_path_string!(pkg.directory(), "bin"),
                    ]
                );
            }
            assert_eq!(
                env.get("GEM_HOME").unwrap(),
                &path_to_native_string(pkg.directory())
            );
            assert_eq!(
                env.get("GEM_PATH").unwrap(),
                &path_to_native_string(pkg.directory())
            );
            assert_eq!(env.get("BUNDLE_PATH"), None);
            assert_eq!(env.get("BUNDLE_GEMFILE"), None);
            assert_eq!(env.get("RUBYOPT"), None);
            Ok(())
        });
    }
}
