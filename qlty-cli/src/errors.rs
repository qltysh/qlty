use crate::CommandSuccess;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Config error")]
    Config,

    #[error("Lint error")]
    Lint,

    #[error("{message}")]
    InvalidOptions { message: String },

    #[error("{message}")]
    CoverageValidation { message: String },

    #[error("Unknown error")]
    Unknown {
        #[from]
        source: anyhow::Error,
    },
}

impl CommandError {
    pub fn err(message: &str) -> Result<CommandSuccess, Self> {
        Err(Self::new(message))
    }

    pub fn new(message: &str) -> Self {
        CommandError::Unknown {
            source: anyhow::anyhow!(message.to_owned()),
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            CommandError::InvalidOptions { .. } => 1,
            CommandError::Config => 2,
            CommandError::Lint => 3,
            CommandError::CoverageValidation { message: _ } => 4,
            CommandError::Unknown { .. } => 99,
        }
    }
}

impl From<ureq::Error> for CommandError {
    fn from(err: ureq::Error) -> CommandError {
        CommandError::Unknown { source: err.into() }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> CommandError {
        CommandError::Unknown { source: err.into() }
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(error: serde_json::Error) -> Self {
        CommandError::Unknown {
            source: error.into(),
        }
    }
}

impl From<git2::Error> for CommandError {
    fn from(error: git2::Error) -> Self {
        CommandError::Unknown {
            source: error.into(),
        }
    }
}
