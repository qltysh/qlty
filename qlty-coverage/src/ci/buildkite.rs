use crate::ci::CI;
use qlty_config::env::{EnvSource, SystemEnv};

#[derive(Debug)]
pub struct Buildkite {
    env: Box<dyn EnvSource>,
}

impl Default for Buildkite {
    fn default() -> Self {
        Self {
            env: Box::<SystemEnv>::default(),
        }
    }
}

impl CI for Buildkite {
    fn detect(&self) -> bool {
        self.env.var("BUILDKITE").is_some()
    }

    fn ci_name(&self) -> String {
        "Buildkite".to_string()
    }

    fn commit_sha(&self) -> String {
        self.env.var("BUILDKITE_COMMIT").unwrap_or_default()
    }

    fn branch(&self) -> String {
        self.env.var("BUILDKITE_BRANCH").unwrap_or_default()
    }

    fn job(&self) -> String {
        self.env.var("BUILDKITE_JOB_ID").unwrap_or_default()
    }

    fn build_id(&self) -> String {
        self.env.var("BUILDKITE_BUILD_ID").unwrap_or_default()
    }

    fn build_url(&self) -> String {
        self.env.var("BUILDKITE_BUILD_URL").unwrap_or_default()
    }

    fn pull_number(&self) -> String {
        self.env
            .var("BUILDKITE_PULL_REQUEST")
            .filter(|v| v != "false")
            .unwrap_or_default()
    }

    fn repository_name(&self) -> String {
        String::from("")
    }

    fn repository_url(&self) -> String {
        String::from("")
    }

    fn pull_url(&self) -> String {
        String::from("")
    }

    fn ci_url(&self) -> String {
        String::from("https://buildkite.com")
    }

    fn workflow(&self) -> String {
        self.env.var("BUILDKITE_PIPELINE_ID").unwrap_or_default()
    }

    fn git_tag(&self) -> Option<String> {
        self.env.var("BUILDKITE_TAG").filter(|tag| !tag.is_empty())
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
        let ci = Buildkite {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(ci.detect(), false);

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BUILDKITE".to_string(), "true".to_string());
        let ci = Buildkite {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.detect(), true);
        assert_eq!(&ci.ci_name(), "Buildkite");

        let not_buildkite = Buildkite {
            env: Box::new(HashMapEnv::new(HashMap::default())),
        };

        assert_eq!(not_buildkite.detect(), false);
    }

    #[test]
    fn branch() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BUILDKITE_BRANCH".to_string(), "main".to_string());

        let ci = Buildkite {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "main");
    }

    #[test]
    fn job() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BUILDKITE_JOB_ID".to_string(), "job_name".to_string());

        let ci = Buildkite {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.job(), "job_name");
    }

    #[test]
    fn build_id() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BUILDKITE_BUILD_ID".to_string(), "1234".to_string());

        let ci = Buildkite {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_id(), "1234");
    }

    #[test]
    fn build_url() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "BUILDKITE_BUILD_URL".to_string(),
            "http://example.com/build/1234".to_string(),
        );

        let ci = Buildkite {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_url(), "http://example.com/build/1234");
    }

    #[test]
    fn commit_sha() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BUILDKITE_COMMIT".to_string(), "abc123".to_string());

        let ci = Buildkite {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.commit_sha(), "abc123");
    }

    #[test]
    fn ci_url() {
        let ci = Buildkite {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(&ci.ci_url(), "https://buildkite.com");
    }

    #[test]
    fn workflow() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "BUILDKITE_PIPELINE_ID".to_string(),
            "pipeline-42".to_string(),
        );

        let ci = Buildkite {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.workflow(), "pipeline-42");
    }

    #[test]
    fn pull_number() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BUILDKITE_PULL_REQUEST".to_string(), "99".to_string());

        let ci = Buildkite {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_number(), "99");
    }

    #[test]
    fn empty_pull_number() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BUILDKITE_PULL_REQUEST".to_string(), "false".to_string());

        let ci = Buildkite {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_number(), "");
    }
}
