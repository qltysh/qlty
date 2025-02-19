use crate::{
    ci::CI,
    env::{EnvSource, SystemEnv},
};

#[derive(Debug)]
pub struct Semaphore {
    env: Box<dyn EnvSource>,
}

impl Default for Semaphore {
    fn default() -> Self {
        Self {
            env: Box::<SystemEnv>::default(),
        }
    }
}

impl CI for Semaphore {
    fn detect(&self) -> bool {
        self.env.var("SEMAPHORE").unwrap_or_default() == "true"
    }

    fn ci_name(&self) -> String {
        "Semaphore".to_string()
    }

    fn ci_url(&self) -> String {
        self.env
            .var("SEMAPHORE_ORGANIZATION_URL")
            .unwrap_or_default()
    }

    fn branch(&self) -> String {
        self.env.var("SEMAPHORE_GIT_BRANCH").unwrap_or_default()
    }

    fn workflow(&self) -> String {
        self.env.var("SEMAPHORE_WORKFLOW_ID").unwrap_or_default()
    }

    fn job(&self) -> String {
        self.env.var("SEMAPHORE_JOB_NAME").unwrap_or_default()
    }

    fn build_id(&self) -> String {
        self.env.var("SEMAPHORE_JOB_ID").unwrap_or_default()
    }

    fn build_url(&self) -> String {
        format!("{}/jobs/{}", self.ci_url(), self.build_id())
    }

    fn pull_number(&self) -> String {
        self.env.var("SEMAPHORE_GIT_PR_NUMBER").unwrap_or_default()
    }

    fn repository_name(&self) -> String {
        self.env.var("SEMAPHORE_GIT_REPO_NAME").unwrap_or_default()
    }

    fn repository_url(&self) -> String {
        self.env.var("SEMAPHORE_GIT_URL").unwrap_or_default()
    }

    fn pull_url(&self) -> String {
        // Semaphore doesn't expose a pull url
        "".to_string()
    }

    fn commit_sha(&self) -> String {
        self.env.var("SEMAPHORE_GIT_SHA").unwrap_or_default()
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
        let ci: Semaphore = Semaphore {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(ci.detect(), false);

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("SEMAPHORE".to_string(), "true".to_string());
        let ci = Semaphore {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.detect(), true);
        assert_eq!(&ci.ci_name(), "Semaphore");
        assert_eq!(&ci.ci_url(), "");
    }

    #[test]
    fn branch() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("SEMAPHORE_GIT_BRANCH".to_string(), "main".to_string());

        let ci = Semaphore {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "main");
    }

    #[test]
    fn workflow() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "SEMAPHORE_WORKFLOW_ID".to_string(),
            "workflow_id".to_string(),
        );

        let ci = Semaphore {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.workflow(), "workflow_id");
    }

    #[test]
    fn job() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("SEMAPHORE_JOB_NAME".to_string(), "job_name".to_string());

        let ci = Semaphore {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.job(), "job_name");
    }

    #[test]
    fn build_id() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("SEMAPHORE_JOB_ID".to_string(), "1234".to_string());

        let ci = Semaphore {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_id(), "1234");
    }

    #[test]
    fn build_url() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("SEMAPHORE_JOB_ID".to_string(), "1234".to_string());
        env.insert(
            "SEMAPHORE_ORGANIZATION_URL".to_string(),
            "http://example.semaphoreci.com".to_string(),
        );

        let ci = Semaphore {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_url(), "http://example.semaphoreci.com/jobs/1234");
    }

    #[test]
    fn pull_number() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("SEMAPHORE_GIT_PR_NUMBER".to_string(), "42".to_string());

        let ci = Semaphore {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_number(), "42");
    }

    #[test]
    fn repository_name() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "SEMAPHORE_GIT_REPO_NAME".to_string(),
            "repo_name".to_string(),
        );

        let ci = Semaphore {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.repository_name(), "repo_name");
    }

    #[test]
    fn repository_url() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "SEMAPHORE_GIT_URL".to_string(),
            "https://github.com/user/repo".to_string(),
        );

        let ci = Semaphore {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.repository_url(), "https://github.com/user/repo");
    }

    #[test]
    fn commit_sha() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("SEMAPHORE_GIT_SHA".to_string(), "abc123".to_string());

        let ci = Semaphore {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.commit_sha(), "abc123");
    }
}
