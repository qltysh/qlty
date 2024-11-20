use super::{source::SourceFetch, Source};
use crate::Library;
use anyhow::{Context, Result};
use git2::{Remote, Repository, ResetType};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

#[cfg(unix)]
use std::os::unix::fs::symlink as symlink_dir;
#[cfg(windows)]
use std::os::windows::fs::symlink_dir;

#[derive(Debug, Clone)]
pub struct GitSource {
    pub library: Library,
    pub origin: String,
    pub reference: GitSourceReference,
}

#[derive(Clone, Debug)]
pub enum GitSourceReference {
    Branch(String),
    Tag(String),
}

impl Source for GitSource {
    fn local_root(&self) -> PathBuf {
        self.local_origin_ref_path(&self.library)
    }

    fn clone_box(&self) -> Box<dyn Source> {
        Box::new(self.clone())
    }
}

impl SourceFetch for GitSource {
    fn fetch(&self) -> Result<()> {
        let parent_dir = self.global_origin_path()?;

        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir)?;
        }

        let checkout_path = self.global_origin_ref_path()?;

        if checkout_path.exists() {
            self.update_checkout(&checkout_path)?;
        } else {
            info!("Creating source checkout {}", self.origin);
            if let Err(err) = self.create_checkout(&checkout_path) {
                std::fs::remove_dir_all(&checkout_path)?;
                return Err(err);
            }
        }

        self.symlink_if_needed()
    }

    fn clone_box(&self) -> Box<dyn SourceFetch> {
        Box::new(self.clone())
    }
}

impl GitSource {
    fn symlink_if_needed(&self) -> Result<()> {
        std::fs::create_dir_all(self.local_sources_path(&self.library))?;

        if !self.local_origin_path(&self.library).exists() {
            debug!(
                "Creating symlink from {:?} to {:?}",
                self.global_origin_path().unwrap().display(),
                self.local_origin_path(&self.library).display()
            );

            symlink_dir(
                self.global_origin_path()?,
                self.local_origin_path(&self.library),
            )
            .with_context(|| {
                format!(
                    "Failed to create symlink from {:?} to {:?}",
                    self.global_origin_path().unwrap().display(),
                    self.local_origin_path(&self.library).display()
                )
            })?;
        } else {
            debug!(
                "Symlink already exists: {:?}",
                self.local_origin_path(&self.library).display()
            );
        }

        Ok(())
    }

    fn create_checkout(&self, checkout_path: &Path) -> Result<()> {
        std::fs::create_dir_all(checkout_path)?;
        let repository = Repository::init(checkout_path)
            .with_context(|| format!("Failed to initialize repository at {:?}", checkout_path))?;

        self.set_origin(&repository, checkout_path, &[])?;

        match &self.reference {
            GitSourceReference::Branch(branch) => {
                let remote_branch = format!("refs/remotes/origin/{}", &branch);
                let branch_ref = repository.find_reference(&remote_branch)?;
                repository.reference(
                    &format!("refs/heads/{}", &branch),
                    branch_ref.target().unwrap(),
                    true,
                    "Creating branch from fetched remote",
                )?;
                repository.set_head(&format!("refs/heads/{}", &branch))?;
            }
            GitSourceReference::Tag(tag) => {
                repository.set_head_detached(
                    repository
                        .revparse_single(&format!("refs/tags/{}", tag))?
                        .id(),
                )?;
            }
        }

        let reference = repository.revparse_single("HEAD")?;
        repository
            .checkout_tree(&reference, None)
            .with_context(|| {
                format!(
                    "Failed to checkout reference {:?} in repository at {:?}",
                    self.reference, checkout_path
                )
            })?;

        Ok(())
    }

    fn update_checkout(&self, checkout_path: &Path) -> Result<()> {
        if let GitSourceReference::Branch(branch_name) = &self.reference {
            info!("Updating source checkout {}", self.origin);

            let repository = Repository::open(checkout_path).with_context(|| {
                format!("Error opening the source repository at {}\n\nTry removing the .qlty/sources directory", checkout_path.display())
            })?;

            self.set_origin(&repository, checkout_path, &[branch_name])?;

            let branch_name = format!("refs/remotes/origin/{}", branch_name);

            let reference = repository.find_reference(&branch_name).with_context(|| {
                format!(
                    "Failed to find {} in repository {}",
                    branch_name,
                    checkout_path.display()
                )
            })?;

            let latest_commit = reference.peel_to_commit().with_context(|| {
                format!(
                    "Failed to peel reference {} to commit in repository {}",
                    branch_name,
                    checkout_path.display()
                )
            })?;

            let latest_commit_object = latest_commit.as_object();
            repository
                .reset(latest_commit_object, ResetType::Hard, None)
                .with_context(|| {
                    format!(
                        "Failed to hard reset branch {} to remote {}",
                        branch_name, branch_name
                    )
                })?;
        } else {
            debug!(
                "Not updating checkout at {:?} because it's a tag",
                checkout_path
            );
        }

        // The repository's current branch isn't pointing to the branch specified by the `reference` field, we assume the repository is in a
        // detached HEAD state, meaning it's pointing to a specific commit, so we assume it's a tag.
        Ok(())
    }

    fn set_origin(
        &self,
        repository: &Repository,
        checkout_path: &Path,
        branches: &[&str],
    ) -> Result<()> {
        let mut origin = if let Ok(found_origin) = repository.find_remote("origin") {
            found_origin
        } else {
            repository.remote("origin", &self.origin).with_context(|| {
                format!(
                    "Failed to add remote origin {} to repository at {}",
                    self.origin,
                    checkout_path.display()
                )
            })?
        };

        self.fetch(&mut origin, branches)
    }

    fn fetch(&self, origin: &mut Remote, branches: &[&str]) -> Result<()> {
        // Per libgit2, passing an empty array of refspecs fetches base refspecs
        origin.fetch(branches, None, None).with_context(|| {
            if branches.is_empty() {
                format!(
                    "Failed to fetch base refspecs from remote origin {}",
                    self.origin
                )
            } else {
                format!(
                    "Failed to fetch branches {:?} from remote origin {}",
                    branches, self.origin
                )
            }
        })
    }

    fn global_origin_path(&self) -> Result<PathBuf> {
        Ok(Library::global_cache_root()?
            .join("sources")
            .join(self.origin_directory_name()))
    }

    fn global_origin_ref_path(&self) -> Result<PathBuf> {
        Ok(self
            .global_origin_path()?
            .join(self.reference_directory_name()))
    }

    fn local_sources_path(&self, library: &Library) -> PathBuf {
        library.local_root.join("sources")
    }

    fn local_origin_path(&self, library: &Library) -> PathBuf {
        self.local_sources_path(library)
            .join(self.origin_directory_name())
    }

    fn local_origin_ref_path(&self, library: &Library) -> PathBuf {
        self.local_origin_path(library)
            .join(self.reference_directory_name())
    }

    fn origin_directory_name(&self) -> String {
        let mut origin = self.origin.clone();
        origin = origin.replace(':', "-");
        origin = origin.replace('/', "-");
        origin = origin.replace('.', "-");
        origin = origin.replace('@', "-");
        origin
    }

    fn reference_directory_name(&self) -> String {
        match &self.reference {
            GitSourceReference::Branch(branch) => branch.to_string(),
            GitSourceReference::Tag(tag) => tag.to_string(),
        }
    }
}
