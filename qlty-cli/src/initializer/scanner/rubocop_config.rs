use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use serde_yaml::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Parse RuboCop configuration files to extract custom cop file paths
pub fn extract_custom_cop_files(
    workspace_root: &Path,
    config_patterns: &[std::path::PathBuf],
) -> Result<Vec<String>> {
    let mut custom_cop_files = HashSet::new();

    // Build a single GlobSet from all patterns
    let mut builder = GlobSetBuilder::new();
    for pattern in config_patterns {
        let pattern_str = pattern.to_string_lossy();
        if let Ok(glob) = Glob::new(&pattern_str) {
            builder.add(glob);
        }
    }
    let globset = builder.build()?;

    // Scan directory once and check all files against the globset
    for entry in fs::read_dir(workspace_root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if globset.is_match(file_name) {
                    parse_rubocop_config(&path, &mut custom_cop_files);
                }
            }
        }
    }

    // Convert paths to just filenames
    Ok(custom_cop_files
        .into_iter()
        .filter_map(|path| {
            Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
        })
        .collect())
}

fn parse_rubocop_config(config_path: &Path, custom_cop_files: &mut HashSet<String>) {
    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(yaml) = serde_yaml::from_str::<Value>(&content) {
            extract_requires_from_yaml(&yaml, custom_cop_files);
        }
    }
}

fn extract_requires_from_yaml(yaml: &Value, custom_cop_files: &mut HashSet<String>) {
    // Look for "require" key at the top level
    if let Value::Mapping(map) = yaml {
        if let Some(Value::Sequence(requires)) = map.get(Value::String("require".to_string())) {
            for require in requires {
                if let Value::String(path) = require {
                    // Only include files that look like local custom cops
                    // (starting with ./ or containing path separators)
                    if path.starts_with("./") || path.contains('/') {
                        // Remove leading ./ if present
                        let clean_path = path.strip_prefix("./").unwrap_or(path);
                        custom_cop_files.insert(clean_path.to_string());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_custom_cop_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let workspace_root = temp_dir.path();

        // Create a .rubocop.yml file with custom cop requires
        let rubocop_config = r#"
require:
  - ./lib/rubocop/cop/custom/my_custom_cop.rb
  - ./lib/rubocop/cop/custom/another_cop.rb
  - rubocop-rails

AllCops:
  TargetRubyVersion: 3.0
"#;
        fs::write(workspace_root.join(".rubocop.yml"), rubocop_config)?;

        // Create a .rubocop_todo.yml file
        let rubocop_todo_config = r#"
require:
  - ./lib/rubocop/cop/custom/todo_cop.rb
"#;
        fs::write(
            workspace_root.join(".rubocop_todo.yml"),
            rubocop_todo_config,
        )?;

        let config_patterns = vec![
            std::path::PathBuf::from(".rubocop.yml"),
            std::path::PathBuf::from(".rubocop_*.yml"),
            std::path::PathBuf::from(".rubocop-*.yml"),
        ];

        let custom_cops = extract_custom_cop_files(workspace_root, &config_patterns)?;

        // Should extract only the local custom cop files (not rubocop-rails)
        assert_eq!(custom_cops.len(), 3);
        assert!(custom_cops.contains(&"my_custom_cop.rb".to_string()));
        assert!(custom_cops.contains(&"another_cop.rb".to_string()));
        assert!(custom_cops.contains(&"todo_cop.rb".to_string()));

        Ok(())
    }

    #[test]
    fn test_extract_custom_cop_files_no_requires() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let workspace_root = temp_dir.path();

        // Create a .rubocop.yml file without requires
        let rubocop_config = r#"
AllCops:
  TargetRubyVersion: 3.0
"#;
        fs::write(workspace_root.join(".rubocop.yml"), rubocop_config)?;

        let config_patterns = vec![std::path::PathBuf::from(".rubocop.yml")];

        let custom_cops = extract_custom_cop_files(workspace_root, &config_patterns)?;

        assert_eq!(custom_cops.len(), 0);

        Ok(())
    }
}
