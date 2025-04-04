use super::{gemfile::RubyPackageFile, package_json::NodePackageFile, PluginInitializer};
use anyhow::Result;
use qlty_config::config::PackageFileCandidate;
use std::path::PathBuf;

const PACKAGE_LOCK_JSON: &str = "package-lock.json";
const YARN_LOCK: &str = "yarn.lock";
const GEMFILE_LOCK: &str = "Gemfile.lock";

pub struct PackageFileScanner {}

impl PackageFileScanner {
    pub fn is_package_file(plugin_initializer: &PluginInitializer, path: &str) -> bool {
        if let Some(package_file_candidate) = &plugin_initializer.package_file_candidate {
            let path = PathBuf::from(path);
            let file_name = path.file_name().unwrap().to_str().unwrap();

            if package_file_candidate.to_string() == file_name {
                let package_file_contents = std::fs::read_to_string(&path);

                if let Ok(package_file_contents) = package_file_contents {
                    match package_file_candidate {
                        PackageFileCandidate::PackageJson => {
                            return NodePackageFile::is_package_json(
                                &package_file_contents,
                                &plugin_initializer.plugin_name,
                            );
                        }
                        PackageFileCandidate::Gemfile => {
                            return RubyPackageFile::is_gemfile(
                                &package_file_contents,
                                &plugin_initializer.plugin_name,
                                &path,
                            );
                        }
                    }
                }
            }
        }

        false
    }

    pub fn check_plugin_packages(
        plugin_initializer: &PluginInitializer,
        path: &str,
    ) -> Result<Vec<String>> {
        let mut package_filters = vec![];

        if let Some(package_file_candidate) = &plugin_initializer.package_file_candidate {
            let package_file_contents = std::fs::read_to_string(path)?;

            match package_file_candidate {
                PackageFileCandidate::PackageJson => {
                    package_filters.extend(NodePackageFile::related_packages(
                        &package_file_contents,
                        plugin_initializer,
                    )?);
                }
                PackageFileCandidate::Gemfile => {
                    package_filters.extend(RubyPackageFile::related_gems(
                        &package_file_contents,
                        plugin_initializer,
                        path,
                    ));
                }
            }
        }

        Ok(package_filters)
    }

    pub fn extract_lockfile_package_version(
        package_file_path: &PathBuf,
        plugin_name: &str,
    ) -> Result<String> {
        // This should be safe to unwrap since we know the path is a file
        let file_name = package_file_path.file_name().unwrap().to_str().unwrap();

        let package_file_candidate: PackageFileCandidate =
            serde_json::from_str(&format!("\"{}\"", file_name))?;

        match package_file_candidate {
            PackageFileCandidate::PackageJson => {
                let lock_file_path = package_file_path.with_file_name(PACKAGE_LOCK_JSON);

                if lock_file_path.exists() {
                    let lock_file_contents = std::fs::read_to_string(&lock_file_path)?;

                    NodePackageFile::extract_version_from_package_json(
                        &lock_file_contents,
                        plugin_name,
                    )
                } else {
                    let lock_file_path = package_file_path.with_file_name(YARN_LOCK);

                    let package_file_contents = std::fs::read_to_string(package_file_path)?;
                    let lock_file_contents = std::fs::read_to_string(lock_file_path)?;

                    NodePackageFile::extract_version_from_yarn_lock(
                        &lock_file_contents,
                        &package_file_contents,
                        plugin_name,
                    )
                }
            }
            PackageFileCandidate::Gemfile => {
                let lock_file_path = package_file_path.with_file_name(GEMFILE_LOCK);
                let gemfile_lock_contents = std::fs::read_to_string(lock_file_path)?;

                RubyPackageFile::extract_version_from_gemfile_lock(
                    &gemfile_lock_contents,
                    plugin_name,
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_is_package_file() {
        let plugin_initializer = PluginInitializer {
            package_file_candidate: Some(PackageFileCandidate::PackageJson),
            plugin_name: "my_plugin".to_owned(),
            ..Default::default()
        };

        let temp_dir = tempdir().unwrap();
        let package_json_path = temp_dir.path().join("package.json");
        std::fs::write(
            &package_json_path,
            r#"
{
  "dependencies": {
    "eslint-plugin": "4.2.0",
    "eslint": "8.1.0",
    "my_plugin": "1.0.0"
  }
}
            "#,
        )
        .unwrap();

        assert!(PackageFileScanner::is_package_file(
            &plugin_initializer,
            package_json_path.to_str().unwrap()
        ));
        assert!(!PackageFileScanner::is_package_file(
            &plugin_initializer,
            "Gemfile"
        ));
    }

    #[test]
    fn test_is_package_file_gemspec() {
        let plugin_initializer = PluginInitializer {
            package_file_candidate: Some(PackageFileCandidate::Gemfile),
            plugin_name: "my_plugin".to_owned(),
            ..Default::default()
        };

        let temp_dir = tempdir().unwrap();
        let gemfile_path = temp_dir.path().join("Gemfile");
        std::fs::write(
            &gemfile_path,
            r#"
            gemspec
            "#,
        )
        .unwrap();

        let gemspec_path = temp_dir.path().join("any.gemspec");
        std::fs::write(
            &gemspec_path,
            r#"
# frozen_string_literal: true
$:.push File.expand_path('lib', __dir__)

# Maintain your gem's version:
require 'random/auth/version'

# Describe your gem and declare its dependencies:
Gem::Specification.new do |s|
  s.name        = 'random'
  s.version     = Random::Auth::VERSION
  s.authors     = ['random Team']
  s.email       = ['dev@random.com']
  s.summary     = 'random'
  s.license     = 'random'

  s.files = Dir['{app,config,db,lib}/**/*', 'LICENSE', 'Rakefile', 'README.md']
  s.test_files = Dir['spec/**/*']

  s.add_dependency 'activerecord-session_store'
  s.add_dependency 'attr_encrypted'
  s.add_dependency 'rails', '~> 6.1'
  s.add_dependency 'my_plugin', '0.1.0'

  s.add_development_dependency 'bundler'
  s.add_development_dependency 'randomplugin', '0.1.0'
  s.add_development_dependency 'rubyplugin3', '0.1.0'
  s.metadata = {
    'rubygems_mfa_required' => 'true'
  }
end
            "#,
        )
        .unwrap();

        assert!(PackageFileScanner::is_package_file(
            &plugin_initializer,
            gemfile_path.to_str().unwrap()
        ));

        std::fs::write(
            &gemspec_path,
            r#"
# frozen_string_literal: true
$:.push File.expand_path('lib', __dir__)

# Maintain your gem's version:
require 'random/auth/version'

# Describe your gem and declare its dependencies:
Gem::Specification.new do |s|
  s.name        = 'random'
  s.version     = Random::Auth::VERSION
  s.authors     = ['random Team']
  s.email       = ['dev@random.com']
  s.summary     = 'random'
  s.license     = 'random'

  s.files = Dir['{app,config,db,lib}/**/*', 'LICENSE', 'Rakefile', 'README.md']
  s.test_files = Dir['spec/**/*']

  s.add_dependency 'activerecord-session_store'
  s.add_dependency 'attr_encrypted'
  s.add_dependency 'rails', '~> 6.1'

  s.add_development_dependency 'bundler'
  s.add_development_dependency 'randomplugin', '0.1.0'
  s.add_development_dependency 'my_plugin', '0.1.0'
  s.metadata = {
    'rubygems_mfa_required' => 'true'
  }
end
            "#,
        )
        .unwrap();

        assert!(PackageFileScanner::is_package_file(
            &plugin_initializer,
            gemfile_path.to_str().unwrap()
        ));
    }

    #[test]
    fn test_check_plugin_packages() {
        let temp_dir = tempdir().unwrap();
        let package_json_path = temp_dir.path().join("package.json");
        std::fs::write(
            &package_json_path,
            r#"
{
  "dependencies": {
    "eslint-plugin": "4.2.0",
    "eslint": "8.1.0",
    "some_other_package": "1.0.0"
  }
}
            "#,
        )
        .unwrap();

        let plugin_name = "eslint".to_string();
        let plugin_initializer = PluginInitializer {
            package_file_candidate: Some(PackageFileCandidate::PackageJson),
            plugin_name: plugin_name.to_owned(),
            package_file_candidate_filters: vec![plugin_name.to_owned()],
            ..Default::default()
        };

        let package_filters = PackageFileScanner::check_plugin_packages(
            &plugin_initializer,
            package_json_path.to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(package_filters, vec![plugin_name.to_owned()]);
    }

    #[test]
    fn test_check_plugin_packages_gemspec() {
        let temp_dir = tempdir().unwrap();
        let gemfile_path = temp_dir.path().join("Gemfile");
        std::fs::write(
            &gemfile_path,
            r#"
            gemspec
            "#,
        )
        .unwrap();

        let gemspec_path = temp_dir.path().join("any.gemspec");
        std::fs::write(
            &gemspec_path,
            r#"
# frozen_string_literal: true
$:.push File.expand_path('lib', __dir__)

# Maintain your gem's version:
require 'random/auth/version'

# Describe your gem and declare its dependencies:
Gem::Specification.new do |s|
  s.name        = 'random'
  s.version     = Random::Auth::VERSION
  s.authors     = ['random Team']
  s.email       = ['dev@random.com']
  s.summary     = 'random'
  s.license     = 'random'

  s.files = Dir['{app,config,db,lib}/**/*', 'LICENSE', 'Rakefile', 'README.md']
  s.test_files = Dir['spec/**/*']

  s.add_dependency 'activerecord-session_store'
  s.add_dependency 'attr_encrypted'
  s.add_dependency 'rails', '~> 6.1'
  s.add_dependency 'second_filter', '0.1.0'

  s.add_runtime_dependency 'third_filter', '0.1.0'

  s.add_development_dependency 'bundler'
  s.add_development_dependency 'randomplugin', '0.1.0'
  s.add_development_dependency 'my_plugin_package', '0.1.0'
  s.metadata = {
    'rubygems_mfa_required' => 'true'
  }
end
            "#,
        )
        .unwrap();

        let plugin_name = "my_plugin".to_string();
        let package_file_candidate_filters = vec![
            "second_filter".to_string(),
            "third_filter".to_string(),
            "my_plugin".to_string(),
            "not_found".to_string(),
        ];
        let plugin_initializer = PluginInitializer {
            package_file_candidate: Some(PackageFileCandidate::Gemfile),
            plugin_name: plugin_name.to_owned(),
            package_file_candidate_filters,
            ..Default::default()
        };

        let package_filters = PackageFileScanner::check_plugin_packages(
            &plugin_initializer,
            gemfile_path.to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(
            package_filters,
            vec![
                "second_filter".to_string(),
                "third_filter".to_string(),
                "my_plugin".to_string()
            ]
        );
    }

    #[test]
    fn test_extract_lockfile_package_version() {
        let temp_dir = tempdir().unwrap();
        let package_file_contents = r#"
{
  "dependencies": {
    "eslint-plugin": "4.2.0",
    "eslint": "8.1.0",
    "some_other_package": "1.0.0"
  }
}
        "#;

        let package_json_path = temp_dir.path().join("package.json");
        std::fs::write(&package_json_path, package_file_contents).unwrap();

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

        let package_json_lock_path = temp_dir.path().join("package-lock.json");
        std::fs::write(&package_json_lock_path, lock_file_contents).unwrap();

        let plugin_name = "eslint".to_string();
        let version =
            PackageFileScanner::extract_lockfile_package_version(&package_json_path, &plugin_name)
                .unwrap();
        assert_eq!(version, "4.17.1");
    }
}
