use crate::{
    ci::CI,
    env::{EnvSource, SystemEnv},
};

#[derive(Debug)]
pub struct CircleCI {
    env: Box<dyn EnvSource>,
}

impl Default for CircleCI {
    fn default() -> Self {
        Self {
            env: Box::<SystemEnv>::default(),
        }
    }
}

impl CI for CircleCI {
    fn detect(&self) -> bool {
        self.env.var("CIRCLECI").unwrap_or_default() == "true"
    }

    fn ci_name(&self) -> String {
        "CircleCI".to_string()
    }

    fn ci_url(&self) -> String {
        // CircleCI doesn't expose a server URL
        "".to_string()
    }

    fn branch(&self) -> String {
        self.env.var("CIRCLE_BRANCH").unwrap_or_default()
    }

    fn workflow(&self) -> String {
        self.env.var("CIRCLE_WORKFLOW_ID").unwrap_or_default()
    }

    fn job(&self) -> String {
        self.env.var("CIRCLE_JOB").unwrap_or_default()
    }

    fn build_id(&self) -> String {
        self.env.var("CIRCLE_BUILD_NUM").unwrap_or_default()
    }

    fn build_url(&self) -> String {
        self.env.var("CIRCLE_BUILD_URL").unwrap_or_default()
    }

    fn pull_number(&self) -> String {
        if self.pull_url() != "" {
            self.pull_url()
                .split('/')
                .next_back()
                .unwrap_or_default()
                .to_string()
        } else {
            "".to_string()
        }
    }

    fn repository_name(&self) -> String {
        self.env.var("CIRCLE_PROJECT_REPONAME").unwrap_or_default()
    }

    fn repository_url(&self) -> String {
        self.env.var("CIRCLE_REPOSITORY_URL").unwrap_or_default()
    }

    fn pull_url(&self) -> String {
        self.env.var("CIRCLE_PULL_REQUEST").unwrap_or_default()
    }

    fn commit_sha(&self) -> String {
        self.env.var("CIRCLE_SHA1").unwrap_or_default()
    }

    fn git_tag(&self) -> Option<String> {
        self.env.var("CIRCLE_TAG").filter(|tag| !tag.is_empty())
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
        let ci = CircleCI {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(ci.detect(), false);

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CIRCLECI".to_string(), "true".to_string());
        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.detect(), true);
        assert_eq!(&ci.ci_name(), "CircleCI");
        assert_eq!(&ci.ci_url(), "");
    }

    #[test]
    fn branch() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CIRCLE_BRANCH".to_string(), "main".to_string());

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "main");
    }

    #[test]
    fn workflow() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CIRCLE_WORKFLOW_ID".to_string(), "workflow_id".to_string());

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.workflow(), "workflow_id");
    }

    #[test]
    fn job() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CIRCLE_JOB".to_string(), "job_name".to_string());

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.job(), "job_name");
    }

    #[test]
    fn build_id() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CIRCLE_BUILD_NUM".to_string(), "1234".to_string());

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_id(), "1234");
    }

    #[test]
    fn build_url() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "CIRCLE_BUILD_URL".to_string(),
            "http://example.com/build/1234".to_string(),
        );

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_url(), "http://example.com/build/1234");
    }

    #[test]
    fn pull_number() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "CIRCLE_PULL_REQUEST".to_string(),
            "https://github.com/user/repo/pull/42".to_string(),
        );

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_number(), "42");
    }

    #[test]
    fn repository_name() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "CIRCLE_PROJECT_REPONAME".to_string(),
            "repo_name".to_string(),
        );

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.repository_name(), "repo_name");
    }

    #[test]
    fn repository_url() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "CIRCLE_REPOSITORY_URL".to_string(),
            "https://github.com/user/repo".to_string(),
        );

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.repository_url(), "https://github.com/user/repo");
    }

    #[test]
    fn commit_sha() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CIRCLE_SHA1".to_string(), "abc123".to_string());

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.commit_sha(), "abc123");
    }

    #[test]
    fn git_tag() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CIRCLE_TAG".to_string(), "v1.2.3".to_string());

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.git_tag(), Some("v1.2.3".to_string()));
    }

    #[test]
    fn git_tag_empty() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CIRCLE_TAG".to_string(), "".to_string());

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.git_tag(), None);
    }

    #[test]
    fn git_tag_not_set() {
        let env: HashMap<String, String> = HashMap::default();

        let ci = CircleCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.git_tag(), None);
    }
}
