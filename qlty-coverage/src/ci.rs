mod buildkite;
mod circleci;
mod codefresh;
mod github;
mod gitlab;
mod semaphore;

pub use buildkite::Buildkite;
pub use circleci::CircleCI;
pub use codefresh::Codefresh;
pub use github::GitHub;
pub use gitlab::GitLab;
use qlty_types::tests::v1::CoverageMetadata;
pub use semaphore::Semaphore;

const QLTY_CI_ACTION_VERSION: &str = "QLTY_CI_ACTION_VERSION";

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
        let ci_action_version = std::env::var(QLTY_CI_ACTION_VERSION).ok();
        let mut uploader_tool = None;
        let mut uploader_tool_version = None;

        if let Some(version) = ci_action_version {
            let mut parts = version.split('@');
            if let Some(tool) = parts.next() {
                uploader_tool = Some(tool.to_string());
            }
            if let Some(version) = parts.next() {
                uploader_tool_version = Some(version.to_string());
            }
        }

        CoverageMetadata {
            ci: self.ci_name(),
            build_id: self.build_id(),
            commit_sha: self.commit_sha(),
            branch: self.branch(),
            pull_request_number: self.pull_number(),
            uploader_tool,
            uploader_tool_version,
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
        Box::<Buildkite>::default(),
        Box::<CircleCI>::default(),
        Box::<Codefresh>::default(),
        Box::<GitHub>::default(),
        Box::<GitLab>::default(),
        Box::<Semaphore>::default(),
    ]
}
