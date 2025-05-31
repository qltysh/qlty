use crate::{
    ci::CI,
    env::{EnvSource, SystemEnv},
};

#[derive(Debug)]
pub struct TravisCI {
    env: Box<dyn EnvSource>,
}

impl Default for TravisCI {
    fn default() -> Self {
        Self {
            env: Box::<SystemEnv>::default(),
        }
    }
}

impl CI for TravisCI {
    fn detect(&self) -> bool {
        self.env.var("TRAVIS").unwrap_or_default() == "true"
    }

    fn ci_name(&self) -> String {
        "Travis".to_string()
    }

    fn ci_url(&self) -> String {
        "https://travis-ci.com".to_string()
    }

    fn branch(&self) -> String {
        self.env
            .var("TRAVIS_PULL_REQUEST_BRANCH")
            .or_else(|| self.env.var("TRAVIS_BRANCH"))
            .unwrap_or_default()
    }

    fn workflow(&self) -> String {
        "".to_string()
    }

    fn job(&self) -> String {
        self.env.var("TRAVIS_JOB_NAME").unwrap_or_default()
    }

    fn build_id(&self) -> String {
        self.env.var("TRAVIS_BUILD_ID").unwrap_or_default()
    }

    fn build_url(&self) -> String {
        self.env.var("TRAVIS_BUILD_WEB_URL").unwrap_or_default()
    }

    fn pull_number(&self) -> String {
        let travis_pull_request = self.env.var("TRAVIS_PULL_REQUEST").unwrap_or_default();
        if travis_pull_request != "false" {
            travis_pull_request
        } else {
            String::new()
        }
    }

    fn repository_name(&self) -> String {
        self.env
            .var("TRAVIS_REPO_SLUG")
            .unwrap_or_default()
            .split('/')
            .nth(1)
            .unwrap_or("")
            .to_string()
    }

    fn repository_url(&self) -> String {
        "".to_string()
    }

    fn pull_url(&self) -> String {
        "".to_string()
    }

    fn commit_sha(&self) -> String {
        self.env.var("TRAVIS_COMMIT").unwrap_or_default()
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
        let ci: TravisCI = TravisCI {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(ci.detect(), false);

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("TRAVIS".to_string(), "true".to_string());
        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.detect(), true);
        assert_eq!(&ci.ci_name(), "Travis");
        assert_eq!(&ci.ci_url(), "https://travis-ci.com");
    }

    #[test]
    fn branch() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("TRAVIS_BRANCH".to_string(), "main".to_string());

        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "main");
    }

    #[test]
    fn branch_pull_request() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("TRAVIS_BRANCH".to_string(), "main".to_string());
        env.insert(
            "TRAVIS_PULL_REQUEST_BRANCH".to_string(),
            "feature-branch".to_string(),
        );

        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "feature-branch");
    }

    #[test]
    fn workflow() {
        let env: HashMap<String, String> = HashMap::default();

        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.workflow(), "");
    }

    #[test]
    fn job() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("TRAVIS_JOB_NAME".to_string(), "job_name".to_string());

        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.job(), "job_name");
    }

    #[test]
    fn build_id() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("TRAVIS_BUILD_ID".to_string(), "1234".to_string());

        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_id(), "1234");
    }

    #[test]
    fn build_url() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "TRAVIS_BUILD_WEB_URL".to_string(),
            "https://travis-ci.com/user/repo/builds/1234".to_string(),
        );

        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(
            &ci.build_url(),
            "https://travis-ci.com/user/repo/builds/1234"
        );
    }

    #[test]
    fn pull_number() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("TRAVIS_PULL_REQUEST".to_string(), "42".to_string());

        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_number(), "42");
    }

    #[test]
    fn pull_number_false() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("TRAVIS_PULL_REQUEST".to_string(), "false".to_string());

        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_number(), "");
    }

    #[test]
    fn repository_name() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("TRAVIS_REPO_SLUG".to_string(), "user/repo_name".to_string());

        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.repository_name(), "repo_name");
    }

    #[test]
    fn repository_url() {
        let env: HashMap<String, String> = HashMap::default();

        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.repository_url(), "");
    }

    #[test]
    fn commit_sha() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("TRAVIS_COMMIT".to_string(), "abc123".to_string());

        let ci = TravisCI {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.commit_sha(), "abc123");
    }
}
