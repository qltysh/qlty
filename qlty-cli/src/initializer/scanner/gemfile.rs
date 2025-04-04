use super::PluginInitializer;
use anyhow::{bail, Result};
use std::path::PathBuf;

const GEMSPEC: &str = "gemspec";

#[derive(Debug)]
pub struct RubyPackageFile {}

impl RubyPackageFile {
    pub fn related_gems(
        package_file_contents: &str,
        plugin_initializer: &PluginInitializer,
        path: &str,
    ) -> Vec<String> {
        let mut related_gems = Self::extract_related_gems(
            package_file_contents,
            plugin_initializer,
            Self::gemfile_filter,
        );

        related_gems.extend(Self::gemspec_related_gems(
            package_file_contents,
            plugin_initializer,
            path,
        ));

        related_gems
    }

    fn extract_related_gems(
        package_file_contents: &str,
        plugin_initializer: &PluginInitializer,
        filter_fn: impl Fn(&str) -> bool,
    ) -> Vec<String> {
        let mut related_gems = vec![];
        for line in package_file_contents.lines() {
            let mut tokens = line.split_whitespace();
            if filter_fn(tokens.next().unwrap_or_default()) {
                let gem_name = tokens
                    .next()
                    .unwrap_or_default()
                    .replace(['\'', '"', ','], "");
                for package_file_candidate_filter in
                    &plugin_initializer.package_file_candidate_filters
                {
                    if gem_name != plugin_initializer.plugin_name
                        && gem_name.contains(package_file_candidate_filter)
                    {
                        related_gems.push(package_file_candidate_filter.to_owned());
                    }
                }
            }
        }

        related_gems
    }

    pub fn extract_version_from_gemfile_lock(
        gemfile_lock_contents: &str,
        plugin_name: &str,
    ) -> Result<String> {
        for line in gemfile_lock_contents.lines() {
            let mut tokens = line.split_whitespace();
            if tokens.next().unwrap_or_default() == plugin_name {
                let version = tokens.next().unwrap_or_default().replace(['(', ')'], "");
                if semver::Version::parse(&version).is_ok() {
                    return Ok(version);
                }
            }
        }

        bail!("No version found in gemfile");
    }

    pub fn is_gemfile(package_file_contents: &str, plugin_name: &str, path: &PathBuf) -> bool {
        Self::contains_gem(package_file_contents, plugin_name, Self::gemfile_filter)
            || Self::check_gemspec_file(package_file_contents, path, plugin_name)
    }

    fn contains_gem(
        package_file_contents: &str,
        search: &str,
        filter_fn: impl Fn(&str) -> bool,
    ) -> bool {
        for line in package_file_contents.lines() {
            let mut tokens = line.split_whitespace();
            if filter_fn(tokens.next().unwrap_or_default()) {
                let gem_name = tokens
                    .next()
                    .unwrap_or_default()
                    .replace(['\'', '"', ','], "");

                if gem_name == search {
                    return true;
                }
            }
        }

        false
    }

    fn is_gemspec(package_file_contents: &str, plugin_name: &str) -> bool {
        Self::contains_gem(package_file_contents, plugin_name, Self::gemspec_filter)
    }

    fn related_gems_from_gemspec(
        package_file_contents: &str,
        plugin_initializer: &PluginInitializer,
    ) -> Vec<String> {
        Self::extract_related_gems(
            package_file_contents,
            plugin_initializer,
            Self::gemspec_filter,
        )
    }

    fn gemfile_filter(token: &str) -> bool {
        token == "gem"
    }

    fn gemspec_filter(token: &str) -> bool {
        token.contains("add_") && token.contains("_dependency")
    }

    fn contains_gemspec_reference(package_file_contents: &str) -> bool {
        for line in package_file_contents.lines() {
            let mut tokens = line.split_whitespace();
            if tokens.next().unwrap_or_default() == GEMSPEC {
                return true;
            }
        }

        false
    }

    fn get_gemspec_path(gemfile_path: &PathBuf) -> Option<PathBuf> {
        let path = std::env::current_dir().unwrap().join(gemfile_path);
        if let Some(parent_path) = path.parent() {
            if let Ok(entries) = std::fs::read_dir(parent_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().unwrap_or_default() == GEMSPEC {
                        return Some(path);
                    }
                }
            }
        }

        None
    }

    fn check_gemspec_file(
        package_file_contents: &str,
        gemfile_path: &PathBuf,
        plugin_name: &str,
    ) -> bool {
        if Self::contains_gemspec_reference(package_file_contents) {
            if let Some(gemspec_path) = Self::get_gemspec_path(gemfile_path) {
                let package_file_contents = std::fs::read_to_string(gemspec_path);
                if let Ok(package_file_contents) = package_file_contents {
                    return Self::is_gemspec(&package_file_contents, plugin_name);
                }
            }
        }

        false
    }

    fn gemspec_related_gems(
        package_file_contents: &str,
        plugin_initializer: &PluginInitializer,
        gemfile_path: &str,
    ) -> Vec<String> {
        if Self::contains_gemspec_reference(package_file_contents) {
            let gemfile_path = PathBuf::from(gemfile_path);
            if let Some(gemspec_path) = Self::get_gemspec_path(&gemfile_path) {
                let package_file_contents = std::fs::read_to_string(gemspec_path);
                if let Ok(package_file_contents) = package_file_contents {
                    return Self::related_gems_from_gemspec(
                        &package_file_contents,
                        plugin_initializer,
                    );
                }
            }
        }

        vec![]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_related_gems() {
        let gemfile_contents = r#"
source "https://rubygems.org"

gem "randomplugin", "0.1.0"
gem "package-file-gemfile", "0.1.0"
gem "package-file-gemfile1", "0.1.0"
gem "rubyplugin3", "0.1.0"
        "#
        .to_owned();

        let plugin_name = "package-file-gemfile";
        let related_gems = RubyPackageFile::related_gems(
            &gemfile_contents,
            &PluginInitializer {
                plugin_name: plugin_name.to_owned(),
                package_file_candidate_filters: vec![plugin_name.to_owned()],
                ..Default::default()
            },
            "Gemfile",
        );

        assert_eq!(related_gems, vec!["package-file-gemfile".to_owned()]);
    }

    #[test]
    fn test_related_gems_when_none() {
        let gemfile_contents = r#"
source "https://rubygems.org"

gem "randomplugin", "0.1.0"
gem "package-file-gemfile", "0.1.0"
gem "rubyplugin3", "0.1.0"
        "#
        .to_owned();

        let plugin_name = "package-file-gemfile";
        let related_gems = RubyPackageFile::related_gems(
            &gemfile_contents,
            &PluginInitializer {
                plugin_name: plugin_name.to_owned(),
                package_file_candidate_filters: vec![plugin_name.to_owned()],
                ..Default::default()
            },
            "Gemfile",
        );

        assert!(related_gems.is_empty());
    }

    #[test]
    fn test_extract_version_from_gemfile() {
        let lock_file_contents = r#"
GEM
  remote: https://rubygems.org/
  specs:
    rubocop-rails (2.25.1)
      activesupport (>= 4.2.0)
      rack (>= 1.1)
      rubocop (>= 1.33.0, < 2.0)
      rubocop-ast (>= 1.31.1, < 2.0)
    rubocop (1.65.1)
      json (~> 2.3)
    rubocop-ast (1.32.1)
      parser (>= 3.3.1.0)
    ruby-progressbar (1.13.0)

PLATFORMS
  arm64-darwin-22
  ruby

DEPENDENCIES
  rubocop (~> 1.65)
  rubocop-rails (~> 2.25)

BUNDLED WITH
   2.5.3
        "#
        .to_owned();

        let plugin_name = "rubocop";
        let version =
            RubyPackageFile::extract_version_from_gemfile_lock(&lock_file_contents, plugin_name)
                .unwrap();

        assert_eq!(version, "1.65.1");
    }

    #[test]
    fn test_extract_version_from_gemfile_fail_case() {
        let lock_file_contents = r#"
GEM
  remote: https://rubygems.org/
  specs:
    rubocop-rails (2.25.1)
      activesupport (>= 4.2.0)
      rack (>= 1.1)
      rubocop (>= 1.33.0, < 2.0)
      rubocop-ast (>= 1.31.1, < 2.0)
    rubocop (1.65.1)
      json (~> 2.3)
    rubocop-ast (1.32.1)
      parser (>= 3.3.1.0)
    ruby-progressbar (1.13.0)

PLATFORMS
  arm64-darwin-22
  ruby

DEPENDENCIES
  rubocop (~> 1.65)
  rubocop-rails (~> 2.25)

BUNDLED WITH
   2.5.3
        "#
        .to_owned();

        let plugin_name = "rubocopa";
        assert!(RubyPackageFile::extract_version_from_gemfile_lock(
            &lock_file_contents,
            plugin_name
        )
        .is_err());
    }

    #[test]
    fn test_related_gems_from_gemspec() {
        let gemfile_contents = r#"
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
  s.add_dependency 'package-file-gemfile1', '0.1.0'

  s.add_development_dependency 'bundler'
  s.add_development_dependency 'randomplugin', '0.1.0'
  s.add_development_dependency 'rubyplugin3', '0.1.0'
  s.metadata = {
    'rubygems_mfa_required' => 'true'
  }
end
        "#
        .to_owned();

        let plugin_name = "package-file-gemfile";
        let related_gems = RubyPackageFile::related_gems_from_gemspec(
            &gemfile_contents,
            &PluginInitializer {
                plugin_name: plugin_name.to_owned(),
                package_file_candidate_filters: vec![plugin_name.to_owned()],
                ..Default::default()
            },
        );

        assert_eq!(related_gems, vec!["package-file-gemfile".to_owned()]);
    }
}
