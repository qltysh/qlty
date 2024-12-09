use crate::sources::{DefaultSource, GitSource, GitSourceReference, LocalSource, Source};
use crate::Library;
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SourceDef {
    #[serde(default)]
    pub default: Option<bool>,

    #[serde(default)]
    pub directory: Option<PathBuf>,

    #[serde(default)]
    pub repository: Option<String>,

    #[serde(default, rename = "ref")]
    pub reference: Option<String>,

    #[serde(default)]
    pub branch: Option<String>,

    #[serde(default)]
    pub tag: Option<String>,

    #[serde(default)]
    pub name: Option<String>,
}

impl SourceDef {
    pub fn source(&self, library: &Library) -> Result<Box<dyn Source>> {
        // If both a repository and directory are defined, we'll use the directory.
        // This allows for a local override of a repository.
        if self.default.unwrap_or_default() {
            Ok(Box::new(DefaultSource {}))
        } else if self.directory.is_some() {
            Ok(Box::new(LocalSource {
                root: self.directory.clone().unwrap(),
            }))
        } else if self.repository.is_some() {
            if self.tag.is_some() && self.branch.is_some() {
                bail!("Source defines both a tag and branch");
            }

            if self.tag.is_none() && self.branch.is_none() {
                bail!("Source defines neither a tag nor branch");
            }

            let reference = if self.tag.is_some() {
                GitSourceReference::Tag(self.tag.clone().unwrap())
            } else {
                GitSourceReference::Branch(self.branch.clone().unwrap())
            };

            Ok(Box::new(GitSource {
                library: library.clone(),
                origin: self.repository.as_ref().unwrap().to_string(),
                reference,
            }))
        } else {
            Err(anyhow!(
                "Source is not the default and defines neither a repository or directory"
            ))
        }
    }
}
