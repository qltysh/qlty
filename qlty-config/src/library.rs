use anyhow::Result;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tracing::{error, warn};
use walkdir::WalkDir;

const AUTO_PRUNE_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

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

    pub fn global_tmp_root() -> Result<PathBuf> {
        Ok(Self::global_root()?.join("tmp"))
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

        let global_tmp_root = Self::global_tmp_root()?;
        fs::create_dir_all(&global_tmp_root)?;

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&global_tmp_root)?.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&global_tmp_root, perms)?;
        }

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

    pub fn auto_prune(&self) {
        let result = self
            .cache_directory()
            .and_then(|cache_directory| Self::auto_prune_dirs(&cache_directory, &self.local_root));

        if let Err(err) = result {
            warn!("Failed to auto-prune cache: {:?}", err);
        }
    }

    // Prunes through the local .qlty paths, which are real directories when symlinking fails
    fn auto_prune_dirs(cache_directory: &Path, local_root: &Path) -> Result<()> {
        let marker = cache_directory.join(".last_prune");

        if let Ok(modified) = fs::metadata(&marker).and_then(|metadata| metadata.modified()) {
            if modified.elapsed().unwrap_or_default() < AUTO_PRUNE_INTERVAL {
                return Ok(());
            }
        }

        fs::create_dir_all(cache_directory)?;
        fs::File::create(&marker)?.set_modified(SystemTime::now())?;
        Self::prune_stale(local_root)
    }

    fn prune_stale(root: &Path) -> Result<()> {
        if root.join("logs").exists() {
            Self::prune_dir(&root.join("logs"), 7)?;
        }

        if root.join("out").exists() {
            Self::prune_dir(&root.join("out"), 3)?;
        }

        if root.join("results").join("issues").exists() {
            Self::prune_dir(&root.join("results").join("issues"), 1)?;
        }

        Ok(())
    }

    pub fn prune(&self) -> Result<()> {
        let cache_directory = self.cache_directory()?;

        Self::prune_stale(&cache_directory)?;

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

    fn prune_dir(dir: &Path, days: u32) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let metadata = match fs::metadata(&path) {
                    Ok(metadata) => metadata,
                    Err(err) if err.kind() == ErrorKind::NotFound => continue,
                    Err(err) => return Err(err.into()),
                };
                let usage_time = std::cmp::max(metadata.accessed()?, metadata.modified()?);

                if usage_time.elapsed().unwrap_or_default().as_secs() > days as u64 * 24 * 60 * 60 {
                    // Concurrent qlty runs may prune the same directory
                    match fs::remove_file(&path) {
                        Err(err) if err.kind() != ErrorKind::NotFound => return Err(err.into()),
                        _ => {}
                    }
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
        // Path::exists() follows symlinks, so a dangling symlink (e.g. one left
        // behind after the cache directory was deleted) reports false while still
        // blocking symlink creation with EEXIST. Check the link node itself and
        // remove it if it is a symlink whose target is gone.
        if let Ok(metadata) = link.symlink_metadata() {
            if metadata.file_type().is_symlink() && !link.exists() {
                fs::remove_file(link)?;
            } else {
                return Ok(());
            }
        }

        #[cfg(unix)]
        {
            if let Err(err) = std::os::unix::fs::symlink(target, link) {
                error!(
                    "Failed to create symlink from {} to {}: {}",
                    target.display(),
                    link.display(),
                    err
                );
            }
        }

        #[cfg(windows)]
        {
            if let Err(err) = std::os::windows::fs::symlink_dir(target, link) {
                error!(
                    "Failed to create symlink from {} to {}: {}",
                    target.display(),
                    link.display(),
                    err
                );
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

#[cfg(test)]
#[cfg(unix)]
mod test {
    use super::*;
    use std::fs::FileTimes;
    use tempfile::TempDir;

    fn write_file_aged(path: &Path, age: Duration) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
        let time = SystemTime::now() - age;
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_times(FileTimes::new().set_accessed(time).set_modified(time))
            .unwrap();
    }

    const FOUR_DAYS: Duration = Duration::from_secs(4 * 24 * 60 * 60);

    fn setup() -> (TempDir, Library, PathBuf, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let library = Library::new(temp_dir.path()).unwrap();

        let target = temp_dir.path().join("target");
        fs::create_dir_all(&target).unwrap();

        let link = temp_dir.path().join("link");

        (temp_dir, library, target, link)
    }

    #[test]
    fn try_symlink_creates_link_when_missing() {
        let (_temp_dir, library, target, link) = setup();

        library.try_symlink_if_missing(&target, &link).unwrap();

        assert_eq!(fs::read_link(&link).unwrap(), target);
    }

    #[test]
    fn try_symlink_replaces_dangling_link() {
        let (temp_dir, library, target, link) = setup();

        let missing_target = temp_dir.path().join("deleted-cache-dir");
        std::os::unix::fs::symlink(&missing_target, &link).unwrap();

        library.try_symlink_if_missing(&target, &link).unwrap();

        assert_eq!(fs::read_link(&link).unwrap(), target);
    }

    #[test]
    fn try_symlink_keeps_valid_link() {
        let (temp_dir, library, target, link) = setup();

        let other_target = temp_dir.path().join("other-target");
        fs::create_dir_all(&other_target).unwrap();
        std::os::unix::fs::symlink(&other_target, &link).unwrap();

        library.try_symlink_if_missing(&target, &link).unwrap();

        assert_eq!(fs::read_link(&link).unwrap(), other_target);
    }

    #[test]
    fn try_symlink_keeps_existing_directory() {
        let (_temp_dir, library, target, link) = setup();

        fs::create_dir_all(&link).unwrap();

        library.try_symlink_if_missing(&target, &link).unwrap();

        assert!(link.is_dir());
        assert!(fs::read_link(&link).is_err());
    }

    #[test]
    fn auto_prune_removes_stale_out_files() {
        let cache_dir = TempDir::new().unwrap();
        let stale = cache_dir.path().join("out").join("invoke-stale.yaml");
        write_file_aged(&stale, FOUR_DAYS);

        Library::auto_prune_dirs(cache_dir.path(), cache_dir.path()).unwrap();

        assert!(!stale.exists());
    }

    #[test]
    fn auto_prune_keeps_fresh_out_files() {
        let cache_dir = TempDir::new().unwrap();
        let fresh = cache_dir.path().join("out").join("invoke-fresh.yaml");
        write_file_aged(&fresh, Duration::ZERO);

        Library::auto_prune_dirs(cache_dir.path(), cache_dir.path()).unwrap();

        assert!(fresh.exists());
    }

    #[test]
    fn auto_prune_keeps_plugin_cachedir() {
        let cache_dir = TempDir::new().unwrap();
        let plugin_cache = cache_dir.path().join("plugin_cachedir").join("cache.bin");
        write_file_aged(&plugin_cache, FOUR_DAYS);

        Library::auto_prune_dirs(cache_dir.path(), cache_dir.path()).unwrap();

        assert!(plugin_cache.exists());
    }

    #[test]
    fn auto_prune_skips_when_recently_pruned() {
        let cache_dir = TempDir::new().unwrap();
        write_file_aged(&cache_dir.path().join(".last_prune"), Duration::ZERO);
        let stale = cache_dir.path().join("out").join("invoke-stale.yaml");
        write_file_aged(&stale, FOUR_DAYS);

        Library::auto_prune_dirs(cache_dir.path(), cache_dir.path()).unwrap();

        assert!(stale.exists());
    }

    #[test]
    fn auto_prune_runs_when_last_prune_expired() {
        let cache_dir = TempDir::new().unwrap();
        write_file_aged(
            &cache_dir.path().join(".last_prune"),
            Duration::from_secs(25 * 60 * 60),
        );
        let stale = cache_dir.path().join("out").join("invoke-stale.yaml");
        write_file_aged(&stale, FOUR_DAYS);

        Library::auto_prune_dirs(cache_dir.path(), cache_dir.path()).unwrap();

        assert!(!stale.exists());
    }

    #[test]
    fn auto_prune_refreshes_expired_marker() {
        let cache_dir = TempDir::new().unwrap();
        let marker = cache_dir.path().join(".last_prune");
        write_file_aged(&marker, Duration::from_secs(25 * 60 * 60));

        Library::auto_prune_dirs(cache_dir.path(), cache_dir.path()).unwrap();

        let age = fs::metadata(&marker)
            .unwrap()
            .modified()
            .unwrap()
            .elapsed()
            .unwrap();
        assert!(age < AUTO_PRUNE_INTERVAL);
    }

    #[test]
    fn auto_prune_removes_stale_files_under_local_root() {
        let cache_dir = TempDir::new().unwrap();
        let local_root = TempDir::new().unwrap();
        let stale = local_root.path().join("out").join("invoke-stale.yaml");
        write_file_aged(&stale, FOUR_DAYS);

        Library::auto_prune_dirs(cache_dir.path(), local_root.path()).unwrap();

        assert!(!stale.exists());
    }
}
