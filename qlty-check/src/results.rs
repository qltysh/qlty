use crate::executor::{installation_error::InstallationError, InvocationResult};
use qlty_types::analysis::v1::{Issue, Location, Message};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Results {
    pub messages: Vec<Message>,
    pub invocations: Vec<InvocationResult>,
    pub issues: Vec<Issue>,
    pub formatted: Vec<PathBuf>,
    pub installation_errors: Vec<InstallationError>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct FixedResult {
    pub rule_key: String,
    pub location: Location,
}

impl Results {
    pub fn new(
        messages: Vec<Message>,
        invocations: Vec<InvocationResult>,
        issues: Vec<Issue>,
        formatted: Vec<PathBuf>,
        installation_errors: Vec<InstallationError>,
    ) -> Self {
        Self {
            messages,
            issues,
            formatted,
            invocations,
            installation_errors,
        }
    }
}
