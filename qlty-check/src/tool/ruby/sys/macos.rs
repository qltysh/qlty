use crate::{
    tool::{
        command_builder::CommandBuilder, finalize_installation_from_cmd_result,
        installations::initialize_installation, ruby::PlatformRuby,
    },
    ui::{ProgressBar, ProgressTask},
    Tool,
};
use anyhow::Result;
use itertools::Itertools;
use qlty_analysis::{join_path_string, utils::fs::path_to_string};
use std::{collections::HashMap, fs::read_dir};
use tracing::debug;

#[cfg(target_arch = "x86_64")]
const ARCH: &str = "x64";
#[cfg(target_arch = "aarch64")]
const ARCH: &str = "arm64";

const HOMEBREW_PATH: &str = "/opt/homebrew/bin";

#[derive(Debug, Clone, Default)]
pub struct RubyMacos {
    cmd_builder: Box<dyn CommandBuilder>,
}

impl PlatformRuby for RubyMacos {
    fn post_install(&self, tool: &dyn Tool, task: &ProgressTask) -> Result<()> {
        task.set_message("Setting up Ruby on macOS");
        let major_version = self.major_version(tool).split('.').take(2).join(".");

        self.rewrite_binstubs(tool)?;

        // ruby-builder places Ruby binaries in a specific location:
        // https://github.com/ruby/setup-ruby?tab=readme-ov-file#using-self-hosted-runners
        let cmd = self
            .cmd_builder
            .build(
                "install_name_tool",
                vec![
                    "-change",
                    format!(
                        "/Users/runner/hostedtoolcache/Ruby/{}/{}/lib/libruby.{}.dylib",
                        tool.version().unwrap_or_default(),
                        ARCH,
                        major_version
                    )
                    .as_str(),
                    join_path_string!(
                        tool.directory(),
                        "lib",
                        format!("libruby.{}.dylib", major_version)
                    )
                    .as_str(),
                    join_path_string!(tool.directory(), "bin", "ruby").as_str(),
                ],
            )
            .dir(tool.directory())
            .full_env(tool.env()?);

        debug!("Running: {:?}", cmd);

        let script = format!("{:?}", cmd);
        debug!(script);

        let mut installation = initialize_installation(tool)?;
        let result = cmd.run();
        finalize_installation_from_cmd_result(tool, &result, &mut installation, script).ok();

        result?;

        Ok(())
    }

    fn extra_env_paths(&self, tool: &dyn Tool) -> Vec<String> {
        vec![
            join_path_string!(tool.directory(), "bin"),
            HOMEBREW_PATH.to_string(),
        ]
    }

    fn extra_env_vars(&self, tool: &dyn Tool, env: &mut HashMap<String, String>) -> Result<()> {
        self.insert_rubylib_env(tool, env)?;
        env.insert(
            "PKG_CONFIG_PATH".to_string(),
            join_path_string!(tool.directory(), "lib", "pkgconfig"),
        );
        Ok(())
    }

    /// On macOS this can be <arch>-darwin19 or <arch>-darwin22 depending on the
    /// version of darwin this was built against. The most reliable way to get
    /// this is to look at the first directory in site_ruby and use that.
    fn platform_directory(&self, tool: &dyn Tool) -> String {
        let mut dir = join_path_string!(tool.directory(), "lib", "ruby", "site_ruby");
        if let Ok(version_dirs) = read_dir(&dir) {
            let version = self.major_version(tool);
            for entry in version_dirs.flatten().filter(|entry| entry.path().is_dir()) {
                if path_to_string(entry.file_name()).starts_with(&version) {
                    dir = path_to_string(entry.path());
                    break;
                }
            }
        }
        if let Ok(subdirs) = read_dir(&dir) {
            subdirs
                .flatten()
                .filter(|entry| entry.path().is_dir())
                .map(|entry| path_to_string(entry.file_name()))
                .collect_vec()
                .first()
                .cloned()
                .unwrap_or_default()
        } else {
            // fail gracefully since we don't want to panic on a faulty unwrap()
            debug!("Failed to find {:?}, returning default.", &dir);
            format!("{}-darwin22", ARCH)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tool::ruby::sys::macos::RubyMacos;
    use crate::tool::ruby::PlatformRuby;
    use crate::{
        tool::{command_builder::test::stub_cmd, ruby::sys::macos::ARCH, ToolType},
        Progress, Tool,
    };
    use qlty_analysis::utils::fs::path_to_string;
    use std::{
        env::join_paths,
        path::PathBuf,
        sync::{Arc, Mutex},
    };
    use tempfile::TempDir;

    #[derive(Debug, Clone, Default)]
    struct TestTool {
        version: String,
        directory: PathBuf,
    }

    impl Tool for TestTool {
        fn name(&self) -> String {
            "test".to_string()
        }

        fn version(&self) -> Option<String> {
            Some(self.version.clone())
        }

        fn tool_type(&self) -> crate::tool::ToolType {
            ToolType::Runtime
        }

        fn version_command(&self) -> Option<String> {
            None
        }

        fn clone_box(&self) -> Box<dyn Tool> {
            Box::new(self.clone())
        }

        fn directory(&self) -> String {
            path_to_string(&self.directory)
        }
    }

    #[test]
    fn test_post_install() {
        let tempdir = TempDir::new().unwrap();
        std::fs::create_dir_all(tempdir.path().join("bin")).unwrap();
        std::fs::write(tempdir.path().join("bin/ruby"), "BINARY_DATA").unwrap();
        std::fs::write(tempdir.path().join("bin/irb"), "#!/some/ruby abc\nscript").unwrap();
        std::fs::write(tempdir.path().join("bin/gem"), "#!/path/to/ruby\nscript").unwrap();
        std::fs::write(tempdir.path().join("bin/other"), "regular_script").unwrap();
        let tool = TestTool {
            directory: tempdir.path().to_path_buf(),
            version: "128.128.128".to_string(),
        };
        let list = Arc::new(Mutex::new(vec![]));
        let runtime = RubyMacos {
            cmd_builder: stub_cmd(list.clone()),
        };
        let task = Progress::new(false, 1).task("", "");
        runtime.post_install(&tool, &task).unwrap();

        // check that ruby binary was not rewritten
        assert_eq!(
            std::fs::read_to_string(tempdir.path().join("bin/ruby")).unwrap(),
            "BINARY_DATA"
        );

        // check that irb and gem shebangs were rewritten
        assert_eq!(
            std::fs::read_to_string(tempdir.path().join("bin/irb")).unwrap(),
            format!("#!{}/bin/ruby\nscript", path_to_string(tempdir.path()))
        );
        assert_eq!(
            std::fs::read_to_string(tempdir.path().join("bin/gem")).unwrap(),
            format!("#!{}/bin/ruby\nscript", path_to_string(tempdir.path()))
        );

        // check that other script was not rewritten
        assert_eq!(
            std::fs::read_to_string(tempdir.path().join("bin/other")).unwrap(),
            "regular_script"
        );

        // check proper install_name_tool
        assert_eq!(
            list.as_ref().lock().unwrap().clone(),
            [[
                "install_name_tool",
                "-change",
                format!(
                    "/Users/runner/hostedtoolcache/Ruby/128.128.128/{}/lib/libruby.128.128.dylib",
                    ARCH
                )
                .as_str(),
                format!(
                    "{}/lib/libruby.128.128.dylib",
                    path_to_string(tempdir.path())
                )
                .as_str(),
                format!("{}/bin/ruby", path_to_string(tempdir.path())).as_str()
            ]]
        );
        drop(tempdir);
    }

    #[test]
    fn test_platform_directory() {
        let tempdir = TempDir::new().unwrap();
        std::fs::create_dir_all(tempdir.path().join("lib/ruby/site_ruby/9.9.0/TEST_DIR")).unwrap();
        let tool = TestTool {
            directory: tempdir.path().to_path_buf(),
            version: "9.9.9".to_string(),
        };
        let runtime = RubyMacos::default();
        assert_eq!(runtime.platform_directory(&tool), "TEST_DIR");

        // default when path does not exist
        let tempdir = TempDir::new().unwrap();
        let tool = TestTool {
            directory: tempdir.path().to_path_buf(),
            version: "9.9.9".to_string(),
        };
        let runtime = RubyMacos::default();
        assert_eq!(
            runtime.platform_directory(&tool),
            format!("{}-darwin22", ARCH)
        );
        drop(tempdir);
    }

    #[test]
    fn test_different_version_directory() {
        let tempdir = TempDir::new().unwrap();
        std::fs::create_dir_all(tempdir.path().join("lib/ruby/site_ruby/9.9.0+1/ARCH")).unwrap();
        let tool = TestTool {
            directory: tempdir.path().to_path_buf(),
            version: "9.9.9".to_string(),
        };
        let mut env = std::collections::HashMap::new();
        let runtime = RubyMacos::default();
        runtime.extra_env_vars(&tool, &mut env).unwrap();
        assert_eq!(
            *env.get("PKG_CONFIG_PATH").unwrap(),
            format!("{}/lib/pkgconfig", path_to_string(tempdir.path()))
        );
        assert_eq!(
            *env.get("RUBYLIB").unwrap(),
            path_to_string(
                join_paths(vec![
                    tempdir.path().join("lib/ruby/site_ruby/9.9.0+1"),
                    tempdir.path().join("lib/ruby/site_ruby/9.9.0+1/ARCH"),
                    tempdir.path().join("lib/ruby/site_ruby"),
                    tempdir.path().join("lib/ruby/vendor_ruby/9.9.0"),
                    tempdir.path().join("lib/ruby/vendor_ruby/9.9.0/ARCH"),
                    tempdir.path().join("lib/ruby/vendor_ruby"),
                    tempdir.path().join("lib/ruby/9.9.0"),
                    tempdir.path().join("lib/ruby/9.9.0/ARCH"),
                    tempdir.path().join("lib/ruby/"),
                ])
                .unwrap()
            )
        );
        drop(tempdir);
    }

    #[test]
    fn test_extra_env_vars() {
        let tempdir = TempDir::new().unwrap();
        std::fs::create_dir_all(tempdir.path().join("lib/ruby/site_ruby/9.9.0/ARCH")).unwrap();
        let tool = TestTool {
            directory: tempdir.path().to_path_buf(),
            version: "9.9.9".to_string(),
        };
        let mut env = std::collections::HashMap::new();
        let runtime = RubyMacos::default();
        runtime.extra_env_vars(&tool, &mut env).unwrap();
        assert_eq!(
            *env.get("PKG_CONFIG_PATH").unwrap(),
            format!("{}/lib/pkgconfig", path_to_string(tempdir.path()))
        );
        assert_eq!(
            *env.get("RUBYLIB").unwrap(),
            path_to_string(
                join_paths(vec![
                    tempdir.path().join("lib/ruby/site_ruby/9.9.0"),
                    tempdir.path().join("lib/ruby/site_ruby/9.9.0/ARCH"),
                    tempdir.path().join("lib/ruby/site_ruby"),
                    tempdir.path().join("lib/ruby/vendor_ruby/9.9.0"),
                    tempdir.path().join("lib/ruby/vendor_ruby/9.9.0/ARCH"),
                    tempdir.path().join("lib/ruby/vendor_ruby"),
                    tempdir.path().join("lib/ruby/9.9.0"),
                    tempdir.path().join("lib/ruby/9.9.0/ARCH"),
                    tempdir.path().join("lib/ruby/"),
                ])
                .unwrap()
            )
        );
        drop(tempdir);
    }
}
