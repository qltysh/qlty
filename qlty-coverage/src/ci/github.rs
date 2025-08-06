use crate::{
    ci::CI,
    env::{EnvSource, SystemEnv},
};
use regex::Regex;
use serde_json::Value;
use std::fs;

#[derive(Debug)]
pub struct GitHub {
    env: Box<dyn EnvSource>,
}

impl Default for GitHub {
    fn default() -> Self {
        Self {
            env: Box::<SystemEnv>::default(),
        }
    }
}

impl GitHub {
    /// Attempts to extract the pull request head SHA from the GitHub event file.
    /// Returns None if not a PR event, or if the file cannot be read/parsed.
    fn get_pr_head_sha(&self) -> Option<String> {
        let event_name = self.env.var("GITHUB_EVENT_NAME")?;

        // Only process pull request events
        if event_name != "pull_request" && event_name != "pull_request_target" {
            return None;
        }

        let event_path = self.env.var("GITHUB_EVENT_PATH")?;

        // Read the event file
        let event_data = match fs::read_to_string(&event_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to read GitHub event file '{}': {}. Falling back to GITHUB_SHA.",
                    event_path, e
                );
                return None;
            }
        };

        // Parse the JSON
        let event_json = match serde_json::from_str::<Value>(&event_data) {
            Ok(json) => json,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to parse GitHub event file '{}': {}. Falling back to GITHUB_SHA.",
                    event_path, e
                );
                return None;
            }
        };

        // Extract the head SHA from the pull request data
        event_json
            .get("pull_request")
            .and_then(|pr| pr.get("head"))
            .and_then(|head| head.get("sha"))
            .and_then(|sha| sha.as_str())
            .map(|s| s.to_string())
    }
}

impl CI for GitHub {
    fn detect(&self) -> bool {
        self.env.var("GITHUB_ACTIONS").unwrap_or_default() == "true"
    }

    fn ci_name(&self) -> String {
        "GitHub".to_string()
    }

    fn ci_url(&self) -> String {
        self.env.var("GITHUB_SERVER_URL").unwrap_or_default()
    }

    fn branch(&self) -> String {
        match self.env.var("GITHUB_REF_TYPE") {
            Some(ref_type) => {
                if ref_type == "tag" {
                    "".to_string()
                } else if let Some(ref_name) = self.env.var("GITHUB_REF_NAME") {
                    ref_name
                } else {
                    self.env.var("GITHUB_HEAD_REF").unwrap_or_default()
                }
            }
            None => "".to_string(),
        }
    }

    fn workflow(&self) -> String {
        self.env.var("GITHUB_WORKFLOW").unwrap_or_default()
    }

    fn job(&self) -> String {
        self.env.var("GITHUB_JOB").unwrap_or_default()
    }

    fn build_id(&self) -> String {
        let run_id = self.env.var("GITHUB_RUN_ID").unwrap_or_default();
        let run_attempt = self.env.var("GITHUB_RUN_ATTEMPT").unwrap_or_default();

        if !run_id.is_empty() && !run_attempt.is_empty() {
            format!("{}:{}", run_id, run_attempt)
        } else {
            run_id
        }
    }

    fn build_url(&self) -> String {
        if self.build_id() != "" {
            format!("{}/actions/runs/{}", self.repository_url(), self.build_id())
        } else {
            "".to_string()
        }
    }

    fn pull_number(&self) -> String {
        let head_ref = self.env.var("GITHUB_HEAD_REF").unwrap_or_default();
        let full_ref = self.env.var("GITHUB_REF").unwrap_or_default();
        let re = Regex::new(r"refs/pull/([0-9]+)/merge").unwrap();

        if !head_ref.is_empty() {
            match re.captures(&full_ref) {
                Some(caps) => caps[1].to_string(),
                None => "".to_string(),
            }
        } else {
            "".to_string()
        }
    }

    fn repository_name(&self) -> String {
        self.env.var("GITHUB_REPOSITORY").unwrap_or_default()
    }

    fn repository_url(&self) -> String {
        if self.repository_name() != "" {
            format!("{}/{}", self.ci_url(), self.repository_name())
        } else {
            "".to_string()
        }
    }

    fn pull_url(&self) -> String {
        if self.pull_number() != "" {
            format!("{}/pull/{}", self.repository_url(), self.pull_number())
        } else {
            "".to_string()
        }
    }

    fn commit_sha(&self) -> String {
        // For pull request events, GitHub's GITHUB_SHA is a merge commit.
        // We need to get the actual head SHA from the event data.
        // References:
        // - https://github.com/orgs/community/discussions/26325
        // - https://www.kenmuse.com/blog/the-many-shas-of-a-github-pull-request/

        self.get_pr_head_sha()
            .unwrap_or_else(|| self.env.var("GITHUB_SHA").unwrap_or_default())
    }

    fn git_tag(&self) -> Option<String> {
        if self.env.var("GITHUB_REF_TYPE")? == "tag" {
            self.env.var("GITHUB_REF_NAME")
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[derive(Debug, Clone, Default)]
    pub struct HashMapEnv {
        inner: HashMap<String, String>,
    }

    impl HashMapEnv {
        pub fn new(env: HashMap<String, String>) -> Self {
            Self { inner: env }
        }
    }

    impl EnvSource for HashMapEnv {
        fn var(&self, name: &str) -> Option<String> {
            self.inner.get(name).cloned()
        }
    }

    #[test]
    fn detect_ci() {
        let ci = GitHub {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(ci.detect(), false);

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("GITHUB_ACTIONS".to_string(), "true".to_string());
        env.insert(
            "GITHUB_SERVER_URL".to_string(),
            "https://github.com".to_string(),
        );
        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.detect(), true);
        assert_eq!(&ci.ci_name(), "GitHub");
        assert_eq!(&ci.ci_url(), "https://github.com");
    }

    #[test]
    fn repository() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "GITHUB_SERVER_URL".to_string(),
            "https://github.com".to_string(),
        );
        env.insert("GITHUB_REPOSITORY".to_string(), "qltysh/qlty".to_string());

        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.repository_name(), "qltysh/qlty");
        assert_eq!(&ci.repository_url(), "https://github.com/qltysh/qlty");
    }

    #[test]
    fn branch_build() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("GITHUB_REF_TYPE".to_string(), "branch".to_string());
        env.insert("GITHUB_REF_NAME".to_string(), "main".to_string());
        env.insert(
            "GITHUB_SHA".to_string(),
            "77948d72a8b5ea21bb335e8e674bad99413da7a2".to_string(),
        );

        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "main");
        assert_eq!(&ci.pull_number(), "");
        assert_eq!(&ci.pull_url(), "");
        assert_eq!(&ci.commit_sha(), "77948d72a8b5ea21bb335e8e674bad99413da7a2");
    }

    #[test]
    fn pull_request_build_without_event_file() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "GITHUB_SERVER_URL".to_string(),
            "https://github.com".to_string(),
        );
        env.insert("GITHUB_REPOSITORY".to_string(), "qltysh/qlty".to_string());
        env.insert("GITHUB_REF_TYPE".to_string(), "branch".to_string());
        env.insert(
            "GITHUB_HEAD_REF".to_string(),
            "feature-branch-1".to_string(),
        );
        env.insert("GITHUB_REF".to_string(), "refs/pull/42/merge".to_string());
        env.insert(
            "GITHUB_SHA".to_string(),
            "77948d72a8b5ea21bb335e8e674bad99413da7a2".to_string(),
        );

        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "feature-branch-1");
        assert_eq!(&ci.pull_number(), "42");
        assert_eq!(&ci.pull_url(), "https://github.com/qltysh/qlty/pull/42");
        assert_eq!(&ci.commit_sha(), "77948d72a8b5ea21bb335e8e674bad99413da7a2");
    }

    #[test]
    fn pull_request_build_with_event_file() {
        // Create a temporary file with PR event data
        let mut event_file = NamedTempFile::new().unwrap();
        let event_json = r#"{
            "pull_request": {
                "head": {
                    "sha": "abc123def456actual_head_sha"
                },
                "base": {
                    "sha": "base_sha_here"
                }
            }
        }"#;
        writeln!(event_file, "{}", event_json).unwrap();
        let event_path = event_file.path().to_str().unwrap().to_string();

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "GITHUB_SERVER_URL".to_string(),
            "https://github.com".to_string(),
        );
        env.insert("GITHUB_REPOSITORY".to_string(), "qltysh/qlty".to_string());
        env.insert("GITHUB_REF_TYPE".to_string(), "branch".to_string());
        env.insert(
            "GITHUB_HEAD_REF".to_string(),
            "feature-branch-1".to_string(),
        );
        env.insert("GITHUB_REF".to_string(), "refs/pull/42/merge".to_string());
        env.insert(
            "GITHUB_SHA".to_string(),
            "77948d72a8b5ea21bb335e8e674bad99413da7a2".to_string(),
        );
        env.insert("GITHUB_EVENT_NAME".to_string(), "pull_request".to_string());
        env.insert("GITHUB_EVENT_PATH".to_string(), event_path);

        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "feature-branch-1");
        assert_eq!(&ci.pull_number(), "42");
        assert_eq!(&ci.pull_url(), "https://github.com/qltysh/qlty/pull/42");
        assert_eq!(&ci.commit_sha(), "abc123def456actual_head_sha");
    }

    #[test]
    fn pull_request_target_build_with_event_file() {
        let mut event_file = NamedTempFile::new().unwrap();
        let event_json = r#"{
            "pull_request": {
                "head": {
                    "sha": "pr_target_head_sha_123"
                },
                "base": {
                    "sha": "base_sha_here"
                }
            }
        }"#;
        writeln!(event_file, "{}", event_json).unwrap();
        let event_path = event_file.path().to_str().unwrap().to_string();

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "GITHUB_SERVER_URL".to_string(),
            "https://github.com".to_string(),
        );
        env.insert("GITHUB_REPOSITORY".to_string(), "qltysh/qlty".to_string());
        env.insert("GITHUB_REF_TYPE".to_string(), "branch".to_string());
        env.insert(
            "GITHUB_HEAD_REF".to_string(),
            "feature-branch-1".to_string(),
        );
        env.insert("GITHUB_REF".to_string(), "refs/pull/42/merge".to_string());
        env.insert("GITHUB_SHA".to_string(), "merge_commit_sha_789".to_string());
        env.insert(
            "GITHUB_EVENT_NAME".to_string(),
            "pull_request_target".to_string(),
        );
        env.insert("GITHUB_EVENT_PATH".to_string(), event_path);

        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.commit_sha(), "pr_target_head_sha_123");
    }

    #[test]
    fn pull_request_build_with_invalid_event_file() {
        // Create a temporary file with invalid JSON
        let mut event_file = NamedTempFile::new().unwrap();
        writeln!(event_file, "invalid json data").unwrap();
        let event_path = event_file.path().to_str().unwrap().to_string();

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("GITHUB_SHA".to_string(), "fallback_sha_value".to_string());
        env.insert("GITHUB_EVENT_NAME".to_string(), "pull_request".to_string());
        env.insert("GITHUB_EVENT_PATH".to_string(), event_path);

        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        // Should fallback to GITHUB_SHA when event file is invalid
        assert_eq!(&ci.commit_sha(), "fallback_sha_value");
    }

    #[test]
    fn pull_request_build_with_missing_head_sha() {
        // Create a temporary file with PR event data but no head SHA
        let mut event_file = NamedTempFile::new().unwrap();
        let event_json = r#"{
            "pull_request": {
                "base": {
                    "sha": "base_sha_here"
                }
            }
        }"#;
        writeln!(event_file, "{}", event_json).unwrap();
        let event_path = event_file.path().to_str().unwrap().to_string();

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "GITHUB_SHA".to_string(),
            "fallback_sha_when_missing_head".to_string(),
        );
        env.insert("GITHUB_EVENT_NAME".to_string(), "pull_request".to_string());
        env.insert("GITHUB_EVENT_PATH".to_string(), event_path);

        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        // Should fallback to GITHUB_SHA when head SHA is missing
        assert_eq!(&ci.commit_sha(), "fallback_sha_when_missing_head");
    }

    #[test]
    fn job() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("GITHUB_WORKFLOW".to_string(), "deploy".to_string());
        env.insert("GITHUB_JOB".to_string(), "run_tests".to_string());

        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.workflow(), "deploy");
        assert_eq!(&ci.job(), "run_tests");
    }

    #[test]
    fn build() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "GITHUB_SERVER_URL".to_string(),
            "https://github.com".to_string(),
        );
        env.insert("GITHUB_REPOSITORY".to_string(), "qltysh/qlty".to_string());
        env.insert("GITHUB_RUN_ID".to_string(), "42".to_string());
        env.insert("GITHUB_RUN_ATTEMPT".to_string(), "3".to_string());

        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_id(), "42:3");
        assert_eq!(
            &ci.build_url(),
            "https://github.com/qltysh/qlty/actions/runs/42:3"
        );
    }

    #[test]
    fn tag_build() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("GITHUB_REF_TYPE".to_string(), "tag".to_string());
        env.insert("GITHUB_REF_NAME".to_string(), "v1.2.3".to_string());
        env.insert(
            "GITHUB_SHA".to_string(),
            "77948d72a8b5ea21bb335e8e674bad99413da7a2".to_string(),
        );

        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "");
        assert_eq!(ci.git_tag(), Some("v1.2.3".to_string()));
        assert_eq!(&ci.commit_sha(), "77948d72a8b5ea21bb335e8e674bad99413da7a2");
    }

    #[test]
    fn branch_build_no_tag() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("GITHUB_REF_TYPE".to_string(), "branch".to_string());
        env.insert("GITHUB_REF_NAME".to_string(), "main".to_string());

        let ci = GitHub {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "main");
        assert_eq!(ci.git_tag(), None);
    }
}
