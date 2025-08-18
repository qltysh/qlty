use anyhow::Result;
use qlty_config::{
    config::{PackageFileCandidate, PluginDef},
    Workspace,
};
use std::{
    collections::HashSet,
    fs::{self, create_dir_all},
    path::PathBuf,
};

#[derive(Debug, Clone, Default)]
pub struct PluginEnabler {
    workspace: Workspace,
}

#[derive(Debug, Clone, Default)]
pub struct EnabledPluginResult {
    pub package_file: Option<String>,
    pub package_filters: Vec<String>,
    pub prefix: Option<String>,
    pub version: String,
}

impl PluginEnabler {
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }

    pub fn enable_plugin(
        &self,
        plugin_name: &str,
        plugin_def: &PluginDef,
    ) -> Result<EnabledPluginResult> {
        let mut result = EnabledPluginResult {
            version: String::new(), // Don't set a default version here - let the caller decide
            ..Default::default()
        };

        // Check for package files and filters if the plugin has package candidates
        if let Some(package_file_candidate) = plugin_def.package_file_candidate {
            let (package_file, package_filters, prefix, version) =
                self.scan_for_package_files(plugin_name, plugin_def, package_file_candidate)?;

            if !package_filters.is_empty() {
                result.package_file = package_file;
                result.package_filters = package_filters;
                result.prefix = prefix;
                result.version = version; // Only set version if extracted from lockfile
            }
        }

        Ok(result)
    }

    fn scan_for_package_files(
        &self,
        plugin_name: &str,
        plugin_def: &PluginDef,
        package_file_candidate: PackageFileCandidate,
    ) -> Result<(Option<String>, Vec<String>, Option<String>, String)> {
        let package_file_candidate_filters = if plugin_def.package_file_candidate_filters.is_empty()
        {
            vec![plugin_name.to_owned()]
        } else {
            plugin_def.package_file_candidate_filters.clone()
        };

        let repo = self.workspace.repo()?;
        let mut all_package_filters = vec![];
        let mut package_file = None;
        let mut prefixes = HashSet::new();
        let mut version = String::new();

        // Scan through the repository index looking for package files
        for entry in repo.index()?.iter() {
            let path_osstr = std::ffi::CString::new(&entry.path[..]).unwrap();
            let path_str = path_osstr.to_str().unwrap();
            let path_buf = PathBuf::from(path_str);
            let file_name = path_buf.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Check if this is the right type of package file
            if package_file_candidate.to_string() == file_name {
                if let Ok(package_file_contents) =
                    fs::read_to_string(self.workspace.root.join(path_str))
                {
                    let found_filters = match package_file_candidate {
                        PackageFileCandidate::PackageJson => self.check_package_json(
                            &package_file_contents,
                            &package_file_candidate_filters,
                        )?,
                        PackageFileCandidate::Gemfile => self.check_gemfile(
                            &package_file_contents,
                            path_str,
                            &package_file_candidate_filters,
                        )?,
                    };

                    if !found_filters.is_empty() {
                        all_package_filters.extend(found_filters);

                        // Process package file path and prefixes
                        if package_file.is_none() {
                            package_file = Some(path_str.to_owned());
                        } else {
                            // Handle multiple package files by extracting prefixes
                            if let Some(existing_path) = &package_file {
                                if prefixes.is_empty() {
                                    let existing_prefix = self.extract_prefix(existing_path);
                                    prefixes.insert(existing_prefix);
                                }
                            }

                            let new_prefix = self.extract_prefix(path_str);
                            prefixes.insert(new_prefix);

                            // Update package_file to just the filename
                            package_file = Some(file_name.to_owned());
                        }

                        // Try to extract version from lockfile if we haven't found one yet
                        if version.is_empty() {
                            version = self
                                .extract_version_from_lockfiles(
                                    &path_buf,
                                    plugin_name,
                                    package_file_candidate,
                                )
                                .unwrap_or_else(|_| String::new());
                        }
                    }
                }
            }
        }

        // Deduplicate package filters
        all_package_filters.sort();
        all_package_filters.dedup();

        let prefix = if prefixes.is_empty() {
            None
        } else if prefixes.len() == 1 && prefixes.contains("") {
            None
        } else {
            prefixes.iter().next().cloned()
        };

        Ok((package_file, all_package_filters, prefix, version))
    }

    fn check_package_json(&self, contents: &str, filters: &[String]) -> Result<Vec<String>> {
        let package_json: serde_json::Value = serde_json::from_str(contents)?;
        let mut found_filters = vec![];

        if let Some(deps) = package_json.get("dependencies") {
            if let Some(deps_obj) = deps.as_object() {
                for filter in filters {
                    if deps_obj.contains_key(filter) {
                        found_filters.push(filter.clone());
                    }
                }
            }
        }

        if let Some(dev_deps) = package_json.get("devDependencies") {
            if let Some(dev_deps_obj) = dev_deps.as_object() {
                for filter in filters {
                    if dev_deps_obj.contains_key(filter) && !found_filters.contains(filter) {
                        found_filters.push(filter.clone());
                    }
                }
            }
        }

        Ok(found_filters)
    }

    fn check_gemfile(&self, contents: &str, path: &str, filters: &[String]) -> Result<Vec<String>> {
        let mut found_filters = vec![];

        // Check Gemfile content directly for gem entries
        for filter in filters {
            let gem_pattern = format!("gem ['\"]{}['\"]", filter);
            if contents.contains(&gem_pattern) {
                found_filters.push(filter.clone());
            }
        }

        // If Gemfile contains 'gemspec', also check for .gemspec files
        if contents.contains("gemspec") {
            let path_buf = PathBuf::from(path);
            if let Some(parent) = path_buf.parent() {
                let full_parent_path = self.workspace.root.join(parent);
                if let Ok(entries) = fs::read_dir(full_parent_path) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        if let Some(ext) = entry_path.extension() {
                            if ext == "gemspec" {
                                if let Ok(gemspec_contents) = fs::read_to_string(&entry_path) {
                                    for filter in filters {
                                        if !found_filters.contains(filter) {
                                            let dependency_pattern =
                                                format!("add_dependency ['\"]{}['\"]", filter);
                                            let runtime_dep_pattern = format!(
                                                "add_runtime_dependency ['\"]{}['\"]",
                                                filter
                                            );
                                            let dev_dep_pattern = format!(
                                                "add_development_dependency ['\"]{}['\"]",
                                                filter
                                            );

                                            if gemspec_contents.contains(&dependency_pattern)
                                                || gemspec_contents.contains(&runtime_dep_pattern)
                                                || gemspec_contents.contains(&dev_dep_pattern)
                                            {
                                                found_filters.push(filter.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(found_filters)
    }

    fn extract_version_from_lockfiles(
        &self,
        package_file_path: &PathBuf,
        plugin_name: &str,
        package_file_candidate: PackageFileCandidate,
    ) -> Result<String> {
        match package_file_candidate {
            PackageFileCandidate::PackageJson => {
                let lock_file_path = package_file_path.with_file_name("package-lock.json");
                if lock_file_path.exists() {
                    let lock_contents =
                        fs::read_to_string(self.workspace.root.join(&lock_file_path))?;
                    self.extract_npm_version(&lock_contents, plugin_name)
                } else {
                    let yarn_lock_path = package_file_path.with_file_name("yarn.lock");
                    if yarn_lock_path.exists() {
                        let lock_contents =
                            fs::read_to_string(self.workspace.root.join(&yarn_lock_path))?;
                        let package_contents =
                            fs::read_to_string(&self.workspace.root.join(package_file_path))?;
                        self.extract_yarn_version(&lock_contents, &package_contents, plugin_name)
                    } else {
                        Ok(String::new())
                    }
                }
            }
            PackageFileCandidate::Gemfile => {
                let lock_file_path = package_file_path.with_file_name("Gemfile.lock");
                if lock_file_path.exists() {
                    let lock_contents =
                        fs::read_to_string(self.workspace.root.join(&lock_file_path))?;
                    self.extract_gemfile_lock_version(&lock_contents, plugin_name)
                } else {
                    Ok(String::new())
                }
            }
        }
    }

    fn extract_npm_version(&self, lock_contents: &str, plugin_name: &str) -> Result<String> {
        let lock_json: serde_json::Value = serde_json::from_str(lock_contents)?;

        if let Some(packages) = lock_json.get("packages") {
            if let Some(packages_obj) = packages.as_object() {
                let node_modules_key = format!("node_modules/{}", plugin_name);
                if let Some(package_info) = packages_obj.get(&node_modules_key) {
                    if let Some(version) = package_info.get("version") {
                        if let Some(version_str) = version.as_str() {
                            return Ok(version_str.to_string());
                        }
                    }
                }
            }
        }

        Ok(String::new())
    }

    fn extract_yarn_version(
        &self,
        _lock_contents: &str,
        _package_contents: &str,
        _plugin_name: &str,
    ) -> Result<String> {
        // Simplified yarn.lock parsing - in practice this would need more complex parsing
        // For now, return empty to fall back to latest version
        Ok(String::new())
    }

    fn extract_gemfile_lock_version(
        &self,
        lock_contents: &str,
        plugin_name: &str,
    ) -> Result<String> {
        // Look for the plugin in Gemfile.lock
        let lines: Vec<&str> = lock_contents.lines().collect();
        let mut in_specs = false;

        for line in lines {
            if line.trim() == "specs:" {
                in_specs = true;
                continue;
            }

            if in_specs && line.starts_with("    ") && line.contains(&format!("{} (", plugin_name))
            {
                // Found the gem line, extract version
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.find(')') {
                        let version = &line[start + 1..end];
                        return Ok(version.to_string());
                    }
                }
            }
        }

        Ok(String::new())
    }

    fn extract_prefix(&self, path: &str) -> String {
        let path_buf = PathBuf::from(path);
        if let Some(parent) = path_buf.parent() {
            parent.to_str().unwrap_or("").to_owned()
        } else {
            String::new()
        }
    }

    pub fn copy_configs(&self, plugin_name: &str, plugin_def: &PluginDef) -> Result<()> {
        let mut config_files = plugin_def.config_files.clone();

        plugin_def.drivers.iter().for_each(|(_, driver)| {
            config_files.extend(driver.config_files.clone());
        });

        for config_file in &config_files {
            if self.workspace.root.join(config_file).exists() {
                return Ok(()); // If any config file for the plugin already exists, skip copying
            }
        }

        for config_file in &config_files {
            for source in self.workspace.sources_list()?.sources.iter() {
                if let Some(source_file) = source.get_config_file(plugin_name, config_file)? {
                    let file_name = source_file.path.file_name().unwrap();
                    let library_configs_dir = self.workspace.library()?.configs_dir();

                    create_dir_all(&library_configs_dir)?; // Ensure the directory exists
                    let destination = library_configs_dir.join(file_name);
                    source_file.write_to(&destination)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qlty_test_utilities::git::sample_repo;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_workspace_with_package_json(package_content: &str) -> (Workspace, TempDir) {
        let (temp_dir, repo) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        // Create package.json with the plugin
        fs::write(temp_path.join("package.json"), package_content).unwrap();

        // Add the file to git index so it can be found by the enabler
        let mut index = repo.index().unwrap();
        index
            .add_path(std::path::Path::new("package.json"))
            .unwrap();
        index.write().unwrap();

        let workspace = Workspace { root: temp_path };

        (workspace, temp_dir)
    }

    #[test]
    fn test_enable_plugin_with_package_file() {
        let package_content = r#"{
  "dependencies": {
    "eslint": "^8.0.0"
  }
}"#;

        let (workspace, _temp_dir) = create_test_workspace_with_package_json(package_content);
        let enabler = PluginEnabler::new(workspace);

        let plugin_def = PluginDef {
            package_file_candidate: Some(PackageFileCandidate::PackageJson),
            package_file_candidate_filters: vec!["eslint".to_string()],
            latest_version: Some("1.0.0".to_string()),
            ..Default::default()
        };

        let result = enabler.enable_plugin("eslint", &plugin_def).unwrap();

        assert_eq!(result.package_file, Some("package.json".to_string()));
        assert_eq!(result.package_filters, vec!["eslint"]);
        assert_eq!(result.prefix, None);
        assert_eq!(result.version, ""); // Since no lockfile and package found
    }

    #[test]
    fn test_enable_plugin_without_package_file() {
        let (workspace, _temp_dir) = create_test_workspace_with_package_json("{}");
        let enabler = PluginEnabler::new(workspace);

        let plugin_def = PluginDef {
            package_file_candidate: Some(PackageFileCandidate::PackageJson),
            package_file_candidate_filters: vec!["eslint".to_string()],
            latest_version: Some("1.0.0".to_string()),
            ..Default::default()
        };

        let result = enabler.enable_plugin("eslint", &plugin_def).unwrap();

        assert_eq!(result.package_file, None);
        assert_eq!(result.package_filters, Vec::<String>::new());
        assert_eq!(result.prefix, None);
        assert_eq!(result.version, "");
    }

    #[test]
    fn test_enable_plugin_no_package_candidate() {
        let (workspace, _temp_dir) = create_test_workspace_with_package_json("{}");
        let enabler = PluginEnabler::new(workspace);

        let plugin_def = PluginDef {
            package_file_candidate: None,
            latest_version: Some("1.0.0".to_string()),
            ..Default::default()
        };

        let result = enabler.enable_plugin("clippy", &plugin_def).unwrap();

        assert_eq!(result.package_file, None);
        assert_eq!(result.package_filters, Vec::<String>::new());
        assert_eq!(result.prefix, None);
        assert_eq!(result.version, "");
    }
}
