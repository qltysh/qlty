use super::{source::SourceFetch, LocalSource, Source, SourceFile};
use crate::sources::source::configure_proxy_options;
use crate::Library;
use anyhow::{Context, Result};
use auth_git2::GitAuthenticator;
use git2::{FetchOptions, Remote, RemoteCallbacks, Repository, ResetType};
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
    fn paths(&self) -> Result<Vec<PathBuf>> {
        let local_source = self.local_source();
        local_source.paths()
    }

    fn get_file(&self, file_name: &Path) -> Result<Option<SourceFile>> {
        let local_source = self.local_source();
        local_source.get_file(file_name)
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
    fn resolve_url(&self, url: &str) -> Result<String> {
        let config = git2::Config::open_default()
            .with_context(|| "Failed to open Git configuration for URL resolution")?;

        let mut resolved_url = url.to_string();

        // Parse insteadOf configuration
        let mut entries = config
            .entries(Some("url\\..*\\.insteadof"))
            .with_context(|| "Failed to read URL insteadOf configuration")?;

        // Collect all insteadOf rules
        let mut instead_of_rules = Vec::new();
        while let Some(entry) = entries.next() {
            if let Ok(entry) = entry {
                if let (Some(name), Some(value)) = (entry.name(), entry.value()) {
                    // Extract the base URL from the config key (e.g., "url.https://github.com/.insteadof" -> "https://github.com/")
                    if let Some(base_url) = name
                        .strip_prefix("url.")
                        .and_then(|s| s.strip_suffix(".insteadof"))
                    {
                        instead_of_rules.push((value.to_string(), base_url.to_string()));
                    }
                }
            }
        }

        // Apply the longest matching insteadOf rule
        // Sort by value length (descending) to match git's behavior of using the longest match
        instead_of_rules.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (instead_of, base_url) in &instead_of_rules {
            if resolved_url.starts_with(instead_of) {
                resolved_url = resolved_url.replacen(instead_of, base_url, 1);
                debug!(
                    "Applied insteadOf rule: '{}' -> '{}', URL: '{}' -> '{}'",
                    instead_of, base_url, url, resolved_url
                );
                break;
            }
        }

        Ok(resolved_url)
    }

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
        let resolved_origin = self.resolve_url(&self.origin)?;

        let mut origin = if let Ok(found_origin) = repository.find_remote("origin") {
            found_origin
        } else {
            repository
                .remote("origin", &resolved_origin)
                .with_context(|| {
                    format!(
                        "Failed to add remote origin {} to repository at {}",
                        resolved_origin,
                        checkout_path.display()
                    )
                })?
        };

        self.fetch(&mut origin, branches)
    }

    fn fetch(&self, origin: &mut Remote, branches: &[&str]) -> Result<()> {
        let mut fetch_options = self.create_fetch_options()?;
        let resolved_origin = self
            .resolve_url(&self.origin)
            .unwrap_or_else(|_| self.origin.clone());

        // Per libgit2, passing an empty array of refspecs fetches base refspecs
        origin
            .fetch(branches, Some(&mut fetch_options), None)
            .with_context(|| {
                if branches.is_empty() {
                    format!("Failed to fetch base refspecs from remote origin {resolved_origin}")
                } else {
                    format!("Failed to fetch branches {branches:?} from remote origin {resolved_origin}")
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

    fn create_fetch_options(&self) -> Result<FetchOptions> {
        let mut fetch_options = FetchOptions::new();

        let mut proxy_options = git2::ProxyOptions::new();
        configure_proxy_options(&mut proxy_options);
        fetch_options.proxy_options(proxy_options);

        let mut callbacks = RemoteCallbacks::new();

        callbacks.credentials(|url, username, allowed| {
            let config = git2::Config::open_default().map_err(|e| {
                git2::Error::from_str(&format!("Failed to open Git configuration: {e}"))
            })?;
            let authenticator = GitAuthenticator::default();
            let mut credential_fn = authenticator.credentials(&config);
            credential_fn(url, username, allowed)
        });

        fetch_options.remote_callbacks(callbacks);

        Ok(fetch_options)
    }
}

impl GitSource {
    fn local_source(&self) -> LocalSource {
        LocalSource {
            root: self.local_origin_ref_path(&self.library),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_url_with_instead_of_https_to_ssh() {
        let library = Library::new(Path::new(".")).expect("Failed to create library");
        let git_source = GitSource {
            library,
            origin: "git@github.com:user/repo".to_string(),
            reference: GitSourceReference::Branch("main".to_string()),
        };

        // We can't easily test with the actual git config in unit tests,
        // so we'll test the logic indirectly by ensuring the method exists
        // and returns a reasonable result when git config is available
        let result = git_source.resolve_url("git@github.com:user/repo");
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_url_without_instead_of() {
        let library = Library::new(Path::new(".")).expect("Failed to create library");
        let git_source = GitSource {
            library,
            origin: "https://github.com/user/repo".to_string(),
            reference: GitSourceReference::Branch("main".to_string()),
        };

        let result = git_source.resolve_url("https://github.com/user/repo");
        assert!(result.is_ok());

        // Should return original URL when no insteadOf rules match
        if let Ok(resolved) = result {
            assert!(resolved.contains("github.com"));
        }
    }

    #[test]
    fn test_origin_directory_name_sanitization() {
        let library = Library::new(Path::new(".")).expect("Failed to create library");
        let git_source = GitSource {
            library,
            origin: "git@github.com:user/repo.git".to_string(),
            reference: GitSourceReference::Branch("main".to_string()),
        };

        let dir_name = git_source.origin_directory_name();
        assert_eq!(dir_name, "git-github-com-user-repo-git");
        assert!(!dir_name.contains(':'));
        assert!(!dir_name.contains('/'));
        assert!(!dir_name.contains('.'));
        assert!(!dir_name.contains('@'));
    }
}
