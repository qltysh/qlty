use anyhow::Result;
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use tracing::error;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Library {
    pub local_root: PathBuf,
    pub tmp_dir: PathBuf,
}

pub struct FolderStatus {
    pub dir: PathBuf,
    pub files_count: usize,
    pub files_bytes: u64,
}

#[allow(unused)]
impl Library {
    pub fn global_root() -> Result<PathBuf> {
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"));
        let home = if cfg!(windows) {
            home.expect("USERPROFILE or HOME environment variable must be set")
        } else {
            home.expect("HOME environment variable must be set")
        };
        Ok(PathBuf::from(home).join(".qlty"))
    }

    pub fn global_logs_root() -> Result<PathBuf> {
        Ok(Self::global_root()?.join("logs"))
    }

    pub fn global_cache_root() -> Result<PathBuf> {
        Ok(Self::global_root()?.join("cache"))
    }

    pub fn new(workspace_root: &Path) -> Result<Self> {
        Ok(Self {
            local_root: workspace_root.join(".qlty"),
            tmp_dir: env::temp_dir().join("qlty"),
        })
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.local_root.join("logs")
    }

    pub fn out_dir(&self) -> PathBuf {
        self.local_root.join("out")
    }

    pub fn results_dir(&self) -> PathBuf {
        self.local_root.join("results")
    }

    pub fn plugin_cachedir_dir(&self) -> PathBuf {
        self.local_root.join("plugin_cachedir")
    }

    pub fn configs_dir(&self) -> PathBuf {
        self.local_root.join("configs")
    }

    pub fn tmp_dir(&self) -> PathBuf {
        self.tmp_dir.clone()
    }

    pub fn qlty_config_path(&self) -> PathBuf {
        self.local_root.join("qlty.toml")
    }

    pub fn gitignore_path(&self) -> PathBuf {
        self.local_root.join(".gitignore")
    }

    pub fn status(&self) -> Result<Vec<FolderStatus>> {
        let mut statuses = vec![];

        for dir in self.status_dirs()? {
            let mut files_count = 0;
            let mut files_bytes = 0;

            if dir.exists() {
                for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() {
                        let path = entry.path();

                        if path.is_file() {
                            files_count += 1;
                            files_bytes += fs::metadata(path)?.len();
                        }
                    }
                }
            }

            statuses.push(FolderStatus {
                dir,
                files_count,
                files_bytes,
            });
        }

        Ok(statuses)
    }

    pub fn create(&self) -> Result<()> {
        self.create_global()?;
        self.create_local()?;
        Ok(())
    }

    fn create_global(&self) -> Result<()> {
        let global_cache_root = Self::global_cache_root()?;

        fs::create_dir_all(global_cache_root.join("sources"))?;
        fs::create_dir_all(global_cache_root.join("tools"))?;
        fs::create_dir_all(
            global_cache_root
                .join("repos")
                .join(self.local_fingerprint())
                .join("logs"),
        )?;
        fs::create_dir_all(
            global_cache_root
                .join("repos")
                .join(self.local_fingerprint())
                .join("out"),
        )?;
        fs::create_dir_all(
            global_cache_root
                .join("repos")
                .join(self.local_fingerprint())
                .join("results"),
        )?;
        fs::create_dir_all(
            global_cache_root
                .join("repos")
                .join(self.local_fingerprint())
                .join("plugin_cachedir"),
        )?;
        Ok(())
    }

    fn create_local(&self) -> Result<()> {
        fs::create_dir_all(self.local_root.join("configs"))?;
        fs::create_dir_all(self.local_root.join("sources"))?;

        let global_repo_path = self.cache_directory()?;

        self.try_symlink_if_missing(&global_repo_path.join("out"), &self.out_dir())?;
        self.try_symlink_if_missing(&global_repo_path.join("logs"), &self.logs_dir())?;
        self.try_symlink_if_missing(&global_repo_path.join("results"), &self.results_dir())?;
        self.try_symlink_if_missing(
            &global_repo_path.join("plugin_cachedir"),
            &self.plugin_cachedir_dir(),
        )?;

        Ok(())
    }

    pub fn cache_directory(&self) -> Result<PathBuf> {
        Ok(Self::global_cache_root()?
            .join("repos")
            .join(self.local_fingerprint()))
    }

    pub fn prune(&self) -> Result<()> {
        let cache_directory = self.cache_directory()?;

        if cache_directory.join("logs").exists() {
            self.prune_dir(&cache_directory.join("logs"), 7)?;
        }

        if cache_directory.join("out").exists() {
            self.prune_dir(&cache_directory.join("out"), 3)?;
        }

        if cache_directory.join("results").join("issues").exists() {
            self.prune_dir(&cache_directory.join("results").join("issues"), 1)?;
        }

        if cache_directory.join("plugin_cachedir").exists() {
            for entry in fs::read_dir(cache_directory.join("plugin_cachedir"))? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    fs::remove_file(&path)?;
                } else {
                    fs::remove_dir_all(&path)?;
                }
            }
        }

        Ok(())
    }

    fn prune_dir(&self, dir: &Path, days: u32) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let metadata = fs::metadata(&path)?;
                let usage_time = std::cmp::max(metadata.accessed()?, metadata.modified()?);

                if usage_time.elapsed()?.as_secs() > days as u64 * 24 * 60 * 60 {
                    fs::remove_file(&path)?;
                }
            }
        }

        Ok(())
    }

    pub fn clean(&self) -> Result<()> {
        for dir in self.status_dirs()? {
            if dir.exists() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_file() {
                        fs::remove_file(&path)?;
                    } else {
                        fs::remove_dir_all(&path)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn try_symlink_if_missing(&self, target: &Path, link: &Path) -> Result<()> {
        if !link.exists() {
            #[cfg(unix)]
            {
                if std::os::unix::fs::symlink(target, link).is_err() {
                    error!(
                        "Failed to create symlink from {} to {}",
                        target.display(),
                        link.display()
                    );
                }
            }

            #[cfg(windows)]
            {
                if std::os::windows::fs::symlink_dir(target, link).is_err() {
                    error!(
                        "Failed to create symlink from {} to {}",
                        target.display(),
                        link.display()
                    );
                }
            }
        }

        Ok(())
    }

    fn local_fingerprint(&self) -> String {
        let digest = md5::compute(self.local_root.to_string_lossy().as_bytes());
        format!("{:x}", digest)
    }

    fn status_dirs(&self) -> Result<Vec<PathBuf>> {
        let global_repo_path = self.cache_directory()?;
        let candidates = vec![
            global_repo_path.join("out"),
            global_repo_path.join("logs"),
            global_repo_path.join("results"),
            global_repo_path.join("plugin_cachedir"),
        ];

        let mut existing_dirs = vec![];

        for candidate in candidates {
            if candidate.exists() {
                existing_dirs.push(candidate);
            }
        }

        Ok(existing_dirs)
    }
}
