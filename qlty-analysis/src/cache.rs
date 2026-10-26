use anyhow::Result;
use std::{collections::HashMap, fmt::Debug, fs, path::PathBuf};
use tracing::{debug, error, trace};

#[derive(Debug, Clone, Default)]
pub struct HashDigest {
    pub parts: HashMap<String, String>,
    pub digest: Option<md5::Digest>,
}

pub trait CacheKey {
    fn hexdigest(&self) -> String;
}

impl HashDigest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, key: &str, value: &str) {
        match self.digest {
            Some(_) => panic!("Cannot add to finalized cache key"),
            None => self.parts.insert(key.to_string(), value.to_string()),
        };
    }

    pub fn finalize(&mut self) {
        let mut context = md5::Context::new();

        let sorted_keys = {
            let mut keys: Vec<&String> = self.parts.keys().collect();
            keys.sort();
            keys
        };

        for key in sorted_keys {
            let value = self.parts.get(key).unwrap();

            context.consume(key.as_bytes());
            context.consume(value.as_bytes());
        }

        self.digest = Some(context.compute());
    }
}

impl CacheKey for HashDigest {
    fn hexdigest(&self) -> String {
        match &self.digest {
            Some(digest) => format!("{:x}", digest),
            None => panic!("Cannot get hexdigest of unfinalized cache key"),
        }
    }
}

pub trait Cache: Debug + Send + Sync + 'static {
    fn read(&self, key: &dyn CacheKey) -> Result<Option<Vec<u8>>>;
    fn write(&self, key: &dyn CacheKey, value: &[u8]) -> Result<()>;
    fn path(&self, key: &dyn CacheKey) -> PathBuf;
    fn clear(&self) -> Result<()>;
    fn clone_box(&self) -> Box<dyn Cache>;
}

impl Clone for Box<dyn Cache> {
    fn clone(&self) -> Box<dyn Cache> {
        self.clone_box()
    }
}

#[derive(Debug, Clone, Default)]
pub struct NullCache {}

impl Cache for NullCache {
    fn read(&self, _key: &dyn CacheKey) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    fn write(&self, _key: &dyn CacheKey, _value: &[u8]) -> Result<()> {
        Ok(())
    }

    fn path(&self, _key: &dyn CacheKey) -> PathBuf {
        PathBuf::new()
    }

    fn clear(&self) -> Result<()> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Cache> {
        Box::new(self.clone())
    }
}

impl NullCache {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct FilesystemCache {
    pub root: PathBuf,
    pub extension: String,
}

impl FilesystemCache {
    pub fn new(root: PathBuf, extension: &str) -> Self {
        Self {
            root,
            extension: extension.to_string(),
        }
    }
}

impl Cache for FilesystemCache {
    fn read(&self, key: &dyn CacheKey) -> Result<Option<Vec<u8>>> {
        let path = self.path(key);

        trace!("FilesystemCache read: {}", path.display());
        let result = fs::read(path);

        match result {
            Ok(contents) => {
                debug!("Cache hit: {:?}", &key.hexdigest());
                Ok(Some(contents))
            }
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => Ok(None),
                _ => Err(error.into()),
            },
        }
    }

    fn write(&self, key: &dyn CacheKey, value: &[u8]) -> Result<()> {
        let path = self.path(key);
        let directory = path.parent();

        if directory.is_some()
            && !directory.unwrap().exists()
            && std::fs::create_dir_all(directory.unwrap()).is_err()
        {
            error!(
                "Failed to create directory: {}",
                directory.unwrap().display()
            );
        }

        trace!(
            "FilesystemCache write: {} ({} bytes)",
            path.display(),
            value.len()
        );
        fs::write(path, value)?;
        Ok(())
    }

    #[allow(unused)]
    fn clear(&self) -> Result<()> {
        fs::remove_dir_all(&self.root)?;
        Ok(())
    }

    fn path(&self, key: &dyn CacheKey) -> PathBuf {
        let mut path = self.root.clone();
        path.push(key.hexdigest());
        path.set_extension(&self.extension);
        path
    }

    fn clone_box(&self) -> Box<dyn Cache> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cache_key_equal() {
        let mut key_a = HashDigest::new();
        key_a.add("foo", "bar");
        key_a.finalize();

        let mut key_b = HashDigest::new();
        key_b.add("foo", "bar");
        key_b.finalize();

        assert_eq!(key_a.hexdigest(), key_b.hexdigest());
    }

    #[test]
    fn cache_key_not_equal() {
        let mut key_a = HashDigest::new();
        key_a.add("foo", "bar");
        key_a.finalize();

        let mut key_b = HashDigest::new();
        key_b.add("foo", "bar");
        key_b.add("baz", "bop");
        key_b.finalize();

        assert_ne!(key_a.hexdigest(), key_b.hexdigest());
    }

    #[test]
    fn filesystem_cache() {
        let tmpdir = tempfile::tempdir().unwrap();
        let cache = FilesystemCache::new(tmpdir.keep(), "bytes");

        let mut digest = HashDigest::new();
        digest.add("foo", "bar");
        digest.finalize();

        cache.write(&digest, "Test 123".as_bytes()).unwrap();
        assert_eq!("Test 123".as_bytes(), cache.read(&digest).unwrap().unwrap());
    }
}
