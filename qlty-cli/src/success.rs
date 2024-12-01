use crate::errors::CommandError;
use qlty_config::config::CheckTrigger;

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    Manual,
    PreCommit,
    PrePush,
    Build,
}

impl From<Trigger> for CheckTrigger {
    fn from(trigger: Trigger) -> Self {
        match trigger {
            Trigger::Manual => CheckTrigger::Manual,
            Trigger::PreCommit => CheckTrigger::PreCommit,
            Trigger::PrePush => CheckTrigger::PrePush,
            Trigger::Build => CheckTrigger::Build,
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct CommandSuccess {
    pub trigger: Option<Trigger>,
    pub issues_count: Option<usize>,
    pub fixed_count: usize,
    pub fixable_count: usize,
    pub fail: bool,
}

impl CommandSuccess {
    pub fn ok() -> Result<Self, CommandError> {
        Ok(Self::default())
    }

    pub fn exit_code(&self) -> i32 {
        if self.fail {
            1
        } else {
            0
        }
    }
}
