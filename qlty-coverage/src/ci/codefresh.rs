use crate::ci::CI;
use qlty_config::env::{EnvSource, SystemEnv};

#[derive(Debug)]
pub struct Codefresh {
    env: Box<dyn EnvSource>,
}

impl Default for Codefresh {
    fn default() -> Self {
        Self {
            env: Box::<SystemEnv>::default(),
        }
    }
}

impl CI for Codefresh {
    fn detect(&self) -> bool {
        !self
            .env
            .var("CF_PIPELINE_NAME")
            .unwrap_or_default()
            .is_empty()
    }

    fn ci_name(&self) -> String {
        "Codefresh".to_string()
    }

    fn ci_url(&self) -> String {
        self.env.var("CF_URL").unwrap_or_default()
    }

    fn branch(&self) -> String {
        self.env.var("CF_BRANCH").unwrap_or_default()
    }

    fn workflow(&self) -> String {
        self.env.var("CF_PIPELINE_NAME").unwrap_or_default()
    }

    fn job(&self) -> String {
        self.env.var("CF_STEP_NAME").unwrap_or_default()
    }

    fn build_id(&self) -> String {
        self.env.var("CF_BUILD_ID").unwrap_or_default()
    }

    fn build_url(&self) -> String {
        self.env.var("CF_BUILD_URL").unwrap_or_default()
    }

    fn pull_number(&self) -> String {
        self.env.var("CF_PULL_REQUEST_NUMBER").unwrap_or_default()
    }

    fn repository_name(&self) -> String {
        let repository_owner = self.env.var("CF_REPO_OWNER").unwrap_or_default();
        let repository_name = self.env.var("CF_REPO_NAME").unwrap_or_default();

        if repository_owner.is_empty() || repository_name.is_empty() {
            "".to_string()
        } else {
            format!("{}/{}", repository_owner, repository_name)
        }
    }

    fn repository_url(&self) -> String {
        // Codefresh does not provide a repository URL
        "".to_string()
    }

    fn pull_url(&self) -> String {
        // Codefresh does not provide a pull request URL
        "".to_string()
    }

    fn commit_sha(&self) -> String {
        self.env.var("CF_REVISION").unwrap_or_default()
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
        let ci = Codefresh {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(ci.detect(), false);

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CF_PIPELINE_NAME".to_string(), "tests".to_string());
        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.detect(), true);
        assert_eq!(&ci.ci_name(), "Codefresh");
    }

    #[test]
    fn ci_url() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CF_URL".to_string(), "https://codefresh.io".to_string());

        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.ci_url(), "https://codefresh.io");
    }

    #[test]
    fn branch() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CF_BRANCH".to_string(), "main".to_string());

        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "main");
    }

    #[test]
    fn workflow() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CF_PIPELINE_NAME".to_string(), "pipepline_name".to_string());

        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.workflow(), "pipepline_name");
    }

    #[test]
    fn job() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CF_STEP_NAME".to_string(), "step_name".to_string());

        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.job(), "step_name");
    }

    #[test]
    fn build_id() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CF_BUILD_ID".to_string(), "1234".to_string());

        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_id(), "1234");
    }

    #[test]
    fn build_url() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "CF_BUILD_URL".to_string(),
            "http://example.com/build/1234".to_string(),
        );

        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_url(), "http://example.com/build/1234");
    }

    #[test]
    fn pull_number() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CF_PULL_REQUEST_NUMBER".to_string(), "43".to_string());

        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_number(), "43");
    }

    #[test]
    fn repository_name() {
        let mut env: HashMap<String, String> = HashMap::default();

        env.insert("CF_REPO_OWNER".to_string(), "owner_name".to_string());

        env.insert("CF_REPO_NAME".to_string(), "repo_name".to_string());

        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.repository_name(), "owner_name/repo_name");
    }

    #[test]
    fn repository_url() {
        let env: HashMap<String, String> = HashMap::default();

        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.repository_url(), "");
    }

    #[test]
    fn pull_url() {
        let env: HashMap<String, String> = HashMap::default();

        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_url(), "");
    }

    #[test]
    fn commit_sha() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "CF_REVISION".to_string(),
            "aca725d7c7f20a91575ed1d2616cf8bffa635704".to_string(),
        );

        let ci = Codefresh {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.commit_sha(), "aca725d7c7f20a91575ed1d2616cf8bffa635704");
    }
}
