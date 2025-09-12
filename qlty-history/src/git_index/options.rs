use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GitIndexOptions {
    pub skip_commits_without_parents: bool,
    pub skip_commits_with_duplicate_diffs: bool,
    pub skip_paths: Option<String>,
    pub skip_commits_with_messages: Option<String>,
    pub diff_size_limit: usize,
    pub threads: Option<usize>,
    pub output_dir: PathBuf,
    pub skip_commit: Vec<String>,
    pub stop_after_commit: Option<String>,
}

impl Default for GitIndexOptions {
    fn default() -> Self {
        Self {
            skip_commits_without_parents: true,
            skip_commits_with_duplicate_diffs: true,
            skip_paths: None,
            skip_commits_with_messages: None,
            diff_size_limit: 100000,
            threads: None,
            output_dir: PathBuf::from("."),
            skip_commit: Vec::new(),
            stop_after_commit: None,
        }
    }
}

impl GitIndexOptions {
    /// Compile the regex patterns from string options
    pub fn compile_regexes(&self) -> Result<(Option<Regex>, Option<Regex>)> {
        let skip_paths_regex = self
            .skip_paths
            .as_ref()
            .map(|pattern| Regex::new(pattern))
            .transpose()
            .context("Invalid skip-paths regex")?;

        let skip_messages_regex = self
            .skip_commits_with_messages
            .as_ref()
            .map(|pattern| Regex::new(pattern))
            .transpose()
            .context("Invalid skip-commits-with-messages regex")?;

        Ok((skip_paths_regex, skip_messages_regex))
    }

    /// Convert skip_commit list to HashSet for efficient lookups
    pub fn get_skip_commits_set(&self) -> HashSet<String> {
        self.skip_commit.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = GitIndexOptions::default();
        assert!(options.skip_commits_without_parents);
        assert!(options.skip_commits_with_duplicate_diffs);
        assert_eq!(options.skip_paths, None);
        assert_eq!(options.skip_commits_with_messages, None);
        assert_eq!(options.diff_size_limit, 100000);
        assert_eq!(options.threads, None);
        assert_eq!(options.output_dir, PathBuf::from("."));
        assert!(options.skip_commit.is_empty());
        assert_eq!(options.stop_after_commit, None);
    }

    #[test]
    fn test_compile_regexes_valid() {
        let options = GitIndexOptions {
            skip_paths: Some(r"\.git/.*".to_string()),
            skip_commits_with_messages: Some(r"WIP|TODO".to_string()),
            ..Default::default()
        };

        let (skip_paths_regex, skip_messages_regex) = options.compile_regexes().unwrap();

        assert!(skip_paths_regex.is_some());
        assert!(skip_messages_regex.is_some());

        let paths_regex = skip_paths_regex.unwrap();
        assert!(paths_regex.is_match(".git/config"));
        assert!(!paths_regex.is_match("src/main.rs"));

        let messages_regex = skip_messages_regex.unwrap();
        assert!(messages_regex.is_match("WIP: work in progress"));
        assert!(messages_regex.is_match("TODO: finish this"));
        assert!(!messages_regex.is_match("feat: add new feature"));
    }

    #[test]
    fn test_compile_regexes_none() {
        let options = GitIndexOptions::default();
        let (skip_paths_regex, skip_messages_regex) = options.compile_regexes().unwrap();

        assert!(skip_paths_regex.is_none());
        assert!(skip_messages_regex.is_none());
    }

    #[test]
    fn test_compile_regexes_invalid_paths() {
        let options = GitIndexOptions {
            skip_paths: Some(r"[invalid(regex".to_string()),
            ..Default::default()
        };

        let result = options.compile_regexes();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid skip-paths regex"));
    }

    #[test]
    fn test_compile_regexes_invalid_messages() {
        let options = GitIndexOptions {
            skip_commits_with_messages: Some(r"[invalid(regex".to_string()),
            ..Default::default()
        };

        let result = options.compile_regexes();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid skip-commits-with-messages regex"));
    }

    #[test]
    fn test_get_skip_commits_set_empty() {
        let options = GitIndexOptions::default();
        let set = options.get_skip_commits_set();
        assert!(set.is_empty());
    }

    #[test]
    fn test_get_skip_commits_set_with_commits() {
        let options = GitIndexOptions {
            skip_commit: vec![
                "abc123".to_string(),
                "def456".to_string(),
                "ghi789".to_string(),
            ],
            ..Default::default()
        };

        let set = options.get_skip_commits_set();
        assert_eq!(set.len(), 3);
        assert!(set.contains("abc123"));
        assert!(set.contains("def456"));
        assert!(set.contains("ghi789"));
        assert!(!set.contains("xyz000"));
    }

    #[test]
    fn test_get_skip_commits_set_with_duplicates() {
        let options = GitIndexOptions {
            skip_commit: vec![
                "abc123".to_string(),
                "abc123".to_string(),
                "def456".to_string(),
            ],
            ..Default::default()
        };

        let set = options.get_skip_commits_set();
        assert_eq!(set.len(), 2);
        assert!(set.contains("abc123"));
        assert!(set.contains("def456"));
    }
}
