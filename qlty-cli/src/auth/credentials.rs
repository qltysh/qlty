use anyhow::{Context, Result};
use keyring::Entry;
use std::sync::{Arc, LazyLock, Mutex};

pub const SERVICE: &str = "qlty-cli";
pub const DEFAULT_USER: &str = "default";

static ENTRY: LazyLock<Mutex<Option<Arc<Entry>>>> = LazyLock::new(|| Mutex::new(None));

pub fn read_token() -> Result<String> {
    entry()?
        .get_password()
        .with_context(|| "Failed to get access token".to_string())
}

pub fn write_token(token: &str) -> Result<()> {
    entry()?
        .set_password(token)
        .with_context(|| "Failed to set access token".to_string())
}

pub fn delete_token() -> Result<()> {
    entry()?
        .delete_credential()
        .with_context(|| "Failed to delete access token".to_string())
}

#[allow(dead_code)]
pub fn set_mock_entry(entry: Arc<Entry>) {
    *ENTRY.lock().unwrap() = Some(entry);
}

fn entry() -> Result<Arc<Entry>> {
    let mut guard = ENTRY.lock().unwrap();
    match &*guard {
        Some(entry) => Ok(entry.clone()),
        None => {
            let entry = Arc::new(Entry::new(SERVICE, DEFAULT_USER).with_context(|| {
                format!(
                    "Failed to create keyring entry for service '{}' and user '{}'",
                    SERVICE, DEFAULT_USER
                )
            })?);
            guard.replace(entry.clone());
            Ok(entry)
        }
    }
}
