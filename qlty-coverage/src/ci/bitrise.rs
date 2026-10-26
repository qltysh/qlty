use crate::ci::CI;
use qlty_config::env::{EnvSource, SystemEnv};

#[derive(Debug)]
pub struct Bitrise {
    env: Box<dyn EnvSource>,
}

impl Default for Bitrise {
    fn default() -> Self {
        Self {
            env: Box::<SystemEnv>::default(),
        }
    }
}

impl CI for Bitrise {
    fn detect(&self) -> bool {
        self.env.var("BITRISE_BUILD_SLUG").is_some()
    }

    fn ci_name(&self) -> String {
        "bitrise".to_string()
    }

    fn ci_url(&self) -> String {
        "https://bitrise.io/".to_string()
    }

    fn commit_sha(&self) -> String {
        self.env.var("GIT_CLONE_COMMIT_HASH").unwrap_or_default()
    }

    fn repository_name(&self) -> String {
        self.env
            .var("BITRISEIO_GIT_REPOSITORY_SLUG")
            .unwrap_or_default()
    }

    fn repository_url(&self) -> String {
        self.env.var("GIT_REPOSITORY_URL").unwrap_or_default()
    }

    fn branch(&self) -> String {
        self.env.var("BITRISE_GIT_BRANCH").unwrap_or_default()
    }

    fn pull_number(&self) -> String {
        self.env.var("BITRISE_PULL_REQUEST").unwrap_or_default()
    }

    fn pull_url(&self) -> String {
        let pull_number = self.pull_number();
        let repository_url = self.repository_url();

        if !pull_number.is_empty() && !repository_url.is_empty() {
            format!("{}/pull/{}", repository_url, pull_number)
        } else {
            String::from("")
        }
    }

    fn git_tag(&self) -> Option<String> {
        self.env
            .var("BITRISE_GIT_TAG")
            .filter(|tag| !tag.is_empty())
    }

    fn workflow(&self) -> String {
        self.env
            .var("BITRISE_TRIGGERED_WORKFLOW_ID")
            .unwrap_or_default()
    }

    fn job(&self) -> String {
        String::from("")
    }

    fn build_id(&self) -> String {
        self.env.var("BITRISE_BUILD_SLUG").unwrap_or_default()
    }

    fn build_url(&self) -> String {
        self.env.var("BITRISE_BUILD_URL").unwrap_or_default()
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
        let ci = Bitrise {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(ci.detect(), false);

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BITRISE_BUILD_SLUG".to_string(), "abc123".to_string());
        let ci = Bitrise {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.detect(), true);
        assert_eq!(&ci.ci_name(), "bitrise");
    }

    #[test]
    fn ci_url() {
        let ci = Bitrise {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(&ci.ci_url(), "https://bitrise.io/");
    }

    #[test]
    fn commit_sha() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "GIT_CLONE_COMMIT_HASH".to_string(),
            "a1b2c3d4e5f6".to_string(),
        );

        let ci = Bitrise {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.commit_sha(), "a1b2c3d4e5f6");
    }

    #[test]
    fn repository() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "BITRISEIO_GIT_REPOSITORY_SLUG".to_string(),
            "my-repo".to_string(),
        );
        env.insert(
            "GIT_REPOSITORY_URL".to_string(),
            "https://github.com/qltysh/qlty".to_string(),
        );

        let ci = Bitrise {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.repository_name(), "my-repo");
        assert_eq!(&ci.repository_url(), "https://github.com/qltysh/qlty");
    }

    #[test]
    fn branch() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BITRISE_GIT_BRANCH".to_string(), "main".to_string());

        let ci = Bitrise {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "main");
    }

    #[test]
    fn pull_request() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BITRISE_PULL_REQUEST".to_string(), "42".to_string());
        env.insert(
            "GIT_REPOSITORY_URL".to_string(),
            "https://github.com/qltysh/qlty".to_string(),
        );

        let ci = Bitrise {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_number(), "42");
        assert_eq!(&ci.pull_url(), "https://github.com/qltysh/qlty/pull/42");
    }

    #[test]
    fn empty_pull_url() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BITRISE_PULL_REQUEST".to_string(), "".to_string());

        let ci = Bitrise {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_url(), "");
    }

    #[test]
    fn git_tag() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BITRISE_GIT_TAG".to_string(), "v1.2.3".to_string());

        let ci = Bitrise {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.git_tag(), Some("v1.2.3".to_string()));
    }

    #[test]
    fn empty_git_tag() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BITRISE_GIT_TAG".to_string(), "".to_string());

        let ci = Bitrise {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.git_tag(), None);
    }

    #[test]
    fn workflow() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "BITRISE_TRIGGERED_WORKFLOW_ID".to_string(),
            "primary".to_string(),
        );

        let ci = Bitrise {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.workflow(), "primary");
    }

    #[test]
    fn job() {
        let ci = Bitrise {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(&ci.job(), "");
    }

    #[test]
    fn build() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BITRISE_BUILD_SLUG".to_string(), "build123".to_string());
        env.insert(
            "BITRISE_BUILD_URL".to_string(),
            "https://app.bitrise.io/build/build123".to_string(),
        );

        let ci = Bitrise {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_id(), "build123");
        assert_eq!(&ci.build_url(), "https://app.bitrise.io/build/build123");
    }
}
