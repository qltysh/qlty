use crate::Tool;

use super::NodePackage;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::debug;

pub type PackageJson = NodePackage;

impl PackageJson {
    // https://stackoverflow.com/questions/47070876
    pub fn merge_json(a: &mut Value, b: Value) {
        match (a, b) {
            (a @ &mut Value::Object(_), Value::Object(b)) => {
                let a = a.as_object_mut().unwrap();
                for (k, v) in b {
                    Self::merge_json(a.entry(k).or_insert(Value::Null), v);
                }
            }
            (a, b) => *a = b,
        }
    }

    pub fn update_package_json(
        &self,
        tool_name: &str,
        package_file: &Option<String>,
    ) -> Result<()> {
        let user_file_contents =
            std::fs::read_to_string(self.plugin.package_file.as_deref().unwrap_or_default())?;
        let mut user_json = serde_json::from_str::<Value>(&user_file_contents)?;
        let staged_file = PathBuf::from(self.directory()).join("package.json");
        let mut data_json = Value::Object(serde_json::Map::new());

        if let Some(root_object) = user_json.as_object_mut() {
            // ignore scripts section to avoid any npm install lifecycle events
            root_object.remove("scripts");

            // collapse devDependencies into dependencies
            if let Some(dev_dependencies) = root_object.clone().get("devDependencies") {
                if let Some(dependencies) = root_object.get_mut("dependencies") {
                    Self::merge_json(dependencies, dev_dependencies.clone());
                } else {
                    root_object.insert("dependencies".to_string(), dev_dependencies.clone());
                }
                root_object.remove("devDependencies");
            }

            // clear out unrelated deps
            if let Some(dependencies) = root_object.get_mut("dependencies") {
                self.remove_unrelated_dependencies(dependencies, tool_name);
                Self::update_file_dependencies(dependencies, package_file);
            }
        }

        if staged_file.exists() {
            // use the original package.json contents, merging package_file contents on top.
            // this will retain any existing dependencies provided by the initial tool installation
            let contents = std::fs::read_to_string(&staged_file)?;
            data_json = serde_json::from_str::<Value>(&contents).unwrap_or_default();
        }

        Self::merge_json(&mut user_json, data_json);

        let final_package_file = serde_json::to_string_pretty(&user_json)?;
        debug!("Writing {} package.json: {}", tool_name, final_package_file);

        if self.plugin.package_filters.is_empty() {
            if let Some(package_file) = &self.plugin.package_file {
                let package_file_path = PathBuf::from(package_file);
                if let Some(parent_path) = package_file_path.parent() {
                    let lock_file = parent_path.join("package-lock.json");

                    if lock_file.exists() {
                        let staged_file_parent = staged_file.parent().unwrap();
                        let staging_lock_file = staged_file_parent.join("package-lock.json");

                        debug!(
                            "Copying lock file from {} to {}",
                            lock_file.display(),
                            staging_lock_file.display()
                        );
                        std::fs::copy(lock_file, staging_lock_file)?;
                    }
                }
            }
        }

        std::fs::write(staged_file, final_package_file)?;

        Ok(())
    }

    // Filter out any dependencies that don't seem related to the plugin
    fn remove_unrelated_dependencies(&self, dependencies: &mut Value, tool_name: &str) {
        if dependencies.is_null() {
            return;
        }

        let filters = &self.plugin.package_filters;
        if !filters.is_empty() {
            if let Some(deps) = dependencies.as_object_mut() {
                deps.retain(|dep_name, _| {
                    dep_name == tool_name || filters.iter().any(|filter| dep_name.contains(filter))
                });
            }
        }
    }

    fn update_file_dependencies(dependencies: &mut Value, package_file: &Option<String>) {
        let workspace_packages = package_file
            .as_ref()
            .and_then(|f| Self::find_workspace_packages(f).ok())
            .unwrap_or_default();

        for (dep_name, value) in dependencies.as_object_mut().unwrap() {
            let version_string = value.as_str().unwrap_or_default().to_string();

            if version_string.starts_with("file:") {
                let path = PathBuf::from(package_file.clone().unwrap_or_default());
                let parent_path = path.parent().unwrap().to_str().unwrap();
                *value =
                    Value::from(version_string.replace("file:", &format!("file:{}/", parent_path)));
            } else if version_string.starts_with("workspace:") {
                if let Some(pkg_path) = workspace_packages.get(dep_name) {
                    *value = Value::from(format!("file:{}", pkg_path));
                }
            }
        }
    }

    // Walk up from package_file to find pnpm-workspace.yaml, then resolve
    // workspace package names to their absolute paths on disk.
    fn find_workspace_packages(package_file: &str) -> Result<HashMap<String, String>> {
        let mut packages = HashMap::new();

        let mut current = PathBuf::from(package_file);
        current.pop();

        let workspace_root = loop {
            let candidate = current.join("pnpm-workspace.yaml");
            if candidate.exists() {
                break Some((current.clone(), candidate));
            }
            if !current.pop() {
                break None;
            }
        };

        let (root, workspace_file) = match workspace_root {
            Some(v) => v,
            None => return Ok(packages),
        };

        let contents = std::fs::read_to_string(&workspace_file)?;
        let globs = Self::parse_pnpm_workspace_globs(&contents);

        for glob_pattern in globs {
            let base = glob_pattern
                .trim_end_matches("/**")
                .trim_end_matches("/*");
            let base_dir = root.join(base);

            if let Ok(entries) = std::fs::read_dir(&base_dir) {
                for entry in entries.flatten() {
                    let pkg_json_path = entry.path().join("package.json");
                    if let Ok(contents) = std::fs::read_to_string(&pkg_json_path) {
                        if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                            if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                                if let Some(pkg_dir) = entry.path().to_str() {
                                    packages.insert(name.to_string(), pkg_dir.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(packages)
    }

    fn parse_pnpm_workspace_globs(contents: &str) -> Vec<String> {
        let mut globs = Vec::new();
        let mut in_packages = false;

        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed == "packages:" {
                in_packages = true;
            } else if in_packages {
                if let Some(stripped) = trimmed.strip_prefix('-') {
                    let glob = stripped
                        .trim()
                        .trim_matches('\'')
                        .trim_matches('"')
                        .to_string();
                    if !glob.is_empty() {
                        globs.push(glob);
                    }
                } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    break;
                }
            }
        }

        globs
    }
}

#[cfg(test)]
mod test {
    use crate::{
        tool::{
            command_builder::test::reroute_tools_root,
            node::{package_json::PackageJson, test::with_node_package},
        },
        Tool,
    };
    use qlty_analysis::utils::fs::path_to_string;
    use serde_json::Value;
    use std::path::Path;

    #[test]
    fn merge_json_values() {
        let tests = [
            (r#"{}"#, r#"{"a":1}"#, r#"{"a":1}"#),
            (r#"{"a":1}"#, r#"{}"#, r#"{"a":1}"#),
            (r#"{"a":1}"#, r#"{"a":2}"#, r#"{"a":2}"#),
            (r#"{"a":1}"#, r#"{"b":2}"#, r#"{"a":1,"b":2}"#),
            (r#"{"a":1}"#, r#"{"a":2,"b":2}"#, r#"{"a":2,"b":2}"#),
            (r#"{"a":[1]}"#, r#"{"a":[2]}"#, r#"{"a":[2]}"#),
            (
                r#"{"a":{"b":1}}"#,
                r#"{"a":{"b":2,"c":2}}"#,
                r#"{"a":{"b":2,"c":2}}"#,
            ),
        ];

        for (a, b, expected) in tests.iter() {
            let mut a = serde_json::from_str(a).unwrap();
            let b = serde_json::from_str(b).unwrap();
            PackageJson::merge_json(&mut a, b);
            assert_eq!(a, serde_json::from_str::<Value>(expected).unwrap());
        }
    }

    #[test]
    fn update_package_json() {
        with_node_package(|pkg, tempdir, _| {
            let user_package_file = tempdir.path().join("user-package.json");
            let user_file_contents = r#"{
                "dependencies": {
                    "eslint": "7.0.0",
                    "@types/node": "13.0.0",
                    "typescript": "3.0.0"
                },
                "devDependencies": {
                    "eslint": "8.0.0",
                    "@types/node": "14.0.0",
                    "typescript": "4.0.0",
                    "other": "1.0.0"
                },
                "scripts": {
                    "test": "echo hello && exit 1"
                }
            }"#;
            std::fs::write(&user_package_file, user_file_contents)?;

            let tests = vec![
                (
                    vec![],
                    r#"{"dependencies":{"eslint":"1.0.0","@types/node":"14.0.0","typescript":"4.0.0","other":"1.0.0"}}"#,
                ),
                (
                    vec!["eslint", "type"],
                    r#"{"dependencies":{"eslint":"1.0.0","@types/node":"14.0.0","typescript":"4.0.0"}}"#,
                ),
                (
                    vec!["other"],
                    r#"{"dependencies":{"eslint":"1.0.0","other":"1.0.0"}}"#,
                ),
            ];
            for (filters, expected) in tests.iter() {
                pkg.plugin.package_file = Some(path_to_string(&user_package_file));
                pkg.plugin.package_filters = filters.iter().map(|s| s.to_string()).collect();
                reroute_tools_root(&tempdir, pkg);
                let stage_path = Path::new(&pkg.directory()).join("package.json");
                std::fs::write(&stage_path, r#"{"dependencies":{"eslint":"1.0.0"}}"#)?;

                pkg.update_package_json("eslint", &Some("package.json".to_string()))?;

                assert_eq!(
                    std::fs::read_to_string(&stage_path)?
                        .replace('\n', "")
                        .replace(' ', ""),
                    expected.to_string()
                );
            }

            Ok(())
        });
    }

    #[test]
    fn test_update_package_json_only_dev_deps() {
        with_node_package(|pkg, tempdir, _| {
            let user_package_file = tempdir.path().join("user-package.json");
            let user_file_contents = r#"{
                "devDependencies": {
                    "eslint": "8.0.0",
                    "@types/node": "14.0.0",
                    "typescript": "4.0.0",
                    "other": "1.0.0"
                },
                "scripts": {
                    "test": "echo hello && exit 1"
                }
            }"#;
            std::fs::write(&user_package_file, user_file_contents)?;

            let tests = vec![
                (
                    vec![],
                    r#"{"dependencies":{"eslint":"1.0.0","@types/node":"14.0.0","typescript":"4.0.0","other":"1.0.0"}}"#,
                ),
                (
                    vec!["eslint", "type"],
                    r#"{"dependencies":{"eslint":"1.0.0","@types/node":"14.0.0","typescript":"4.0.0"}}"#,
                ),
                (
                    vec!["other"],
                    r#"{"dependencies":{"eslint":"1.0.0","other":"1.0.0"}}"#,
                ),
            ];
            for (filters, expected) in tests.iter() {
                pkg.plugin.package_file = Some(path_to_string(&user_package_file));
                pkg.plugin.package_filters = filters.iter().map(|s| s.to_string()).collect();
                reroute_tools_root(&tempdir, pkg);
                let stage_path = Path::new(&pkg.directory()).join("package.json");
                std::fs::write(&stage_path, r#"{"dependencies":{"eslint":"1.0.0"}}"#)?;

                pkg.update_package_json("eslint", &Some("package.json".to_string()))?;

                assert_eq!(
                    std::fs::read_to_string(&stage_path)?
                        .replace('\n', "")
                        .replace(' ', ""),
                    expected.to_string()
                );
            }

            Ok(())
        });
    }

    #[test]
    fn test_update_package_file_based_dependency() {
        with_node_package(|pkg, tempdir, _| {
            let user_package_file = tempdir.path().join("user-package.json");
            let user_file_contents = r#"{
                "devDependencies": {
                    "eslint": "8.0.0",
                    "@types/node": "14.0.0",
                    "typescript": "4.0.0",
                    "other": "1.0.0",
                    "eslint-plugin": "file:packages/eslint-plugin"
                },
                "scripts": {
                    "test": "echo hello && exit 1"
                }
            }"#;
            std::fs::write(&user_package_file, user_file_contents)?;

            pkg.plugin.package_file = Some(path_to_string(&user_package_file));
            pkg.plugin.package_filters = vec!["eslint".to_string()];
            reroute_tools_root(&tempdir, pkg);
            let stage_path = Path::new(&pkg.directory()).join("package.json");
            std::fs::write(&stage_path, r#"{"dependencies":{"eslint":"1.0.0"}}"#)?;

            pkg.update_package_json("eslint", &Some("/Some/Path/to/package.json".to_string()))?;

            assert_eq!(
                std::fs::read_to_string(&stage_path)?
                    .replace('\n', "")
                    .replace(' ', ""),
                "{\"dependencies\":{\"eslint\":\"1.0.0\",\"eslint-plugin\":\"file:/Some/Path/to/packages/eslint-plugin\"}}".to_string()
            );

            Ok(())
        });
    }

    #[test]
    fn test_update_workspace_protocol_dependency() {
        with_node_package(|pkg, tempdir, _| {
            // Set up monorepo structure:
            //   pnpm-workspace.yaml
            //   packages/vitest-config/package.json  (name: @acme/vitest-config)
            //   apps/api/package.json  (depends on @acme/vitest-config: workspace:*)
            let workspace_root = tempdir.path();

            std::fs::write(
                workspace_root.join("pnpm-workspace.yaml"),
                "packages:\n  - 'packages/**'\n  - 'apps/**'\n",
            )?;

            let pkg_dir = workspace_root.join("packages").join("vitest-config");
            std::fs::create_dir_all(&pkg_dir)?;
            std::fs::write(
                pkg_dir.join("package.json"),
                r#"{"name":"@acme/vitest-config","version":"1.0.0"}"#,
            )?;

            let api_dir = workspace_root.join("apps").join("api");
            std::fs::create_dir_all(&api_dir)?;
            let user_package_file = api_dir.join("package.json");
            std::fs::write(
                &user_package_file,
                r#"{
                    "devDependencies": {
                        "knip": "5.88.1",
                        "@acme/vitest-config": "workspace:*"
                    }
                }"#,
            )?;

            pkg.plugin.package_file = Some(path_to_string(&user_package_file));
            pkg.plugin.package_filters = vec!["knip".to_string(), "acme".to_string()];
            reroute_tools_root(&tempdir, pkg);

            let stage_path = Path::new(&pkg.directory()).join("package.json");
            std::fs::write(&stage_path, r#"{"dependencies":{"knip":"5.88.1"}}"#)?;

            pkg.update_package_json("knip", &Some(path_to_string(&user_package_file)))?;

            let result: Value =
                serde_json::from_str(&std::fs::read_to_string(&stage_path)?)?;
            let deps = result.get("dependencies").unwrap().as_object().unwrap();

            // workspace:* should be resolved to a file: path pointing at the package on disk
            let vitest_config_dep = deps.get("@acme/vitest-config").unwrap().as_str().unwrap();
            assert!(
                vitest_config_dep.starts_with("file:"),
                "Expected file: path, got: {}",
                vitest_config_dep
            );
            assert!(
                vitest_config_dep.contains("vitest-config"),
                "Expected path to vitest-config package, got: {}",
                vitest_config_dep
            );

            Ok(())
        });
    }

    #[test]
    fn test_lock_file_copying_with_empty_filters() {
        with_node_package(|pkg, tempdir, _| {
            // Create package.json file
            let parent_dir = tempdir.path().join("project");
            std::fs::create_dir_all(&parent_dir)?;
            let user_package_file = parent_dir.join("package.json");
            let user_file_contents = r#"{
                "dependencies": {
                    "eslint": "8.0.0"
                }
            }"#;
            std::fs::write(&user_package_file, user_file_contents)?;

            // Create package-lock.json file
            let lock_file = parent_dir.join("package-lock.json");
            let lock_contents = r#"{
                "name": "test-project",
                "lockfileVersion": 2,
                "requires": true,
                "packages": {
                    "": {
                        "dependencies": {
                            "eslint": "8.0.0"
                        }
                    }
                }
            }"#;
            std::fs::write(&lock_file, lock_contents)?;

            // Configure package and execute
            pkg.plugin.package_file = Some(path_to_string(&user_package_file));
            pkg.plugin.package_filters = vec![]; // Empty filters should trigger lock file copying
            reroute_tools_root(&tempdir, pkg);

            let stage_path = Path::new(&pkg.directory()).join("package.json");
            std::fs::write(&stage_path, r#"{"dependencies":{"eslint":"1.0.0"}}"#)?;

            pkg.update_package_json("eslint", &Some("package.json".to_string()))?;

            // Verify lock file was copied
            let staged_lock_file = Path::new(&pkg.directory()).join("package-lock.json");
            assert!(staged_lock_file.exists(), "Lock file was not copied");

            let lock_content = std::fs::read_to_string(&staged_lock_file)?;
            assert!(
                lock_content.contains("eslint"),
                "Lock file contents are incorrect"
            );

            Ok(())
        });
    }

    #[test]
    fn test_lock_file_not_copied_with_filters() {
        with_node_package(|pkg, tempdir, _| {
            // Create package.json file
            let parent_dir = tempdir.path().join("project");
            std::fs::create_dir_all(&parent_dir)?;
            let user_package_file = parent_dir.join("package.json");
            let user_file_contents = r#"{
                "dependencies": {
                    "eslint": "8.0.0"
                }
            }"#;
            std::fs::write(&user_package_file, user_file_contents)?;

            // Create package-lock.json file
            let lock_file = parent_dir.join("package-lock.json");
            let lock_contents = r#"{
                "name": "test-project",
                "lockfileVersion": 2,
                "requires": true,
                "packages": {
                    "": {
                        "dependencies": {
                            "eslint": "8.0.0"
                        }
                    }
                }
            }"#;
            std::fs::write(&lock_file, lock_contents)?;

            // Configure package and execute
            pkg.plugin.package_file = Some(path_to_string(&user_package_file));
            pkg.plugin.package_filters = vec!["eslint".to_string()]; // With filters, lock file should not be copied
            reroute_tools_root(&tempdir, pkg);

            let stage_path = Path::new(&pkg.directory()).join("package.json");
            std::fs::write(&stage_path, r#"{"dependencies":{"eslint":"1.0.0"}}"#)?;

            pkg.update_package_json("eslint", &Some("package.json".to_string()))?;

            // Verify lock file was not copied
            let staged_lock_file = Path::new(&pkg.directory()).join("package-lock.json");
            assert!(
                !staged_lock_file.exists(),
                "Lock file was copied but should not have been"
            );

            Ok(())
        });
    }
}
