use super::RubygemsPackage;
use crate::ui::ProgressBar;
use crate::{tool::Tool, ui::ProgressTask};
use anyhow::{bail, Context, Result};
use itertools::Itertools;
use qlty_analysis::{join_path_string, utils::fs::path_to_native_string};
use std::{collections::HashMap, ops::RangeInclusive, path::PathBuf};
use tracing::debug;
use tree_sitter::Parser;
use url::Url;

pub type RubyGemfile = RubygemsPackage;

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

        let Ok(source_calls) = source_calls(&contents) else {
            return false;
        };

        let env = self.env().unwrap_or_default();
        source_calls
            .iter()
            .any(|call| !call.is_rubygems() && !self.keeps_credentialed_source(call, &env))
    }

    // A custom source is only kept when Bundler will receive its credentials.
    // Without them it would fail the install for gems the tool rarely needs.
    fn keeps_credentialed_source(&self, call: &SourceCall, env: &HashMap<String, String>) -> bool {
        self.plugin.package_filters.is_empty()
            && call.line_starts_with_source
            && call.has_credentials(env)
    }

    // The rows a source call decides and whether they are kept. Every other
    // row, including the body and `end` of a kept block, goes through the line
    // rules in copy_package_file. Those rules drop a line starting with `source`
    // but keep the rest of its block, leaving a stray `end`, so multi-line
    // blocks are always decided here.
    fn source_rows(
        &self,
        call: &SourceCall,
        keeps_global_source: bool,
        env: &HashMap<String, String>,
    ) -> Option<(RangeInclusive<usize>, bool)> {
        if !call.line_starts_with_source {
            return None;
        }
        if !self.plugin.package_filters.is_empty() {
            return call
                .has_multiline_block()
                .then(|| (call.opener_rows.clone(), false));
        }

        let keeps_rubygems = call.is_rubygems()
            && (call.has_multiline_block()
                || (call.block_end_row.is_none() && keeps_global_source));
        if keeps_rubygems || call.has_credentials(env) {
            return Some((call.opener_rows.clone(), true));
        }

        call.block_end_row
            .map(|end_row| (*call.opener_rows.start()..=end_row, false))
    }

    pub fn copy_package_file(&self, task: &ProgressTask) -> Result<()> {
        if self.plugin.package_file.is_none() {
            bail!("No package file provided");
        }

        let package_file = self.plugin.package_file.as_ref().unwrap();
        let package_file_name = self.package_file_name();
        task.set_dim_message(&format!("bundle install ({})", package_file_name));

        let package_file_contents = std::fs::read_to_string(package_file)?;
        let source_calls = source_calls(&package_file_contents)?;
        let env = self.env()?;
        // Bundler 4 rejects more than one global source, so a kept one replaces
        // the default rubygems.org source instead of adding to it.
        let keeps_global_source = source_calls
            .iter()
            .any(|call| call.block_end_row.is_none() && self.keeps_credentialed_source(call, &env));
        let decided_rows = source_calls
            .iter()
            .filter_map(|call| self.source_rows(call, keeps_global_source, &env))
            .collect_vec();
        let mut uses_gemspec = false;
        let mut new_package_file_contents = if keeps_global_source {
            vec![]
        } else {
            vec!["source 'https://rubygems.org'"]
        };
        for (row, line) in package_file_contents.lines().enumerate() {
            let line = line.trim();
            if let Some((_, keep)) = decided_rows.iter().find(|(rows, _)| rows.contains(&row)) {
                if *keep {
                    new_package_file_contents.push(line);
                }
                continue;
            }
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

struct SourceCall {
    // Every row of a one-line source, or the rows up to where its block opens.
    opener_rows: RangeInclusive<usize>,
    block_end_row: Option<usize>,
    line_starts_with_source: bool,
    url: String,
}

impl SourceCall {
    fn host(&self) -> Option<String> {
        let url = Url::parse(self.url.trim_matches(['"', '\''])).ok()?;
        url.host_str().map(str::to_string)
    }

    fn is_rubygems(&self) -> bool {
        self.url == ":rubygems"
            || self
                .host()
                .is_some_and(|host| host == "rubygems.org" || host.ends_with(".rubygems.org"))
    }

    fn has_credentials(&self, env: &HashMap<String, String>) -> bool {
        self.host()
            .is_some_and(|host| env.contains_key(&credential_key(&host)))
    }

    fn has_multiline_block(&self) -> bool {
        self.block_end_row
            .is_some_and(|end_row| end_row > *self.opener_rows.end())
    }
}

// Parsed rather than matched line by line, so comments, parentheses and
// multi-line calls can't hide where a `source` call or its block starts.
fn source_calls(contents: &str) -> Result<Vec<SourceCall>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_ruby::language())
        .context("Failed to load the Ruby grammar to parse the Gemfile")?;
    let tree = parser
        .parse(contents, None)
        .context("Failed to parse the Gemfile")?;
    let source = contents.as_bytes();
    let mut cursor = tree.walk();
    let mut nodes = vec![tree.root_node()];
    let mut calls = vec![];

    while let Some(node) = nodes.pop() {
        nodes.extend(node.children(&mut cursor));

        let is_source_call = node.kind() == "call"
            && node.child_by_field_name("receiver").is_none()
            && node
                .child_by_field_name("method")
                .and_then(|method| method.utf8_text(source).ok())
                == Some("source");
        if !is_source_call {
            continue;
        }

        let start_row = node.start_position().row;
        let block = node.child_by_field_name("block");
        let opener_end_row = match block {
            Some(block) => block.start_position().row,
            None => node.end_position().row,
        };
        calls.push(SourceCall {
            opener_rows: start_row..=opener_end_row,
            block_end_row: block.map(|block| block.end_position().row),
            line_starts_with_source: contents
                .lines()
                .nth(start_row)
                .and_then(|line| line.split_whitespace().next())
                == Some("source"),
            url: node
                .child_by_field_name("arguments")
                .and_then(|arguments| arguments.named_child(0))
                .and_then(|argument| argument.utf8_text(source).ok())
                .unwrap_or_default()
                .to_string(),
        });
    }

    Ok(calls)
}

// Mirrors Bundler::Settings.key_for, which looks up a source's credentials by host.
fn credential_key(host: &str) -> String {
    format!(
        "BUNDLE_{}",
        host.replace('.', "__").replace('-', "___").to_uppercase()
    )
}

#[cfg(test)]
mod test {
    use super::credential_key;
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
    use qlty_config::config::PluginEnvironment;
    use std::{env::split_paths, fs::create_dir_all, path::Path};

    fn bundler_credential(name: &str) -> Vec<PluginEnvironment> {
        vec![PluginEnvironment {
            name: name.to_string(),
            value: "user:pass".to_string(),
            ..Default::default()
        }]
    }

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
    fn test_credential_key() {
        assert_eq!(
            credential_key("my-gems.example.com"),
            "BUNDLE_MY___GEMS__EXAMPLE__COM"
        );
    }

    #[test]
    fn test_copy_package_file_drops_source_block_without_credentials() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    gem "rubocop", "1.92.0"
                    source "https://enterprise.contribsys.com/", cooldown: 7 do
                      gem 'sidekiq-pro', '< 9'
                      platforms :ruby do
                        gem 'sidekiq-ent', '< 9'
                      end
                    end
                    gem "rake"
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source 'https://rubygems.org'
                    gem "rubocop", "1.92.0"
                    gem "rake""#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_drops_rubygems_lookalike_source_block() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    gem "rubocop", "1.92.0"
                    source "https://nexus.example.com/repository/rubygems.org-private/" do
                      gem 'private'
                    end
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source 'https://rubygems.org'
                    gem "rubocop", "1.92.0""#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_keeps_rubygems_source_block() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    source "https://rubygems.org" do
                      gem "rubocop", "1.92.0"
                    end
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source 'https://rubygems.org'
                    source "https://rubygems.org" do
                    gem "rubocop", "1.92.0"
                    end"#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_keeps_credentialed_global_source() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    source "https://rubygems.org"
                    source "https://gems.example.com"
                    gem "rubocop", "1.92.0"
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            pkg.plugin.environment = bundler_credential("BUNDLE_GEMS__EXAMPLE__COM");
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source "https://rubygems.org"
                    source "https://gems.example.com"
                    gem "rubocop", "1.92.0""#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_keeps_credentialed_source_block() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    source "https://rubygems.org"
                    gem "rubocop", "1.92.0"
                    source "https://enterprise.contribsys.com/", cooldown: 7 do
                      gem 'sidekiq-pro', '< 9'
                      gem 'sidekiq-ent', '< 9'
                    end
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            pkg.plugin.environment = bundler_credential("BUNDLE_ENTERPRISE__CONTRIBSYS__COM");
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source 'https://rubygems.org'
                    gem "rubocop", "1.92.0"
                    source "https://enterprise.contribsys.com/", cooldown: 7 do
                    gem 'sidekiq-pro', '< 9'
                    gem 'sidekiq-ent', '< 9'
                    end"#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_keeps_credentialed_source_block_with_tab_before_comment() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                "source 'https://gems.example.com/' do\t# private\n  gem 'private'\nend\n",
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            pkg.plugin.environment = bundler_credential("BUNDLE_GEMS__EXAMPLE__COM");
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                "source 'https://rubygems.org'\nsource 'https://gems.example.com/' do\t# private\ngem 'private'\nend"
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_keeps_credentialed_source_block_with_comment_after_do() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    source 'https://gems.example.com/' do# private
                      gem 'private'
                    end
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            pkg.plugin.environment = bundler_credential("BUNDLE_GEMS__EXAMPLE__COM");
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source 'https://rubygems.org'
                    source 'https://gems.example.com/' do# private
                    gem 'private'
                    end"#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_drops_source_with_do_in_comment() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    source 'https://gems.example.com/'# nothing else to do
                    gem 'tool', '1.0.0'
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source 'https://rubygems.org'
                    gem 'tool', '1.0.0'"#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_keeps_credentialed_multiline_source_block_opener() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    source "https://enterprise.contribsys.com/",
                      cooldown: 7 do
                      gem 'sidekiq-ent', '< 9'
                    end
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            pkg.plugin.environment = bundler_credential("BUNDLE_ENTERPRISE__CONTRIBSYS__COM");
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source 'https://rubygems.org'
                    source "https://enterprise.contribsys.com/",
                    cooldown: 7 do
                    gem 'sidekiq-ent', '< 9'
                    end"#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_drops_multiline_one_line_source() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    source ENV.fetch("GEM_SOURCE",
                      "https://rubygems.org")
                    gem 'tool', '1.0.0'
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source 'https://rubygems.org'
                    gem 'tool', '1.0.0'"#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_drops_single_line_source_block() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    gem 'tool', '1.0.0'
                    source 'https://gems.example.com/' do gem 'private' end
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source 'https://rubygems.org'
                    gem 'tool', '1.0.0'"#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_keeps_parenthesized_source() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    source('https://gems.example.com/')
                    gem 'tool', '1.0.0'
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source 'https://rubygems.org'
                    source('https://gems.example.com/')
                    gem 'tool', '1.0.0'"#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_keeps_source_after_semicolon() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                "gem 'tool', '1.0.0'; source 'https://gems.example.com/'\n",
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                "source 'https://rubygems.org'\ngem 'tool', '1.0.0'; source 'https://gems.example.com/'"
            );
            Ok(())
        });
    }

    #[test]
    fn test_copy_package_file_drops_source_block_with_package_filters() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                indoc! {r#"
                    source "https://rubygems.org"
                    gem "rubocop", "1.92.0"
                    source "https://enterprise.contribsys.com/" do
                      gem 'sidekiq-ent', '< 9'
                    end
                "#},
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            pkg.plugin.package_filters = vec!["rubocop".to_string()];
            reroute_tools_root(temp_path, pkg);

            pkg.copy_package_file(&new_task()).unwrap();

            assert_eq!(
                std::fs::read_to_string(Path::new(&pkg.directory()).join("Gemfile"))?,
                indoc! {r#"
                    source 'https://rubygems.org'
                    gem "rubocop", "1.92.0""#}
            );
            Ok(())
        });
    }

    #[test]
    fn test_package_file_declares_custom_source_credentialed_block() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                "source 'https://gems.example.com' do # private\n  gem 'tool', '1.0.0'\nend",
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            pkg.plugin.environment = bundler_credential("BUNDLE_GEMS__EXAMPLE__COM");

            assert!(!pkg.package_file_declares_custom_source());
            Ok(())
        });
    }

    #[test]
    fn test_package_file_declares_custom_source_block_with_package_filters() {
        with_rubygems_package(|pkg, temp_path, _| {
            let req_file = temp_path.path().join("Gemfile");
            std::fs::write(
                &req_file,
                "source 'https://gems.example.com' do\n  gem 'tool', '1.0.0'\nend",
            )
            .unwrap();
            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            pkg.plugin.package_filters = vec!["tool".to_string()];

            assert!(pkg.package_file_declares_custom_source());
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
