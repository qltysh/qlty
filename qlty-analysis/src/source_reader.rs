use std::fmt::Debug;
use std::{collections::HashMap, path::PathBuf, sync::RwLock};

pub trait SourceReader: Debug + Send + Sync {
    fn read(&self, path: PathBuf) -> std::io::Result<String>;
    fn write(&self, path: PathBuf, content: String) -> std::io::Result<()>;
}

#[derive(Debug, Default)]
pub struct SourceReaderFs {
    cache: RwLock<HashMap<PathBuf, String>>,
}

impl Clone for SourceReaderFs {
    fn clone(&self) -> Self {
        Self {
            cache: RwLock::new(self.cache.read().unwrap().clone()),
        }
    }
}

impl SourceReaderFs {
    pub fn with_cache(cache: HashMap<PathBuf, String>) -> Self {
        Self {
            cache: RwLock::new(cache),
        }
    }

    fn get_cache(&self, path: &PathBuf) -> Option<String> {
        self.cache.read().unwrap().get(path).cloned()
    }
}

impl SourceReader for SourceReaderFs {
    fn read(&self, path: PathBuf) -> std::io::Result<String> {
        match self.get_cache(&path) {
            Some(content) => Ok(content),
            None => {
                let content = std::fs::read_to_string(&path)?;
                self.cache
                    .write()
                    .unwrap()
                    .insert(path.clone(), content.clone());
                Ok(content)
            }
        }
    }

    fn write(&self, path: PathBuf, content: String) -> std::io::Result<()> {
        let mut cache = self.cache.write().unwrap();
        std::fs::write(path.clone(), &content)?;
        cache.insert(path.clone(), content);
        Ok(())
    }
}

pub fn offset_to_location(s: &str, pos: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (i, c) in s.char_indices() {
        if i == pos {
            break;
        }
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

pub fn location_to_offset(s: &str, line: usize, column: usize) -> usize {
    if line == 1 {
        return column - 1;
    }

    let mut current_line = 1;
    let mut current_column = 1;
    for (i, c) in s.char_indices() {
        if current_line == line && current_column == column {
            return i;
        }
        if c == '\n' {
            current_line += 1;
            current_column = 1;
        } else {
            current_column += 1;
        }
    }
    s.len()
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_source_reader_fs() {
        let reader = SourceReaderFs::default();
        let content = reader.read(PathBuf::from("Cargo.toml")).unwrap();
        assert!(content.contains("[package]"));
    }

    #[test]
    fn test_source_reader_fs_with_cache() {
        let reader =
            SourceReaderFs::with_cache(HashMap::from([("Cargo.toml".into(), "content".into())]));
        let content = reader.read(PathBuf::from("Cargo.toml")).unwrap();
        assert_eq!(content, "content");
    }

    #[test]
    fn test_source_reader_fs_readwrite() {
        let reader = SourceReaderFs::default();
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");

        std::fs::write(path.clone(), "content").ok();
        assert_eq!(reader.read(path.clone()).ok().unwrap(), "content");

        std::fs::write(path.clone(), "not_content").ok();
        assert_eq!(reader.read(path.clone()).ok().unwrap(), "content");

        reader.write(path.clone(), "other".into()).ok();
        assert_eq!(reader.read(path.clone()).ok().unwrap(), "other");
    }

    #[test]
    fn test_location_for_pos() {
        let data = indoc::indoc! {"
            const x = 1;
            const y = array.map((item) => {
              return item + 1;
            });
        "};
        let item_pos = data.match_indices("item").last().unwrap().0;
        assert_eq!(offset_to_location(data, item_pos), (3, 10));
        assert_eq!(location_to_offset(data, 3, 10), item_pos);

        assert_eq!(location_to_offset(data, 1, 1), 0);
        assert_eq!(offset_to_location(data, data.len()), (5, 1));
        assert_eq!(location_to_offset(data, 5, 1), data.len());
    }
}
