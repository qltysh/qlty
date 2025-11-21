mod bitrise;
mod buildkite;
mod circleci;
mod codefresh;
mod github;
mod gitlab;
mod semaphore;
mod travisci;

pub use bitrise::Bitrise;
pub use buildkite::Buildkite;
pub use circleci::CircleCI;
pub use codefresh::Codefresh;
pub use github::GitHub;
pub use gitlab::GitLab;
use qlty_types::tests::v1::CoverageMetadata;
pub use semaphore::Semaphore;
pub use travisci::TravisCI;

const QLTY_CI_UPLOADER_TOOL: &str = "QLTY_CI_UPLOADER_TOOL";
const QLTY_CI_UPLOADER_TOOL_VERSION: &str = "QLTY_CI_UPLOADER_TOOL_VERSION";

pub trait CI {
    fn detect(&self) -> bool;

    // Information about the CI system
    fn ci_name(&self) -> String;
    fn ci_url(&self) -> String;

    // Information about the repository
    fn repository_name(&self) -> String;
    fn repository_url(&self) -> String;

    // Information about what is being built
    fn branch(&self) -> String;
    fn pull_number(&self) -> String;
    fn pull_url(&self) -> String;
    fn commit_sha(&self) -> String;
    fn git_tag(&self) -> Option<String> {
        None
    }

    fn is_merge_group_branch(&self) -> bool {
        let branch = self.branch();
        branch.starts_with("gh-readonly-queue/")
    }

    // Information about the commit
    // TODO: Message
    // TODO: Author and committer
    // TODO: Timestamp

    // Information about the build configuration
    // Structured as Workflow > Job
    fn workflow(&self) -> String;
    fn job(&self) -> String;

    // Unique identifier of this execution or run
    fn build_id(&self) -> String;
    fn build_url(&self) -> String;

    fn metadata(&self) -> CoverageMetadata {
        CoverageMetadata {
            ci: self.ci_name(),
            build_id: self.build_id(),
            commit_sha: self.commit_sha(),
            branch: self.branch(),
            pull_request_number: self.pull_number(),
            git_tag: self.git_tag(),
            uploader_tool: std::env::var(QLTY_CI_UPLOADER_TOOL).ok(),
            uploader_tool_version: std::env::var(QLTY_CI_UPLOADER_TOOL_VERSION).ok(),
            publish_command: std::env::args().collect::<Vec<String>>().join(" "),
            ..Default::default()
        }
    }
}

pub fn current() -> Option<Box<dyn CI>> {
    all().into_iter().find(|ci| ci.detect())
}

pub fn all() -> Vec<Box<dyn CI>> {
    vec![
        Box::<Bitrise>::default(),
        Box::<Buildkite>::default(),
        Box::<CircleCI>::default(),
        Box::<Codefresh>::default(),
        Box::<GitHub>::default(),
        Box::<GitLab>::default(),
        Box::<Semaphore>::default(),
        Box::<TravisCI>::default(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCI {
        branch: String,
    }

    impl CI for MockCI {
        fn detect(&self) -> bool {
            true
        }

        fn ci_name(&self) -> String {
            "MockCI".to_string()
        }

        fn ci_url(&self) -> String {
            "https://mock.ci".to_string()
        }

        fn repository_name(&self) -> String {
            "mock/repo".to_string()
        }

        fn repository_url(&self) -> String {
            "https://github.com/mock/repo".to_string()
        }

        fn branch(&self) -> String {
            self.branch.clone()
        }

        fn pull_number(&self) -> String {
            "".to_string()
        }

        fn pull_url(&self) -> String {
            "".to_string()
        }

        fn commit_sha(&self) -> String {
            "abc123".to_string()
        }

        fn workflow(&self) -> String {
            "workflow".to_string()
        }

        fn job(&self) -> String {
            "job".to_string()
        }

        fn build_id(&self) -> String {
            "123".to_string()
        }

        fn build_url(&self) -> String {
            "https://mock.ci/builds/123".to_string()
        }
    }

    #[test]
    fn test_is_merge_group_event_with_merge_queue_branch() {
        let ci = MockCI {
            branch: "gh-readonly-queue/main/pr-30-e6afd52a678226e8c732f2012aabb2fbfd97e5ac"
                .to_string(),
        };
        assert_eq!(ci.is_merge_group_branch(), true);
    }

    #[test]
    fn test_is_merge_group_event_with_regular_branch() {
        let ci = MockCI {
            branch: "main".to_string(),
        };
        assert_eq!(ci.is_merge_group_branch(), false);
    }
}
