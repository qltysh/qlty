use crate::{
    config::Builder,
    sources::{SourceFetch, SourcesList},
    Library, QltyConfig,
};
use anyhow::{anyhow, Context, Result};
use git2::Repository;
use ignore::{Walk, WalkBuilder};
use std::path::{Path, PathBuf};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Workspace {
    pub root: PathBuf,
}

impl Workspace {
    pub fn require_initialized() -> Result<Self> {
        Self::assert_within_initialized_project()?;

        Ok(Self {
            root: Self::assert_within_git_directory()?,
        })
    }

    pub fn new() -> Result<Self> {
        Ok(Self {
            root: Self::assert_within_git_directory()?,
        })
    }

    pub fn for_root(root: &Path) -> Result<Self> {
        Ok(Self {
            root: root.to_owned(),
        })
    }

    pub fn repo(&self) -> Result<Repository> {
        Repository::open(&self.root).context("Unable to open git repository")
    }

    pub fn walker(&self) -> Walk {
        self.walk_builder().build()
    }

    pub fn walk_builder(&self) -> WalkBuilder {
        let mut builder = WalkBuilder::new(&self.root);
        builder.follow_links(false);
        builder.hidden(false);
        builder
    }

    pub fn fetch_sources(&self) -> Result<()> {
        self.sources_list()?.fetch()
    }

    pub fn config_exists(&self) -> Result<bool> {
        Ok(self.config_path()?.exists())
    }

    pub fn config(&self) -> Result<QltyConfig> {
        let config = Builder::full_config_for_workspace(self);

        if let Ok(config) = &config {
            config.print_deprecation_warnings();
        }

        config
    }

    pub fn sources_list(&self) -> Result<SourcesList> {
        Builder::sources_config(self)?.sources_list(&self.library()?)
    }

    pub fn library(&self) -> Result<Library> {
        Library::new(&self.root)
    }

    pub fn config_path(&self) -> Result<PathBuf> {
        Ok(self.library()?.qlty_config_path())
    }

    pub fn current_dir() -> PathBuf {
        let curdir = std::env::current_dir().expect("current dir");
        let canonical = curdir.canonicalize().unwrap_or(curdir);
        if cfg!(windows) {
            let path = canonical.to_string_lossy().to_string();
            PathBuf::from(path.strip_prefix(r"\\?\").unwrap_or(&path))
        } else {
            canonical
        }
    }

    pub fn assert_git_directory_root() -> Result<PathBuf> {
        let current = Self::current_dir();
        let git_repository = Self::closest_git_repository_path(&current);

        if git_repository.is_none() {
            return Err(anyhow!(
                "This must be run at the root of a Git repository. Current directory is not within a repository: {}",
                current.display()
            ));
        }

        if git_repository.as_ref().unwrap() != &current {
            return Err(anyhow!(
                "This must be run at the root of a Git repository. Current directory is not the repository root: {}",
                current.display()
            ));
        }

        Ok(git_repository.unwrap())
    }
    pub fn assert_within_initialized_project() -> Result<PathBuf> {
        let current = Self::current_dir();
        let git_repository = Self::closest_git_repository_path(&current);

        if git_repository.is_none() {
            return Err(anyhow!(
                "This must be run within a Git repository with Qlty set up."
            ));
        } else {
            let git_repository = git_repository.unwrap();
            let workspace = Self::for_root(&git_repository)?;

            if !workspace.config_exists()? {
                return Err(anyhow!(
                    "Qlty must be set up in this repository. Try: qlty init"
                ));
            }

            Ok(git_repository)
        }
    }

    pub fn assert_within_git_directory() -> Result<PathBuf> {
        let current = Self::current_dir();
        let git_repository = Self::closest_git_repository_path(&current);

        if git_repository.is_none() {
            return Err(anyhow!("This must be run within a Git repository."));
        }

        Ok(git_repository.unwrap())
    }

    pub fn closest_git_repository_path(current: &Path) -> Option<PathBuf> {
        let mut current = current.to_path_buf();

        loop {
            let git_dir = current.join(".git");

            if git_dir.exists() {
                return Some(current);
            }

            if !current.pop() {
                return None;
            }
        }
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
