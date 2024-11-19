use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, TimestampSeconds};
use std::{fs::File, io::Write, path::PathBuf, time::SystemTime};

use crate::Library;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub version: String,

    pub openai_api_key: Option<String>,

    #[serde_as(as = "TimestampSeconds<i64>")]
    pub version_checked_at: SystemTime,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            version: "0".to_string(),
            openai_api_key: None,
            version_checked_at: SystemTime::UNIX_EPOCH,
        }
    }
}

impl UserData {
    pub fn create_or_load() -> Result<Self> {
        if Self::exists()? {
            Self::load()
        } else {
            Self::create()
        }
    }

    pub fn touch_version_checked_at(&mut self) -> Result<()> {
        self.version_checked_at = SystemTime::now();
        self.save()
    }

    fn exists() -> Result<bool> {
        let path = Self::path()?;
        Ok(path.exists())
    }

    fn create() -> Result<Self> {
        std::fs::create_dir_all(Library::global_cache_root()?)?;
        let settings = Self::default();
        settings.save()?;
        Ok(settings)
    }

    fn load() -> Result<Self> {
        let path = Self::path()?;
        let contents = std::fs::read_to_string(path)?;
        let settings = serde_yaml::from_str(&contents)?;
        Ok(settings)
    }

    fn save(&self) -> Result<()> {
        let yaml = serde_yaml::to_string(&self)?;
        let mut file = File::create(Self::path()?)?;
        file.write_all(yaml.as_bytes())?;
        Ok(())
    }

    fn path() -> Result<PathBuf> {
        Ok(Library::global_cache_root()?.join(Self::filename()))
    }

    fn filename() -> PathBuf {
        PathBuf::from("user.yaml")
    }
}
