mod circleci;
mod codefresh;
mod github;
mod gitlab;
mod semaphore;

pub use circleci::CircleCI;
pub use codefresh::Codefresh;
pub use github::GitHub;
pub use gitlab::GitLab;
use qlty_types::tests::v1::CoverageMetadata;
pub use semaphore::Semaphore;

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
        CoverageMetadata {
            ci: self.ci_name(),
            build_id: self.build_id(),
            commit_sha: self.commit_sha(),
            branch: self.branch(),
            pull_request_number: self.pull_number(),

            ..Default::default()
        }
    }
}

pub fn current() -> Option<Box<dyn CI>> {
    all().into_iter().find(|ci| ci.detect())
}

pub fn all() -> Vec<Box<dyn CI>> {
    vec![
        Box::<GitHub>::default(),
        Box::<GitLab>::default(),
        Box::<CircleCI>::default(),
        Box::<Codefresh>::default(),
        Box::<Semaphore>::default(),
    ]
}
