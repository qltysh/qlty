mod driver_candidate;
mod driver_initializer;
mod gemfile;
mod package_file;
mod package_json;

use super::{Renderer, Settings, SourceSpec};
use anyhow::Result;
use driver_candidate::DriverCandidate;
use driver_initializer::{ConfigDriver, DriverInitializer, TargetDriver};
use indicatif::ProgressBar;
use itertools::Itertools;
use package_file::PackageFileScanner;
use qlty_config::{
    config::{Builder, Ignore, PackageFileCandidate, PluginDef, SuggestionMode},
    sources::{SourceFetch, SourceFile},
    QltyConfig,
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

type Name = String;
type FilesCount = u32;

#[derive(Debug, Clone, Default)]
pub struct Scanner {
    pub settings: Settings,
    pub default_config: QltyConfig,
    pub source_specs: Vec<SourceSpec>,
    pub source_list: Box<dyn SourceFetch>,
    pub plugins: Vec<InstalledPlugin>,
    pub sources_only_config: QltyConfig,

    default_ignores: Vec<Ignore>,
    plugin_initializers: Vec<PluginInitializer>,
    plugins_to_activate: HashMap<Name, PluginToActivate>,
}

#[derive(Debug, Clone, Default)]
struct PluginToActivate {
    package_file: Option<String>,
    package_filters: Vec<String>,
    file_count: FilesCount,
    drivers: HashMap<Name, Box<dyn DriverInitializer>>,
    prefixes: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct InstalledPlugin {
    pub name: String,
    pub version: String,
    pub files_count: FilesCount,
    pub config_files: Vec<SourceFile>,
    pub enabled_drivers: Vec<String>,
    pub package_file: Option<String>,
    pub package_filters: Vec<String>,
    pub prefix: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct PluginInitializer {
    plugin_name: String,
    package_file_candidate: Option<PackageFileCandidate>,
    package_file_candidate_filters: Vec<String>,
    driver_initializers: Vec<Box<dyn DriverInitializer>>,
}

impl Scanner {
    pub fn new(
        settings: Settings,
        source_specs: &[SourceSpec],
        source_list: Box<dyn SourceFetch>,
    ) -> Result<Self> {
        Ok(Self {
            settings,
            source_specs: source_specs.to_vec(),
            source_list,
            default_config: Builder::default_config().unwrap(),
            plugins: vec![],
            default_ignores: vec![],
            plugin_initializers: vec![],
            plugins_to_activate: HashMap::new(),
            ..Default::default()
        })
    }

    pub fn prepare(&mut self) -> Result<()> {
        self.sources_only_config = self.sources_only_config()?;

        Ok(())
    }

    pub fn scan(&mut self, progress: &ProgressBar) -> Result<()> {
        self.compute_default_ignores();
        self.compute_plugin_initializers()?;
        self.compute_plugins_to_enable(progress)?;
        self.compute_plugin_details()?;
        Ok(())
    }

    fn compute_plugins_to_enable(&mut self, progress: &ProgressBar) -> Result<()> {
        let repo = self.settings.workspace.repo()?;

        for (count, entry) in repo.index()?.iter().enumerate() {
            let path_osstr = std::ffi::CString::new(&entry.path[..]).unwrap();
            let path_osstr = path_osstr.to_str().unwrap();

            if self
                .default_ignores
                .iter()
                .any(|ignore| ignore.matches_path(path_osstr))
            {
                continue;
            }

            for plugin_initializer in &self.plugin_initializers {
                let is_package_file =
                    PackageFileScanner::is_package_file(plugin_initializer, path_osstr);

                if is_package_file {
                    let package_filters =
                        PackageFileScanner::check_plugin_packages(plugin_initializer, path_osstr)?;

                    Self::insert_package_filters_and_package_file(
                        &mut self.plugins_to_activate,
                        package_filters,
                        plugin_initializer,
                        path_osstr,
                    );
                }

                for driver_initializer in &plugin_initializer.driver_initializers {
                    if driver_initializer.is_enabler(path_osstr) {
                        self.plugins_to_activate
                            .entry(plugin_initializer.plugin_name.to_owned())
                            .or_default()
                            .drivers
                            .insert(driver_initializer.key(), driver_initializer.clone_box());
                    }
                }

                if plugin_initializer.matches_workspace_entry(path_osstr) {
                    let entry: &mut u32 = &mut self
                        .plugins_to_activate
                        .entry(plugin_initializer.plugin_name.to_owned())
                        .or_default()
                        .file_count;

                    *entry += 1;
                }
            }

            progress.set_message(format!("{} files", count));
            progress.tick();
        }

        Ok(())
    }

    fn compute_plugin_details(&mut self) -> Result<()> {
        for (plugin_name, plugin_to_activate) in &self.plugins_to_activate {
            let drivers_to_activate = &plugin_to_activate.drivers;
            if drivers_to_activate.is_empty() {
                continue;
            }

            let plugin = self
                .sources_only_config
                .plugins
                .definitions
                .get(plugin_name)
                .cloned()
                .unwrap();

            let mut configs_to_install = vec![];
            let mut config_files = plugin.config_files.clone();

            plugin.drivers.iter().for_each(|(driver_name, driver)| {
                if drivers_to_activate.contains_key(driver_name) {
                    config_files.extend(driver.config_files.clone());
                }
            });

            for config_file in &config_files {
                for source in self.source_list.sources().iter() {
                    if self.settings.workspace.root.join(config_file).exists() {
                        continue;
                    }

                    if let Some(source_file) = source.get_config_file(plugin_name, config_file)? {
                        configs_to_install.push(source_file);
                    }
                }
            }

            let unique_drivers = drivers_to_activate
                .iter()
                .map(|(_, driver)| driver.driver_name())
                .unique()
                .collect::<Vec<String>>();

            let enabled_drivers = if unique_drivers.len() == plugin.drivers.keys().len() {
                vec![]
            } else {
                drivers_to_activate
                    .iter()
                    .map(|(_, driver)| driver.driver_name())
                    .collect()
            };

            let mut driver_versions = drivers_to_activate
                .iter()
                .map(|(_, driver)| (driver.version()))
                .unique()
                .collect::<Vec<String>>();

            driver_versions.sort();

            let package_filters: Vec<String> = plugin_to_activate
                .package_filters
                .clone()
                .into_iter()
                .unique()
                .collect();

            let package_file = if package_filters.is_empty() {
                None
            } else {
                plugin_to_activate.package_file.clone()
            };

            let prefixes = if plugin_to_activate.prefixes.is_empty() {
                vec![None] // Use None to represent no prefix
            } else {
                plugin_to_activate
                    .prefixes
                    .iter()
                    .map(Some)
                    .collect::<Vec<_>>()
            };

            for prefix in prefixes {
                let version = if let Some(package_file) = &plugin_to_activate.package_file {
                    let package_file_path = match prefix {
                        Some(prefix) => PathBuf::from(prefix).join(package_file),
                        None => PathBuf::from(package_file),
                    };
                    PackageFileScanner::extract_lockfile_package_version(
                        &package_file_path,
                        plugin_name,
                    )
                    .unwrap_or_else(|_| driver_versions.last().unwrap().clone())
                } else {
                    driver_versions.last().unwrap().clone()
                };

                self.plugins.push(InstalledPlugin {
                    name: plugin_name.to_owned(),
                    version,
                    files_count: plugin_to_activate.file_count,
                    config_files: configs_to_install.clone(),
                    enabled_drivers: enabled_drivers.clone(),
                    package_file: package_file.clone(),
                    package_filters: package_filters.clone(),
                    prefix: prefix.cloned(),
                });
            }
        }

        Ok(())
    }

    fn compute_plugin_initializers(&mut self) -> Result<()> {
        let mut plugin_initializers = self
            .sources_only_config
            .plugins
            .definitions
            .iter()
            .map(|(plugin_name, plugin_definition)| {
                self.build_plugin_initializer(plugin_name, plugin_definition)
            })
            .collect::<Result<Vec<Option<PluginInitializer>>>>()?;

        plugin_initializers.retain(|x| x.is_some());

        self.plugin_initializers = plugin_initializers
            .into_iter()
            .map(|x| x.unwrap())
            .collect();

        Ok(())
    }

    fn build_plugin_initializer(
        &self,
        plugin_name: &str,
        plugin_def: &PluginDef,
    ) -> Result<Option<PluginInitializer>> {
        let package_file_candidate_filters = if plugin_def.package_file_candidate_filters.is_empty()
        {
            vec![plugin_name.to_owned()]
        } else {
            plugin_def.package_file_candidate_filters.clone()
        };

        let mut plugin_initializer = PluginInitializer {
            plugin_name: plugin_name.to_owned(),
            package_file_candidate: plugin_def.package_file_candidate,
            package_file_candidate_filters,
            ..Default::default()
        };

        for candidate in DriverCandidate::build_driver_candidates(plugin_def) {
            let driver_initializer: Box<dyn DriverInitializer> = match &candidate.driver.suggested {
                SuggestionMode::Targets => {
                    Box::new(TargetDriver::new(candidate, plugin_def, self)?)
                }
                SuggestionMode::Config => Box::new(ConfigDriver::new(candidate, plugin_def, self)?),
                SuggestionMode::Never => continue,
            };

            plugin_initializer
                .driver_initializers
                .push(driver_initializer);
        }

        if !plugin_initializer.driver_initializers.is_empty() {
            Ok(Some(plugin_initializer))
        } else {
            Ok(None)
        }
    }

    fn compute_default_ignores(&mut self) {
        self.default_ignores = self.default_config.ignore.clone();
        self.default_ignores
            .iter()
            .for_each(|ignore| ignore.initialize_globset());
    }

    fn sources_only_config(&self) -> Result<QltyConfig> {
        self.source_list.fetch()?;

        Builder::full_config_from_toml_str(
            &Renderer::new(&self.source_specs, &[]).render()?,
            &self.settings.workspace.library()?,
        )
    }

    fn insert_package_filters_and_package_file(
        plugins_to_activate: &mut HashMap<Name, PluginToActivate>,
        package_filters: Vec<String>,
        plugin_initializer: &PluginInitializer,
        path: &str,
    ) {
        if let Some(plugin_to_activate) =
            plugins_to_activate.get_mut(&plugin_initializer.plugin_name)
        {
            plugin_to_activate
                .package_filters
                .extend(package_filters.clone());

            if let Some(package_file) = &plugin_to_activate.package_file {
                plugin_to_activate.package_file = Self::process_plugin_package_file_prefix(
                    package_file,
                    &mut plugin_to_activate.prefixes,
                );

                Self::process_plugin_package_file_prefix(path, &mut plugin_to_activate.prefixes);
            } else {
                plugin_to_activate.package_file = Some(path.to_owned());
            }
        } else {
            plugins_to_activate.insert(
                plugin_initializer.plugin_name.to_owned(),
                PluginToActivate {
                    package_filters,
                    package_file: Some(path.to_owned()),
                    ..Default::default()
                },
            );
        }
    }

    fn process_plugin_package_file_prefix(
        path: &str,
        prefix_set: &mut HashSet<String>,
    ) -> Option<String> {
        let path_buf = PathBuf::from(path);
        let prefix = path_buf.parent().unwrap();
        let prefix_str = prefix.to_str().unwrap().to_owned();

        prefix_set.insert(prefix_str.clone());

        Some(path_buf.file_name().unwrap().to_str().unwrap().to_owned())
    }
}

impl PluginInitializer {
    pub fn matches_workspace_entry(&self, path: &str) -> bool {
        self.driver_initializers
            .iter()
            .any(|driver| driver.is_workspace_entry(path))
    }
}

#[cfg(test)]
mod test {
    use qlty_analysis::utils::fs::path_to_native_string;
    use qlty_config::{
        sources::{LocalSource, Source},
        Workspace,
    };
    use qlty_test_utilities::git::sample_repo;
    use std::fs::{self, File};
    use tempfile::TempDir;

    use super::*;

    #[derive(Default, Clone, Debug)]
    struct MockSourceList {
        temp_path: PathBuf,
    }

    impl SourceFetch for MockSourceList {
        fn fetch(&self) -> Result<()> {
            fs::create_dir_all(&self.temp_path.join(path_to_native_string(
                ".qlty/sources/https---testing-com/test_branch/linters/exists",
            )))
            .unwrap();

            let path = self.temp_path.join(path_to_native_string(
                ".qlty/sources/https---testing-com/test_branch/linters/exists/plugin.toml",
            ));

            fs::write(
                &path,
                r#"
config_version = "0"

[plugins.definitions.exists]
file_types = ["ALL"]
latest_version = "1.0.0"
known_good_version = "1.0.0"

[plugins.definitions.exists.drivers.lint]
prepare_script = "mkdir ${linter} && echo dir %2 > ${linter}/ls.cmd || echo dir %2 > ${linter}/ls.cmd"
script = "ls -l ${target}"
success_codes = [0]
output = "pass_fail"
suggested = "targets"
config_files = ["config.toml"]
                "#,
            )
            .unwrap();

            Ok(())
        }

        fn clone_box(&self) -> Box<dyn SourceFetch> {
            Box::new(self.clone())
        }

        fn sources(&self) -> Vec<Box<dyn Source>> {
            vec![Box::new(LocalSource {
                root: self
                    .temp_path
                    .join(".qlty/sources/https---testing-com/test_branch/"),
            })]
        }
    }

    fn create_scanner() -> (Scanner, TempDir) {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        let workspace = Workspace {
            root: temp_path.clone(),
        };

        let source_spec = SourceSpec {
            name: "testing".to_string(),
            target: Some("https://testing.com".to_string()),
            reference: Some(crate::initializer::SourceRefSpec::Branch(
                "test_branch".to_string(),
            )),
            default: false,
        };

        let settings = Settings {
            workspace,
            skip_plugins: false,
            skip_default_source: true,
            source: Some(source_spec.clone()),
        };

        (
            Scanner::new(
                settings,
                &[source_spec],
                Box::new(MockSourceList {
                    temp_path: temp_path.clone(),
                }),
            )
            .unwrap(),
            temp_dir,
        )
    }

    fn update_source_suggested(scanner: &mut Scanner, suggestion: SuggestionMode) {
        let mut modified_source = scanner.sources_only_config().unwrap();
        modified_source
            .plugins
            .definitions
            .get_mut("exists")
            .unwrap()
            .drivers
            .get_mut("lint")
            .unwrap()
            .suggested = suggestion;

        scanner.sources_only_config = modified_source;
    }

    #[test]
    fn test_prepare() {
        // gotta keep td in scope otherwise it vanishes
        let (mut scanner, _td) = create_scanner();

        assert_eq!(scanner.sources_only_config.source.len(), 0);
        assert!(scanner.prepare().is_ok());
        assert_eq!(scanner.sources_only_config.source.len(), 1);

        let scanner_source = scanner.sources_only_config.source.get(0).unwrap();
        assert_eq!(scanner_source.name, Some("testing".to_string()));
        assert_eq!(scanner_source.branch, Some("test_branch".to_string()));
        assert!(scanner
            .sources_only_config
            .plugins
            .definitions
            .get("exists")
            .is_some());
    }

    #[test]
    fn test_scan_when_suggested() {
        // gotta keep td in scope otherwise it vanishes
        let (mut scanner, _td) = create_scanner();

        scanner.prepare().unwrap();

        assert!(scanner.scan(&ProgressBar::hidden()).is_ok());
        assert!(scanner
            .plugins
            .iter()
            .find(|p| p.name == "exists")
            .is_some());

        let installed_plugin = scanner.plugins.iter().find(|p| p.name == "exists").unwrap();
        assert_eq!(installed_plugin.version, "latest");
        assert_eq!(installed_plugin.files_count, 7);
        assert_eq!(installed_plugin.config_files.len(), 0);
        assert_eq!(installed_plugin.package_file, None);
    }

    #[test]
    fn test_scan_suggested_config_without_config() {
        // gotta keep td in scope otherwise it vanishes
        let (mut scanner, _td) = create_scanner();

        scanner.prepare().unwrap();
        update_source_suggested(&mut scanner, SuggestionMode::Config);

        assert!(scanner.scan(&ProgressBar::hidden()).is_ok());
        assert_eq!(scanner.plugins.len(), 0);
        assert!(scanner
            .plugins
            .iter()
            .find(|p| p.name == "exists")
            .is_none());
    }

    #[test]
    fn test_scan_suggested_config_with_config() {
        let (mut scanner, td) = create_scanner();
        File::create(td.path().join("config.toml")).unwrap();

        scanner.prepare().unwrap();
        update_source_suggested(&mut scanner, SuggestionMode::Config);

        assert!(scanner.scan(&ProgressBar::hidden()).is_ok());
        assert_eq!(scanner.plugins.len(), 0);
        assert!(scanner
            .plugins
            .iter()
            .find(|p| p.name == "exists")
            .is_none());
    }

    #[test]
    fn test_scan_when_not_suggested() {
        // gotta keep td in scope otherwise it vanishes
        let (mut scanner, _td) = create_scanner();

        scanner.prepare().unwrap();
        update_source_suggested(&mut scanner, SuggestionMode::Never);

        assert!(scanner.scan(&ProgressBar::hidden()).is_ok());
        assert_eq!(scanner.plugins.len(), 0);
        assert!(scanner
            .plugins
            .iter()
            .find(|p| p.name == "exists")
            .is_none());
    }

    #[test]
    fn test_insert_package_filters_and_package_file_with_prefixes() {
        let mut plugins_to_activate = HashMap::new();
        let plugin_initializer = PluginInitializer {
            plugin_name: "test".to_string(),
            package_file_candidate: None,
            package_file_candidate_filters: vec![],
            driver_initializers: vec![],
        };

        let path = "deep/package.json";

        Scanner::insert_package_filters_and_package_file(
            &mut plugins_to_activate,
            vec!["test".to_string()],
            &plugin_initializer,
            path,
        );

        assert_eq!(plugins_to_activate.len(), 1);
        assert_eq!(
            plugins_to_activate.get("test").unwrap().package_file,
            Some(path.to_string())
        );
        assert_eq!(
            plugins_to_activate.get("test").unwrap().package_filters,
            vec!["test".to_string()]
        );

        let path = "package.json";

        assert_eq!(
            plugins_to_activate.get("test").unwrap().prefixes,
            HashSet::new()
        );

        Scanner::insert_package_filters_and_package_file(
            &mut plugins_to_activate,
            vec!["test2".to_string()],
            &plugin_initializer,
            path,
        );

        assert_eq!(plugins_to_activate.len(), 1);
        assert_eq!(
            plugins_to_activate.get("test").unwrap().package_file,
            Some(path.to_string())
        );
        assert_eq!(
            plugins_to_activate.get("test").unwrap().package_filters,
            vec!["test".to_string(), "test2".to_string()]
        );

        let mut prefix_set = HashSet::new();
        prefix_set.insert("deep".to_string());
        prefix_set.insert("".to_string());

        assert_eq!(
            plugins_to_activate.get("test").unwrap().prefixes,
            prefix_set
        );
    }

    #[test]
    fn test_config_file_from_plugin_dir() {
        let (mut scanner, td) = create_scanner();
        let config_file_path = td.path().join(path_to_native_string(
            ".qlty/sources/https---testing-com/test_branch/linters/exists/config.toml",
        ));

        scanner.prepare().unwrap();
        File::create(&config_file_path).unwrap();

        assert!(scanner.scan(&ProgressBar::hidden()).is_ok());
        assert!(scanner
            .plugins
            .iter()
            .find(|p| p.name == "exists")
            .is_some());

        let installed_plugin = scanner.plugins.iter().find(|p| p.name == "exists").unwrap();
        assert_eq!(installed_plugin.version, "latest");
        assert_eq!(installed_plugin.files_count, 7);
        assert_eq!(
            installed_plugin
                .config_files
                .iter()
                .map(|f| f.path.clone())
                .collect::<Vec<_>>(),
            vec![config_file_path]
        );
        assert_eq!(installed_plugin.package_file, None);
    }

    #[test]
    fn test_config_file_from_plugin_dir_when_exists_in_repo() {
        let (mut scanner, td) = create_scanner();
        let config_file_path = td.path().join(path_to_native_string(
            ".qlty/sources/https---testing-com/test_branch/linters/exists/config.toml",
        ));

        scanner.prepare().unwrap();
        File::create(&config_file_path).unwrap();
        File::create(td.path().join("config.toml")).unwrap();

        assert!(scanner.scan(&ProgressBar::hidden()).is_ok());
        assert!(scanner
            .plugins
            .iter()
            .find(|p| p.name == "exists")
            .is_some());

        let installed_plugin = scanner.plugins.iter().find(|p| p.name == "exists").unwrap();
        assert_eq!(installed_plugin.version, "latest");
        assert_eq!(installed_plugin.files_count, 7);
        assert_eq!(installed_plugin.config_files.len(), 0);
        assert_eq!(installed_plugin.package_file, None);
    }
}
