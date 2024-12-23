use anyhow::{Context, Result};
use keyring::Entry;

pub const SERVICE: &str = "qlty-cli";
pub const DEFAULT_USER: &str = "default";

pub struct Token {
    pub user: String,
}

impl Default for Token {
    fn default() -> Self {
        Self::new(DEFAULT_USER)
    }
}

impl Token {
    pub fn new(user: &str) -> Self {
        Self {
            user: user.to_owned(),
        }
    }

    pub fn get(&self) -> Result<String> {
        self.entry()?.get_password().with_context(|| {
            format!(
                "Failed to get access token for service '{}' and user '{}'",
                SERVICE, self.user
            )
        })
    }

    pub fn set(&self, token: &str) -> Result<()> {
        self.entry()?.set_password(token).with_context(|| {
            format!(
                "Failed to set access token for service '{}' and user '{}'",
                SERVICE, self.user
            )
        })
    }

    pub fn delete(&self) -> Result<()> {
        self.entry()?.delete_credential().with_context(|| {
            format!(
                "Failed to delete access token for service '{}' and user '{}'",
                SERVICE, self.user
            )
        })
    }

    fn entry(&self) -> Result<Entry> {
        Entry::new(SERVICE, &self.user).with_context(|| {
            format!(
                "Failed to create keyring entry for service '{}' and user '{}'",
                SERVICE, self.user
            )
        })
    }
}
