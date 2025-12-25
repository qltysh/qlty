use crate::{
    ci::CI,
    env::{EnvSource, SystemEnv},
};

#[derive(Debug)]
pub struct Jenkins {
    env: Box<dyn EnvSource>,
}

impl Default for Jenkins {
    fn default() -> Self {
        Self {
            env: Box::<SystemEnv>::default(),
        }
    }
}

impl CI for Jenkins {
    fn detect(&self) -> bool {
        self.env.var("JENKINS_URL").is_some()
    }

    fn ci_name(&self) -> String {
        "Jenkins".to_string()
    }

    fn ci_url(&self) -> String {
        self.env.var("JENKINS_URL").unwrap_or_default()
    }

    fn repository_name(&self) -> String {
        String::new()
    }

    fn repository_url(&self) -> String {
        self.env.var("GIT_URL").unwrap_or_default()
    }

    fn branch(&self) -> String {
        self.env
            .var("CHANGE_BRANCH")
            .or_else(|| self.env.var("BRANCH_NAME"))
            .unwrap_or_default()
    }

    fn pull_number(&self) -> String {
        self.env.var("CHANGE_ID").unwrap_or_default()
    }

    fn pull_url(&self) -> String {
        self.env.var("CHANGE_URL").unwrap_or_default()
    }

    fn commit_sha(&self) -> String {
        self.env.var("GIT_COMMIT").unwrap_or_default()
    }

    fn git_tag(&self) -> Option<String> {
        self.env.var("TAG_NAME").filter(|tag| !tag.is_empty())
    }

    fn workflow(&self) -> String {
        String::new()
    }

    fn job(&self) -> String {
        self.env.var("JOB_NAME").unwrap_or_default()
    }

    fn build_id(&self) -> String {
        self.env.var("INVOCATION_ID").unwrap_or_default()
    }

    fn build_url(&self) -> String {
        self.env.var("JOB_URL").unwrap_or_default()
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
        let ci = Jenkins {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(ci.detect(), false);

        let mut env: HashMap<String, String> = HashMap::default();
        env.insert(
            "JENKINS_URL".to_string(),
            "https://jenkins.example.com".to_string(),
        );
        let ci = Jenkins {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.detect(), true);
        assert_eq!(&ci.ci_name(), "Jenkins");
        assert_eq!(&ci.ci_url(), "https://jenkins.example.com");
    }

    #[test]
    fn branch_from_change_branch() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CHANGE_BRANCH".to_string(), "feature-branch".to_string());
        env.insert("BRANCH_NAME".to_string(), "PR-123".to_string());

        let ci = Jenkins {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "feature-branch");
    }

    #[test]
    fn branch_from_branch_name() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("BRANCH_NAME".to_string(), "main".to_string());

        let ci = Jenkins {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.branch(), "main");
    }

    #[test]
    fn commit_sha() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("GIT_COMMIT".to_string(), "abc123def456".to_string());

        let ci = Jenkins {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.commit_sha(), "abc123def456");
    }

    #[test]
    fn pull_request() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("CHANGE_ID".to_string(), "42".to_string());
        env.insert(
            "CHANGE_URL".to_string(),
            "https://github.com/owner/repo/pull/42".to_string(),
        );

        let ci = Jenkins {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.pull_number(), "42");
        assert_eq!(&ci.pull_url(), "https://github.com/owner/repo/pull/42");
    }

    #[test]
    fn git_tag() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("TAG_NAME".to_string(), "v1.0.0".to_string());

        let ci = Jenkins {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.git_tag(), Some("v1.0.0".to_string()));
    }

    #[test]
    fn git_tag_empty() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("TAG_NAME".to_string(), "".to_string());

        let ci = Jenkins {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(ci.git_tag(), None);
    }

    #[test]
    fn job() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("JOB_NAME".to_string(), "my-pipeline/main".to_string());

        let ci = Jenkins {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.job(), "my-pipeline/main");
    }

    #[test]
    fn workflow_empty() {
        let ci = Jenkins {
            env: Box::new(HashMapEnv::default()),
        };
        assert_eq!(&ci.workflow(), "");
    }

    #[test]
    fn build() {
        let mut env: HashMap<String, String> = HashMap::default();
        env.insert("INVOCATION_ID".to_string(), "12345-abcde".to_string());
        env.insert(
            "JOB_URL".to_string(),
            "https://jenkins.example.com/job/my-job/123/".to_string(),
        );

        let ci = Jenkins {
            env: Box::new(HashMapEnv::new(env)),
        };
        assert_eq!(&ci.build_id(), "12345-abcde");
        assert_eq!(
            &ci.build_url(),
            "https://jenkins.example.com/job/my-job/123/"
        );
    }
}
