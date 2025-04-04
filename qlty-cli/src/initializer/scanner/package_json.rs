use super::PluginInitializer;
use anyhow::{bail, Result};
use serde_json::{Map, Value};
use tracing::warn;

#[derive(Debug)]
pub struct NodePackageFile {}

impl NodePackageFile {
    pub fn related_packages(
        package_file_contents: &str,
        plugin_initializer: &PluginInitializer,
    ) -> Result<Vec<String>> {
        let mut package_filters = vec![];
        let package_file_contents = serde_json::from_str::<Value>(package_file_contents)?;

        package_filters.extend(NodePackageFile::related_dependencies(
            &package_file_contents["dependencies"],
            plugin_initializer,
        ));

        package_filters.extend(NodePackageFile::related_dependencies(
            &package_file_contents["devDependencies"],
            plugin_initializer,
        ));

        Ok(package_filters)
    }

    fn related_dependencies(
        dependencies: &Value,
        plugin_initializer: &PluginInitializer,
    ) -> Vec<String> {
        let mut package_filters = vec![];

        if let Some(deps) = dependencies.as_object() {
            for package_file_candidate_filter in &plugin_initializer.package_file_candidate_filters
            {
                if deps.iter().any(|(dep_name, _)| {
                    dep_name != &plugin_initializer.plugin_name
                        && dep_name.contains(package_file_candidate_filter)
                }) {
                    package_filters.push(package_file_candidate_filter.clone());
                }
            }
        }

        package_filters
    }

    pub fn extract_version_from_package_json(
        lock_file_contents: &str,
        plugin_name: &str,
    ) -> Result<String> {
        let lock_file_contents = serde_json::from_str::<Value>(lock_file_contents)?;

        if let Some(lock_file_data) = lock_file_contents.as_object() {
            let packages = lock_file_data.get("packages");

            if let Some(packages) = packages {
                let package_name = format!("node_modules/{}", plugin_name);

                if let Some(package) = packages.get(&package_name) {
                    if let Some(version) = package.get("version") {
                        return Ok(version.as_str().unwrap().to_owned());
                    }
                }
            } else if let Some(dependencies) = lock_file_data.get("dependencies") {
                if let Some(dependency) = dependencies.get(plugin_name) {
                    if let Some(version) = dependency.get("version") {
                        return Ok(version.as_str().unwrap().to_owned());
                    }
                }
            }
        }

        warn!(
            "No version found in package lock file for plugin: {}",
            plugin_name
        );
        bail!("No version found in package lock file");
    }

    pub fn extract_version_from_yarn_lock(
        lock_file_contents: &str,
        package_file_contents: &str,
        plugin_name: &str,
    ) -> Result<String> {
        let package_file_data = serde_json::from_str::<Value>(package_file_contents)?;
        let package_file_data = if let Some(package_data) = package_file_data.as_object() {
            package_data
        } else {
            bail!("Invalid package file data");
        };

        let package_file_version = Self::package_file_version(plugin_name, package_file_data)?;

        let lock_file_search = format!("{}@{}", plugin_name, package_file_version);

        let mut version_on_next_line = false;

        for line in lock_file_contents.lines() {
            let mut tokens = line.split_whitespace();

            if version_on_next_line {
                tokens.next(); // first token is just "version"

                if let Some(version) = tokens.next() {
                    return Ok(version.replace('"', ""));
                }
            }

            let potential_package_name = tokens.next().unwrap_or_default().replace([':', '"'], "");
            if potential_package_name == lock_file_search {
                version_on_next_line = true;
            }
        }

        bail!("No version found in yarn lock file");
    }

    fn package_file_version(
        plugin_name: &str,
        package_file_data: &Map<String, Value>,
    ) -> Result<String> {
        if let Some(version) =
            Self::get_version_from_field("dependencies", plugin_name, package_file_data).or_else(
                || Self::get_version_from_field("devDependencies", plugin_name, package_file_data),
            )
        {
            Ok(version)
        } else {
            bail!(
                "No version found in package file for plugin: {}",
                plugin_name
            );
        }
    }

    fn get_version_from_field(
        field: &str,
        plugin_name: &str,
        package_file_data: &Map<String, Value>,
    ) -> Option<String> {
        package_file_data
            .get(field)
            .and_then(|dependencies| dependencies.get(plugin_name))
            .and_then(|version| version.as_str().map(|s| s.to_string()))
    }

    pub fn is_package_json(package_file_contents: &str, plugin_name: &str) -> bool {
        let package_file_contents = serde_json::from_str::<Value>(package_file_contents);
        if let Ok(package_file_contents) = package_file_contents {
            return Self::contains_dependency(&package_file_contents["dependencies"], plugin_name)
                || Self::contains_dependency(
                    &package_file_contents["devDependencies"],
                    plugin_name,
                );
        }

        false
    }

    fn contains_dependency(all_dependencies: &Value, search: &str) -> bool {
        if let Some(deps) = all_dependencies.as_object() {
            return deps.iter().any(|(dep_name, _)| dep_name == search);
        }

        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_related_packages() {
        let package_file_contents = r#"
{
  "dependencies": {
    "eslint-plugin": "4.2.0",
    "eslint": "8.1.0",
    "some_other_package": "1.0.0"
  }
}
        "#
        .to_owned();

        let plugin_name = "eslint";
        let related_packages = NodePackageFile::related_packages(
            &package_file_contents,
            &PluginInitializer {
                plugin_name: plugin_name.to_owned(),
                package_file_candidate_filters: vec![plugin_name.to_owned()],
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(related_packages, vec!["eslint".to_owned()]);
    }

    #[test]
    fn test_related_packages_when_none() {
        let package_file_contents = r#"
{
  "dependencies": {
    "eslint": "8.1.0",
    "some_other_package": "1.0.0"
  }
}
        "#
        .to_owned();

        let plugin_name = "eslint";
        let related_packages = NodePackageFile::related_packages(
            &package_file_contents,
            &PluginInitializer {
                plugin_name: plugin_name.to_owned(),
                package_file_candidate_filters: vec![plugin_name.to_owned()],
                ..Default::default()
            },
        )
        .unwrap();

        assert!(related_packages.is_empty());
    }

    #[test]
    fn test_extract_version_from_package_json() {
        let lock_file_contents = r#"
        {
            "packages": {
                "node_modules/eslint": {
                    "version": "4.17.1"
                }
            }
        }
        "#
        .to_owned();

        let plugin_name = "eslint";
        let version =
            NodePackageFile::extract_version_from_package_json(&lock_file_contents, plugin_name)
                .unwrap();
        assert_eq!(version, "4.17.1");
    }

    #[test]
    fn test_extract_version_from_package_json_fail_case() {
        let lock_file_contents = r#"
        {
            "packages": {
                "node_modules/eslint": {
                    "version": "4.17.1"
                }
            }
        }
        "#
        .to_owned();

        let plugin_name = "not_in_package_json";
        assert!(NodePackageFile::extract_version_from_package_json(
            &lock_file_contents,
            plugin_name
        )
        .is_err());
    }

    #[test]
    fn test_extract_version_from_yarn_lock() {
        let lock_file_contents = r#"
# THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
# yarn lockfile v1

eslint-visitor-keys@^3.0.0, eslint-visitor-keys@^3.3.0, eslint-visitor-keys@^3.4.1:
  version "3.4.3"
  resolved "https://registry.yarnpkg.com/eslint-visitor-keys/-/eslint-visitor-keys-3.4.3.tgz#0cd72fe8550e3c2eae156a96a4dddcd1c8ac5800"
  integrity sha512-wpc+LXeiyiisxPlEkUzU6svyS1frIO3Mgxj1fdy7Pm8Ygzguax2N3Fa/D/ag1WqbOprdI+uY6wMUl8/a2G+iag==

eslint@8.1.0:
  version "8.1.0"
  resolved "https://registry.yarnpkg.com/eslint/-/eslint-8.1.0.tgz#00f1f7dbf4134f26588e6c9f2efe970760f64664"
  integrity sha512-JZvNneArGSUsluHWJ8g8MMs3CfIEzwaLx9KyH4tZ2i+R2/rPWzL8c0zg3rHdwYVpN/1sB9gqnjHwz9HoeJpGHw==
  dependencies:
    "@eslint/eslintrc" "^1.0.3"

eslint@^4.2.0:
  version "4.19.1"
  resolved "https://registry.yarnpkg.com/eslint/-/eslint-4.19.1.tgz#32d1d653e1d90408854bfb296f076ec7e186a300"
  integrity sha512-bT3/1x1EbZB7phzYu7vCr1v3ONuzDtX8WjuM9c0iYxe+cq+pwcKEoQjl7zd3RpC6YOLgnSy3cTN58M2jcoPDIQ==
  dependencies:
    ajv "^5.3.0"
        "#
        .to_owned();

        let package_file_contents = r#"
{
  "dependencies": {
    "eslint-plugin": "4.2.0",
    "eslint": "8.1.0",
    "some_other_package": "1.0.0"
  }
}
    "#
        .to_owned();

        let plugin_name = "eslint";
        assert_eq!(
            NodePackageFile::extract_version_from_yarn_lock(
                &lock_file_contents,
                &package_file_contents,
                plugin_name
            )
            .unwrap(),
            "8.1.0"
        );
    }

    #[test]
    fn test_extract_version_from_yarn_lock_fail_case() {
        let lock_file_contents = r#"
# THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
# yarn lockfile v1

eslint-visitor-keys@^3.0.0, eslint-visitor-keys@^3.3.0, eslint-visitor-keys@^3.4.1:
  version "3.4.3"
  resolved "https://registry.yarnpkg.com/eslint-visitor-keys/-/eslint-visitor-keys-3.4.3.tgz#0cd72fe8550e3c2eae156a96a4dddcd1c8ac5800"
  integrity sha512-wpc+LXeiyiisxPlEkUzU6svyS1frIO3Mgxj1fdy7Pm8Ygzguax2N3Fa/D/ag1WqbOprdI+uY6wMUl8/a2G+iag==

eslint@8.1.0:
  version "8.1.0"
  resolved "https://registry.yarnpkg.com/eslint/-/eslint-8.1.0.tgz#00f1f7dbf4134f26588e6c9f2efe970760f64664"
  integrity sha512-JZvNneArGSUsluHWJ8g8MMs3CfIEzwaLx9KyH4tZ2i+R2/rPWzL8c0zg3rHdwYVpN/1sB9gqnjHwz9HoeJpGHw==
  dependencies:
    "@eslint/eslintrc" "^1.0.3"

eslint@^4.2.0:
  version "4.19.1"
  resolved "https://registry.yarnpkg.com/eslint/-/eslint-4.19.1.tgz#32d1d653e1d90408854bfb296f076ec7e186a300"
  integrity sha512-bT3/1x1EbZB7phzYu7vCr1v3ONuzDtX8WjuM9c0iYxe+cq+pwcKEoQjl7zd3RpC6YOLgnSy3cTN58M2jcoPDIQ==
  dependencies:
    ajv "^5.3.0"
        "#
        .to_owned();

        let package_file_contents = r#"
{
  "dependencies": {
    "eslint-plugin": "4.2.0",
    "eslint": "8.1.0",
    "some_other_package": "1.0.0"
  }
}
    "#
        .to_owned();

        let plugin_name = "eslinta";
        assert!(NodePackageFile::extract_version_from_yarn_lock(
            &lock_file_contents,
            &package_file_contents,
            plugin_name
        )
        .is_err());
    }
}
