use super::RubygemsPackage;
use crate::tool::finalize_installation_from_cmd_result;
use crate::tool::installations::initialize_installation;
use crate::ui::ProgressBar;
use crate::{tool::Tool, ui::ProgressTask};
use anyhow::{bail, Context, Result};
use itertools::Itertools;
use qlty_analysis::{join_path_string, utils::fs::path_to_native_string};
use std::{collections::HashMap, path::PathBuf};
use tracing::{debug, info};

pub type RubyGemfile = RubygemsPackage;

impl RubyGemfile {
    pub fn gemfile_install(&self, task: &ProgressTask) -> Result<()> {
        self.copy_package_file(task)?;

        self.run_command(
            self.cmd
                .build("ruby", vec!["-S", "bundle", "install"])
                .env_remove("RUBYOPT"), // having RUBYOPT here will throw off bundle install
        )?;

        let list_command = self
            .cmd
            .build("ruby", vec!["-S", "bundle", "list"])
            .full_env(self.env())
            .dir(self.directory())
            .stderr_capture()
            .stdout_capture()
            .unchecked();

        let script = format!("{:?}", list_command);
        debug!(script);

        let mut installation = initialize_installation(self);
        let result = list_command.run();
        let _ = finalize_installation_from_cmd_result(self, &result, &mut installation, script);

        let list_output: std::process::Output =
            result.with_context(|| "Failed to run: ruby -S bundle list")?;

        let stdout = String::from_utf8_lossy(&list_output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&list_output.stderr).to_string();
        info!("ruby -S bundle list list stdout: {}", stdout);
        info!("ruby -S bundle list list stderr: {}", stderr);

        if !list_output.status.success() {
            bail!("Failed to run: ruby -S bundle list");
        }

        Ok(())
    }

    pub fn copy_package_file(&self, task: &ProgressTask) -> Result<()> {
        if self.plugin.package_file.is_none() {
            bail!("No package file provided");
        }

        let package_file = self.plugin.package_file.as_ref().unwrap();
        let package_file_name = self.package_file_name();
        task.set_dim_message(&format!("bundle install ({})", package_file_name));

        let package_file_contents = std::fs::read_to_string(package_file)?;
        let mut uses_gemspec = false;
        let mut new_package_file_contents = vec!["source 'https://rubygems.org'"];
        for line in package_file_contents.lines() {
            let line = line.trim();
            let mut tokens = line.split_whitespace();
            match tokens.next().unwrap_or_default() {
                "gem" => {
                    let gem_name = tokens
                        .next()
                        .unwrap_or_default()
                        .replace(['\'', '"', ','], "");
                    if self.plugin.package_filters.is_empty()
                        || self.name == gem_name
                        || self
                            .plugin
                            .package_filters
                            .iter()
                            .any(|filter| gem_name.contains(filter))
                    {
                        new_package_file_contents.push(line);
                    }
                }
                "gemspec" => {
                    uses_gemspec = true;
                    new_package_file_contents.push(line);
                }
                "source" | "ruby" => {}
                _ => {
                    if self.plugin.package_filters.is_empty() {
                        new_package_file_contents.push(line);
                    }
                }
            }
        }

        if uses_gemspec {
            self.copy_gemspec_files(package_file, &self.name, &self.plugin.package_filters)?;
        }

        let new_package_file_path = join_path_string!(self.directory(), package_file_name);
        std::fs::write(new_package_file_path, new_package_file_contents.join("\n"))?;

        Ok(())
    }

    fn copy_gemspec_files(
        &self,
        package_file: &String,
        tool_name: &String,
        package_filters: &[String],
    ) -> Result<()> {
        if let Some(package_path) = PathBuf::from(package_file).parent() {
            if let Ok(entries) = std::fs::read_dir(package_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().unwrap_or_default() == "gemspec" {
                        let dest = PathBuf::from(self.directory()).join(path.file_name().unwrap());
                        self.interpolate_gemspec(path, dest, tool_name, package_filters)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn interpolate_gemspec(
        &self,
        source: PathBuf,
        dest: PathBuf,
        tool_name: &String,
        package_filters: &[String],
    ) -> Result<()> {
        let script = format!(
            "s=eval(STDIN.read);s.dependencies.select!{{|d|/^{}$|{}/i=~d.name}};puts s.to_ruby",
            tool_name,
            package_filters.join("|")
        );
        let cmd = self
            .cmd
            .build("ruby", vec!["-rrubygems", "-e", script.as_str()])
            .env_remove("RUBYOPT")
            .full_env(self.env())
            .dir(source.parent().unwrap())
            .stdin_path(source)
            .stdout_path(dest);

        debug!("Interpolated gemspec file: {:?}", cmd);
        cmd.run().map(|_| ()).map_err(Into::into)
    }

    pub fn package_file_envs(&self, env: &mut HashMap<String, String>) {
        if self.plugin.package_file.is_none() {
            return;
        }

        env.insert(
            "BUNDLE_PATH".to_string(),
            path_to_native_string(self.directory()),
        );

        env.insert(
            "BUNDLE_GEMFILE".to_string(),
            PathBuf::from(path_to_native_string(self.directory()))
                .join(self.package_file_name())
                .to_str()
                .unwrap()
                .to_string(),
        );

        let rubyopt_prefix = if let Some(runtime) = self.runtime() {
            runtime.env().get("RUBYOPT").cloned()
        } else {
            None
        };
        env.insert(
            "RUBYOPT".to_string(),
            [rubyopt_prefix, Some("-rbundler/setup".to_string())]
                .iter()
                .flatten()
                .join(" "),
        );
        env.insert("BUNDLE_JOBS".to_string(), "4".to_string());
        env.insert("BUNDLE_RETRY".to_string(), "3".to_string());
    }

    fn package_file_name(&self) -> String {
        PathBuf::from(self.plugin.package_file.as_ref().unwrap())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        tool::{
            command_builder::test::reroute_tools_root,
            ruby::test::{new_task, with_rubygems_package},
        },
        Tool,
    };
    use indoc::indoc;
    use itertools::Itertools;
    use qlty_analysis::{
        join_path_string,
        utils::fs::{path_to_native_string, path_to_string},
    };
    use std::{env::split_paths, fs::create_dir_all, path::Path};

    #[test]
    fn test_package_file_install_and_validate() {
        with_rubygems_package(|pkg, temp_path, list| {
            let req_file = &temp_path.path().join("Gemfile");
            std::fs::write(req_file, "source 'https://rubygems.org'").unwrap();

            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            reroute_tools_root(temp_path, pkg);

            create_dir_all(join_path_string!(pkg.directory(), "ruby", "1.2.3", "bin")).unwrap();
            create_dir_all(join_path_string!(pkg.directory(), "ruby", "4", "bin")).unwrap();

            pkg.install_and_validate(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                vec![
                    vec!["ruby", "-S", "bundle", "install",],
                    vec!["ruby", "-S", "bundle", "list",]
                ],
            );
            assert_eq!(pkg.version(), Some("bundled".to_string()));
            assert_eq!(pkg.version_command(), None);

            let env = pkg.env();
            assert_eq!(
                env.get("BUNDLE_PATH").unwrap(),
                &path_to_native_string(pkg.directory())
            );

            let filtered_package_file_path = Path::new(&pkg.directory()).join("Gemfile");
            assert_eq!(
                env.get("BUNDLE_GEMFILE").unwrap(),
                &path_to_native_string(filtered_package_file_path.to_str().unwrap())
            );
            if cfg!(windows) {
                assert_eq!(env.get("RUBYOPT").unwrap(), "-rbundler/setup");
            } else {
                assert_eq!(
                    env.get("RUBYOPT").unwrap(),
                    "-rqlty_load_path -rbundler/setup"
                );
            }
            assert_eq!(
                split_paths(env.get("PATH").unwrap())
                    .take(2)
                    .map(path_to_string)
                    .sorted()
                    .collect::<Vec<_>>(),
                vec![
                    join_path_string!(pkg.directory(), "ruby", "1.2.3", "bin"),
                    join_path_string!(pkg.directory(), "ruby", "4", "bin")
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn test_package_file_install_and_validate_with_package_filter() {
        with_rubygems_package(|pkg, temp_path, list| {
            let req_file = &temp_path.path().join("Gemfile");
            std::fs::write(
                req_file,
                indoc! {r#"
                    source 'https://not-rubygems.org'
                    ruby '2.7.0'
                    gemspec
                    gem 'tool', '1.0.0'
                    gem 'tool_dep', '1.2.0'
                    gem 'unrelated', '1.0.0'
                "#},
            )
            .unwrap();

            for filename in vec!["gemfile1.gemspec", "gemfile2.gemspec"] {
                std::fs::write(
                    &temp_path.path().join(filename),
                    r#"
                        Gem::Specification.new do |s|
                            s.add_dependency('tool')
                            s.add_dependency('tool_dep')
                            s.add_dependency('unrelated')
                            s.add_development_dependency('tool_dep2')
                            s.add_development_dependency('unrelated2')
                        end
                    "#,
                )?;
            }

            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            pkg.plugin.package_filters = vec!["tool".to_owned(), "dep".to_owned()];
            reroute_tools_root(temp_path, pkg);

            create_dir_all(join_path_string!(pkg.directory(), "ruby", "1.2.3", "bin")).unwrap();
            create_dir_all(join_path_string!(pkg.directory(), "ruby", "4", "bin")).unwrap();

            pkg.install_and_validate(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                vec![
                    vec!["ruby", "-rrubygems", "-e", "s=eval(STDIN.read);s.dependencies.select!{|d|/^tool$|tool|dep/i=~d.name};puts s.to_ruby"],
                    vec!["ruby", "-rrubygems", "-e", "s=eval(STDIN.read);s.dependencies.select!{|d|/^tool$|tool|dep/i=~d.name};puts s.to_ruby"],
                    vec!["ruby", "-S", "bundle", "install"],
                    vec!["ruby", "-S", "bundle", "list",]
                ],
            );
            assert_eq!(pkg.version(), Some("bundled".to_string()));
            assert_eq!(pkg.version_command(), None);

            let env = pkg.env();
            assert_eq!(
                env.get("BUNDLE_PATH").unwrap(),
                &path_to_native_string(pkg.directory())
            );

            let filtered_package_file_path = Path::new(&pkg.directory()).join("Gemfile");

            assert_eq!(
                env.get("BUNDLE_GEMFILE").unwrap(),
                &path_to_native_string(filtered_package_file_path.to_str().unwrap())
            );
            if cfg!(windows) {
                assert_eq!(env.get("RUBYOPT").unwrap(), "-rbundler/setup");
            } else {
                assert_eq!(
                    env.get("RUBYOPT").unwrap(),
                    "-rqlty_load_path -rbundler/setup"
                );
            }
            assert_eq!(
                split_paths(env.get("PATH").unwrap())
                    .take(2)
                    .map(path_to_string)
                    .sorted()
                    .collect::<Vec<_>>(),
                vec![
                    join_path_string!(pkg.directory(), "ruby", "1.2.3", "bin"),
                    join_path_string!(pkg.directory(), "ruby", "4", "bin")
                ]
            );

            assert_eq!(
                std::fs::read_to_string(filtered_package_file_path)?.trim(),
                indoc! {r#"
                    source 'https://rubygems.org'
                    gemspec
                    gem 'tool', '1.0.0'
                    gem 'tool_dep', '1.2.0'
                "#}
                .trim_end()
            );
            assert!(!std::fs::read_to_string(
                Path::new(&pkg.directory()).join("gemfile1.gemspec")
            )?
            .contains("unrelated"));
            assert!(!std::fs::read_to_string(
                Path::new(&pkg.directory()).join("gemfile2.gemspec")
            )?
            .contains("unrelated"));
            Ok(())
        });
    }

    #[test]
    fn test_package_file_install_and_validate_with_no_package_filter() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = &temp_path.path().join("Gemfile");
            std::fs::write(
                req_file,
                indoc! {r#"
                    source 'https://not-rubygems.org'
                    ruby '2.7.0'
                    gemspec
                    gem 'tool', '1.0.0'
                    gem 'tool_dep', '1.2.0'
                    gem 'unrelated', '1.0.0'
                "#},
            )
            .unwrap();

            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            reroute_tools_root(temp_path, pkg);

            create_dir_all(join_path_string!(pkg.directory(), "ruby", "1.2.3", "bin")).unwrap();
            create_dir_all(join_path_string!(pkg.directory(), "ruby", "4", "bin")).unwrap();

            pkg.install_and_validate(&new_task())?;

            let filtered_package_file_path = Path::new(&pkg.directory()).join("Gemfile");
            assert_eq!(
                std::fs::read_to_string(filtered_package_file_path)?.trim(),
                indoc! {r#"
                    source 'https://rubygems.org'
                    gemspec
                    gem 'tool', '1.0.0'
                    gem 'tool_dep', '1.2.0'
                    gem 'unrelated', '1.0.0'
                "#}
                .trim_end()
            );
            Ok(())
        });
    }
}
