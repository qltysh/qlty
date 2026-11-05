use super::RubygemsPackage;
use crate::ui::ProgressBar;
use crate::{tool::Tool, ui::ProgressTask};
use anyhow::{bail, Result};
use itertools::Itertools;
use qlty_analysis::{join_path_string, utils::fs::path_to_native_string};
use std::{collections::HashMap, path::PathBuf};
use tracing::debug;

pub type RubyGemfile = RubygemsPackage;

const SOURCE_PREAMBLE: &str = r#"source "https://rubygems.org"

# Added by Qlty. Global sources are replaced by rubygems.org above. A source
# block is kept only when Bundler has credentials for it, so its gems can
# install; otherwise its gems are left out. The Ruby version is Qlty's own.
def source(url, *args, &block)
  return unless block

  # Not `URI`, which would activate the default uri gem before Bundler checks
  # the version in the lockfile.
  _, userinfo, host = url.to_s.match(%r{\A[a-z][a-z0-9+.-]*://(?:([^@/]*)@)?([^/:?#]+)}i).to_a
  if url == :rubygems ||
      (host && (host == "rubygems.org" || !userinfo.to_s.delete(":").empty? ||
        Bundler.settings[host] || (Bundler.settings[url.to_s] rescue nil)))
    return super
  end
  # Bundler also evaluates the Gemfile with a subclass to find plugins.
  return unless instance_of?(Bundler::Dsl)

  if host
    $stderr.puts "Qlty skipped the gems in the source block for #{host} because Bundler " \
      "has no credentials for it. To install them, set " \
      "BUNDLE_#{host.upcase.gsub(".", "__").gsub("-", "___")} in the plugin's environment. " \
      "See https://docs.qlty.sh/cloud/build-secrets"
  else
    $stderr.puts "Qlty skipped the gems in a source block with no host."
  end
end

"#;

impl RubyGemfile {
    pub fn gemfile_install(&self, task: &ProgressTask) -> Result<()> {
        self.copy_package_file(task)?;

        self.run_command(
            self.cmd
                .build("ruby", vec!["-S", "bundle", "install"])
                .env_remove("RUBYOPT"), // having RUBYOPT here will throw off bundle install
        )?;

        self.run_command(self.cmd.build("ruby", vec!["-S", "bundle", "list"]))
    }

    pub fn package_file_declares_custom_source(&self) -> bool {
        let Some(package_file) = self.plugin.package_file.as_ref() else {
            return false;
        };
        let Ok(contents) = std::fs::read_to_string(package_file) else {
            return false;
        };

        contents.lines().any(|line| {
            let line = line.trim();
            line.starts_with("source")
                && line["source".len()..].starts_with([' ', '('])
                && !line.contains("rubygems.org")
                && !line.contains(":rubygems")
        })
    }

    pub fn copy_package_file(&self, task: &ProgressTask) -> Result<()> {
        if self.plugin.package_file.is_none() {
            bail!("No package file provided");
        }

        let package_file = self.plugin.package_file.as_ref().unwrap();
        let package_file_name = self.package_file_name();
        task.set_dim_message(&format!("bundle install ({})", package_file_name));

        let package_file_contents = std::fs::read_to_string(package_file)?;
        // Without filters, Bundler evaluates the whole Gemfile, so its `source`
        // blocks are decided in Ruby rather than rewritten as text.
        let new_package_file_contents = if self.plugin.package_filters.is_empty() {
            format!(
                "{SOURCE_PREAMBLE}{}",
                blank_global_directives(&package_file_contents)
            )
        } else {
            self.filtered_package_file(&package_file_contents)
        };

        let uses_gemspec = package_file_contents
            .lines()
            .any(|line| line.split_whitespace().next() == Some("gemspec"));
        if uses_gemspec {
            self.copy_gemspec_files(package_file, &self.name, &self.plugin.package_filters)?;
        }

        let new_package_file_path = join_path_string!(self.directory(), package_file_name);
        std::fs::write(new_package_file_path, new_package_file_contents)?;

        if self.plugin.package_filters.is_empty() {
            let package_file_path = PathBuf::from(package_file);
            if let Some(parent_dir) = package_file_path.parent() {
                let lock_file = parent_dir.join("Gemfile.lock");
                if lock_file.exists() {
                    let staging_lock_file = join_path_string!(self.directory(), "Gemfile.lock");

                    debug!(
                        "Copying lock file from {} to {}",
                        lock_file.display(),
                        staging_lock_file
                    );

                    std::fs::copy(lock_file, staging_lock_file)?;
                }
            }
        }

        Ok(())
    }

    // Only the gems the filters select and any `gemspec` line are kept, all
    // from rubygems.org.
    fn filtered_package_file(&self, contents: &str) -> String {
        let mut lines = vec!["source 'https://rubygems.org'"];
        for line in contents.lines() {
            let line = line.trim();
            let mut tokens = line.split_whitespace();
            match tokens.next().unwrap_or_default() {
                "gem" => {
                    let gem_name = tokens
                        .next()
                        .unwrap_or_default()
                        .replace(['\'', '"', ','], "");
                    if self.name == gem_name
                        || self
                            .plugin
                            .package_filters
                            .iter()
                            .any(|filter| gem_name.contains(filter))
                    {
                        lines.push(line);
                    }
                }
                "gemspec" => lines.push(line),
                _ => {}
            }
        }
        lines.join("\n")
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
            .full_env(self.env()?)
            .dir(source.parent().unwrap())
            .stdin_path(source)
            .stdout_path(dest);

        debug!("Interpolated gemspec file: {:?}", cmd);
        cmd.run().map(|_| ()).map_err(Into::into)
    }

    pub fn package_file_envs(&self, env: &mut HashMap<String, String>) -> Result<()> {
        if self.plugin.package_file.is_none() {
            return Ok(());
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
            runtime.env()?.get("RUBYOPT").cloned()
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

        Ok(())
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

// `ruby` lines and one-line global `source` lines are dropped rather than
// evaluated, as before, because their arguments often read something the tool
// directory doesn't have, such as `File.read(".ruby-version")` or
// `ENV.fetch("TOKEN")`. A `source` line with a `do`, or one that continues onto
// the next line, may open a block, so it's left for the preamble. Lines are left
// blank so the lines after them keep their line numbers.
fn blank_global_directives(contents: &str) -> String {
    contents
        .split_inclusive('\n')
        .map(|line| {
            let mut tokens = line.split_whitespace();
            let blank = match tokens.next() {
                Some("ruby") => true,
                Some("source") => {
                    !line.trim_end().ends_with([',', '\\']) && !tokens.any(|token| token == "do")
                }
                _ => false,
            };
            if blank {
                "\n"
            } else {
                line
            }
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::{blank_global_directives, SOURCE_PREAMBLE};
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

            let env = pkg.env().unwrap();
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

            let env = pkg.env().unwrap();
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
                    ruby File.read(".ruby-version").strip
                    gemspec
                    gem 'tool', '1.0.0'
                    gem 'tool_dep', '1.2.0'
                    gem 'unrelated', '1.0.0'
                    source 'https://gems.example.com' do
                      gem 'private'
                    end
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
                std::fs::read_to_string(filtered_package_file_path)?,
                format!(
                    "{SOURCE_PREAMBLE}{}",
                    indoc! {r#"


                        gemspec
                        gem 'tool', '1.0.0'
                        gem 'tool_dep', '1.2.0'
                        gem 'unrelated', '1.0.0'
                        source 'https://gems.example.com' do
                          gem 'private'
                        end
                    "#}
                )
            );
            Ok(())
        });
    }

    #[test]
    fn test_blank_global_directives_keeps_source_continued_onto_next_line() {
        let gemfile = indoc! {r#"
            source "https://gems.example.com",
                   type: "rubygems" do
              gem "private"
            end
        "#};
        assert_eq!(blank_global_directives(gemfile), gemfile);
    }

    #[test]
    fn test_lock_file_copied_without_filters() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            let lock_file = temp_path.path().join("Gemfile.lock");

            // Write a Gemfile and Gemfile.lock
            std::fs::write(&req_file, "source 'https://rubygems.org'").unwrap();
            std::fs::write(&lock_file, "GEM\n  specs:\n    rake (13.0.1)").unwrap();

            // Setup the plugin
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            pkg.plugin.package_filters = vec![];
            reroute_tools_root(temp_path, pkg);

            // Run copy_package_file
            pkg.copy_package_file(&new_task()).unwrap();

            let copied_lock_path = Path::new(&pkg.directory()).join("Gemfile.lock");
            assert!(copied_lock_path.exists(), "Expected lock file to be copied");

            let copied_content = std::fs::read_to_string(copied_lock_path).unwrap();
            assert!(
                copied_content.contains("rake"),
                "Expected lock file to contain copied contents"
            );

            Ok(())
        });
    }

    #[test]
    fn test_lock_file_not_copied_with_filters() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            let lock_file = temp_path.path().join("Gemfile.lock");

            // Write a Gemfile and Gemfile.lock
            std::fs::write(&req_file, "source 'https://rubygems.org'").unwrap();
            std::fs::write(&lock_file, "GEM\n  specs:\n    rake (13.0.1)").unwrap();

            // Setup the plugin with filters
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            pkg.plugin.package_filters = vec!["somefilter".to_string()];
            reroute_tools_root(temp_path, pkg);

            // Run copy_package_file
            pkg.copy_package_file(&new_task()).unwrap();

            let copied_lock_path = Path::new(&pkg.directory()).join("Gemfile.lock");
            assert!(
                !copied_lock_path.exists(),
                "Expected lock file NOT to be copied"
            );

            Ok(())
        });
    }

    #[test]
    fn test_package_file_declares_custom_source() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                "source 'https://gems.example.com'\ngem 'tool', '1.0.0'",
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());

            assert!(pkg.package_file_declares_custom_source());
            Ok(())
        });
    }

    #[test]
    fn test_package_file_declares_custom_source_rubygems_symbol() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(&req_file, "source :rubygems\ngem 'tool', '1.0.0'").unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());

            assert!(!pkg.package_file_declares_custom_source());
            Ok(())
        });
    }

    #[test]
    fn test_package_file_declares_custom_source_parenthesized() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                "source('https://gems.example.com')\ngem 'tool', '1.0.0'",
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());

            assert!(pkg.package_file_declares_custom_source());
            Ok(())
        });
    }

    #[test]
    fn test_package_file_declares_custom_source_rubygems() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                "source 'https://rubygems.org'\ngem 'tool', '1.0.0'",
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());

            assert!(!pkg.package_file_declares_custom_source());
            Ok(())
        });
    }

    #[test]
    fn test_package_file_declares_custom_source_without_package_file() {
        with_rubygems_package(|pkg, _, _| {
            pkg.plugin.package_file = None;

            assert!(!pkg.package_file_declares_custom_source());
            Ok(())
        });
    }
}
