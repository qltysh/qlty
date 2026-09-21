//! The on-disk response cache. TypeSafe responses use slopdetect's layout so
//! its caches are reusable: `<cache_dir>/jev/<key>.json` holding
//! `{"key": key, "response": <raw provider body>}`. Vercel responses have a
//! different shape and live apart, under `<cache_dir>/jev-vercel/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{json, Value};

use super::JevProvider;
use crate::error::{Error, Result};

static TEMPORARY_FILES: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    pub fn new(cache_dir: &Path, provider: JevProvider) -> Self {
        let namespace = match provider {
            JevProvider::TypeSafe => "jev",
            JevProvider::Vercel => "jev-vercel",
        };
        Self {
            dir: cache_dir.join(namespace),
        }
    }

    /// The cached raw response for `key`, or `None` when there is none.
    pub fn read(&self, key: &str) -> Result<Option<Value>> {
        let path = self.path(key);
        if !path.exists() {
            return Ok(None);
        }
        let text = fs::read_to_string(&path).map_err(|_| unreadable(&path))?;
        let mut cached: Value = serde_json::from_str(&text).map_err(|_| unreadable(&path))?;
        if cached.get("key").and_then(Value::as_str) != Some(key) {
            return Err(Error::Cache("Jev cache identity mismatch.".to_owned()));
        }
        let Some(response) = cached.get_mut("response").map(Value::take) else {
            return Err(unreadable(&path));
        };
        Ok(Some(response))
    }

    /// Writes the raw response for `key` atomically: a temporary file in the
    /// same directory, then a rename.
    pub fn write(&self, key: &str, response: &Value) -> Result<()> {
        fs::create_dir_all(&self.dir)?;
        let path = self.path(key);
        let sequence = TEMPORARY_FILES.fetch_add(1, Ordering::Relaxed);
        let temporary = self
            .dir
            .join(format!("{key}.json.{}.{sequence}.tmp", process::id()));
        let document = json!({ "key": key, "response": response });
        let mut text = serde_json::to_string_pretty(&document)
            .map_err(|_| Error::Cache(format!("Cannot write JSON file: {}", path.display())))?;
        text.push('\n');
        let outcome = fs::write(&temporary, text).and_then(|()| fs::rename(&temporary, &path));
        if outcome.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        Ok(outcome?)
    }

    fn path(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}.json"))
    }
}

fn unreadable(path: &Path) -> Error {
    Error::Cache(format!("Cannot read JSON file: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn missing_entry_reads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), JevProvider::TypeSafe);
        assert_eq!(cache.read(KEY).unwrap(), None);
    }

    #[test]
    fn round_trips_a_response() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), JevProvider::TypeSafe);
        let response = json!({"model": "jev-1.13.0", "answers": {}, "x": "λ"});
        cache.write(KEY, &response).unwrap();
        assert_eq!(cache.read(KEY).unwrap(), Some(response));
    }

    #[test]
    fn writes_slopdetects_file_layout() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), JevProvider::TypeSafe);
        cache.write(KEY, &json!({"a": 1})).unwrap();
        let path = dir.path().join("jev").join(format!("{KEY}.json"));
        let document: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(document, json!({"key": KEY, "response": {"a": 1}}));
        assert_eq!(fs::read_dir(dir.path().join("jev")).unwrap().count(), 1);
    }

    #[test]
    fn rejects_an_entry_whose_key_does_not_match() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), JevProvider::TypeSafe);
        fs::create_dir_all(dir.path().join("jev")).unwrap();
        fs::write(
            dir.path().join("jev").join(format!("{KEY}.json")),
            r#"{"key": "other", "response": {}}"#,
        )
        .unwrap();
        assert_eq!(
            cache.read(KEY).unwrap_err().to_string(),
            "Jev cache identity mismatch."
        );
    }

    #[test]
    fn rejects_an_unparsable_entry() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), JevProvider::TypeSafe);
        fs::create_dir_all(dir.path().join("jev")).unwrap();
        fs::write(dir.path().join("jev").join(format!("{KEY}.json")), "nope").unwrap();
        assert!(matches!(cache.read(KEY), Err(Error::Cache(_))));
    }

    #[test]
    fn providers_use_separate_directories_for_the_same_request() {
        let dir = tempfile::tempdir().unwrap();
        Cache::new(dir.path(), JevProvider::TypeSafe)
            .write(KEY, &json!({"model": "jev-1.13.0"}))
            .unwrap();
        Cache::new(dir.path(), JevProvider::Vercel)
            .write(KEY, &json!({"answers": {}}))
            .unwrap();
        assert!(dir.path().join("jev").join(format!("{KEY}.json")).exists());
        assert!(dir
            .path()
            .join("jev-vercel")
            .join(format!("{KEY}.json"))
            .exists());
        assert_eq!(
            Cache::new(dir.path(), JevProvider::TypeSafe)
                .read(KEY)
                .unwrap(),
            Some(json!({"model": "jev-1.13.0"}))
        );
    }
}
