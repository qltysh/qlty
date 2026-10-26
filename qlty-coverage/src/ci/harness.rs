use crate::ci::CI;
use qlty_config::env::{EnvSource, SystemEnv};

#[derive(Debug)]
pub struct Harness {
    env: Box<dyn EnvSource>,
}

impl Default for Harness {
    fn default() -> Self {
        Self {
            env: Box::<SystemEnv>::default(),
        }
    }
}

impl CI for Harness {
    fn detect(&self) -> bool {
        // Detect on a Harness-specific signal rather than DRONE=true: Harness is built on the
        // Drone engine and sets DRONE=true, but so does standalone Drone (which has no HARNESS_*
        // vars), and we don't want to mislabel a Drone build as Harness with empty metadata.
        self.env.var("HARNESS_EXECUTION_ID").is_some()
    }

    fn ci_name(&self) -> String {
        "Harness".to_string()
    }

    fn ci_url(&self) -> String {
        // Harness SaaS has no stable per-build server-URL variable
        "".to_string()
    }

    fn branch(&self) -> String {
        // On PR builds DRONE_BRANCH is the target branch, so prefer the source branch.
        if !self.pull_number().is_empty() {
            let source = self.env.var("DRONE_SOURCE_BRANCH").unwrap_or_default();
            if !source.is_empty() {
                return source;
            }
        }
        self.env.var("DRONE_BRANCH").unwrap_or_default()
    }

    fn workflow(&self) -> String {
        self.env.var("HARNESS_PIPELINE_ID").unwrap_or_default()
    }

    fn job(&self) -> String {
        self.env.var("DRONE_STAGE_NAME").unwrap_or_default()
    }

    fn build_id(&self) -> String {
        // HARNESS_EXECUTION_ID is a globally-unique immutable UUID, stable across all
        // steps/shards of one run. The incremental HARNESS_BUILD_ID is only unique within a
        // single pipeline and would collide across pipelines in the same project, causing
        // coverage from unrelated runs to be merged.
        self.env.var("HARNESS_EXECUTION_ID").unwrap_or_default()
    }

    fn build_url(&self) -> String {
        self.env.var("DRONE_BUILD_LINK").unwrap_or_default()
    }

    fn pull_number(&self) -> String {
        self.env.var("DRONE_PULL_REQUEST").unwrap_or_default()
    }

    fn repository_name(&self) -> String {
        self.env.var("DRONE_REPO").unwrap_or_default()
    }

    fn repository_url(&self) -> String {
        self.env.var("DRONE_REPO_LINK").unwrap_or_default()
    }

    fn pull_url(&self) -> String {
        if !self.pull_number().is_empty() && !self.repository_url().is_empty() {
            format!("{}/pull/{}", self.repository_url(), self.pull_number())
        } else {
            "".to_string()
        }
    }

    fn commit_sha(&self) -> String {
        // On PR builds DRONE_COMMIT_SHA is already the PR head commit, so no merge-commit
        // dance is needed like on GitHub.
        self.env.var("DRONE_COMMIT_SHA").unwrap_or_default()
    }

    fn git_tag(&self) -> Option<String> {
        self.env.var("DRONE_TAG").filter(|tag| !tag.is_empty())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

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
        let ci = Harness {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(ci.detect(), false);

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "HARNESS_EXECUTION_ID".to_string(),
            "Hx9kE2QwRim".to_string(),
        );
        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.detect(), true);
        assert_eq!(&ci.ci_name(), "Harness");
        assert_eq!(&ci.ci_url(), "");
    }

    #[test]
    fn does_not_detect_on_drone_alone() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("DRONE".to_string(), "true".to_string());

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.detect(), false);
    }

    #[test]
    fn branch_on_pull_request() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("DRONE_PULL_REQUEST".to_string(), "42".to_string());
        env.insert("DRONE_SOURCE_BRANCH".to_string(), "feature".to_string());
        env.insert("DRONE_BRANCH".to_string(), "main".to_string());

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "feature");
    }

    #[test]
    fn branch_on_push() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("DRONE_BRANCH".to_string(), "main".to_string());

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "main");
    }

    #[test]
    fn pull_number() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("DRONE_PULL_REQUEST".to_string(), "42".to_string());

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_number(), "42");
    }

    #[test]
    fn pull_url() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("DRONE_PULL_REQUEST".to_string(), "42".to_string());
        env.insert(
            "DRONE_REPO_LINK".to_string(),
            "https://github.com/qltysh/qlty".to_string(),
        );

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_url(), "https://github.com/qltysh/qlty/pull/42");
    }

    #[test]
    fn build_id() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "HARNESS_EXECUTION_ID".to_string(),
            "Hx9kE2QwRim-uniqueUUID".to_string(),
        );
        env.insert("HARNESS_BUILD_ID".to_string(), "12".to_string());

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_id(), "Hx9kE2QwRim-uniqueUUID");
    }

    #[test]
    fn build_url() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "DRONE_BUILD_LINK".to_string(),
            "https://app.harness.io/build/1".to_string(),
        );

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_url(), "https://app.harness.io/build/1");
    }

    #[test]
    fn workflow_and_job() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("HARNESS_PIPELINE_ID".to_string(), "pipeline_id".to_string());
        env.insert("DRONE_STAGE_NAME".to_string(), "build".to_string());

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.workflow(), "pipeline_id");
        assert_eq!(&ci.job(), "build");
    }

    #[test]
    fn repository() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("DRONE_REPO".to_string(), "qltysh/qlty".to_string());
        env.insert(
            "DRONE_REPO_LINK".to_string(),
            "https://github.com/qltysh/qlty".to_string(),
        );

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.repository_name(), "qltysh/qlty");
        assert_eq!(&ci.repository_url(), "https://github.com/qltysh/qlty");
    }

    #[test]
    fn commit_sha() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("DRONE_COMMIT_SHA".to_string(), "abc123".to_string());

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.commit_sha(), "abc123");
    }

    #[test]
    fn git_tag() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("DRONE_TAG".to_string(), "v1.2.3".to_string());

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.git_tag(), Some("v1.2.3".to_string()));
    }

    #[test]
    fn git_tag_empty() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("DRONE_TAG".to_string(), "".to_string());

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.git_tag(), None);
    }

    #[test]
    fn git_tag_not_set() {
        let env: HashMap<String, String> = HashMap::default();

        let ci = Harness {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.git_tag(), None);
    }
}
