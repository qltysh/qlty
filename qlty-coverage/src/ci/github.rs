use crate::{
    ci::CI,
    env::{EnvSource, SystemEnv},
};
use regex::Regex;

#[derive(Debug)]
pub struct GitHub {
    env: Box<dyn EnvSource>,
}

impl Default for GitHub {
    fn default() -> Self {
        Self {
            env: Box::new(SystemEnv::default()),
        }
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
                } else {
                    if let Some(ref_name) = self.env.var("GITHUB_REF_NAME") {
                        ref_name
                    } else {
                        self.env.var("GITHUB_HEAD_REF").unwrap_or_default()
                    }
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
        self.env.var("GITHUB_SHA").unwrap_or_default()
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
    fn pull_request_build() {
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
}
