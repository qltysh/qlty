//! Opt-in PHP packages for the Ubuntu runner. Never installs system packages or
//! executes their maintainer scripts; the image supplies shared libraries.
use crate::{tool::Tool, ui::ProgressTask};
use anyhow::{bail, ensure, Context, Result};
use duct::cmd;
use fslock::LockFile;
use once_cell::sync::Lazy;
use qlty_config::Library;
use semver::Version;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

const SOURCES: &str = "/opt/qlty/php-apt/sources.list";
static UPDATED: Lazy<Mutex<HashSet<PathBuf>>> = Lazy::new(Mutex::default);

#[derive(Debug, Clone)]
pub struct AptPhp {
    pub version: String,
    pub debian_version: String,
    minor: String,
    architecture: String,
    source: String,
    state: PathBuf,
}

impl AptPhp {
    pub fn resolve(requested: &str) -> Result<Self> {
        let minor = requested_minor(requested)?;
        ensure!(cfg!(target_os = "linux"), "Rootless APT PHP is supported only on the Ubuntu 24.04 runner; disable QLTY_FEATURE_PHP_APT_INSTALL to use PHP from PATH elsewhere");
        let release = fs::read_to_string("/etc/os-release")?;
        ensure!(
            release
                .lines()
                .any(|line| line == "ID=ubuntu" || line == "ID=\"ubuntu\"")
                && release
                    .lines()
                    .any(|line| line == "VERSION_ID=\"24.04\"" || line == "VERSION_ID=24.04"),
            "Rootless APT PHP requires Ubuntu 24.04; disable QLTY_FEATURE_PHP_APT_INSTALL to use PHP from PATH on this machine"
        );
        let architecture = match std::env::consts::ARCH {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            other => bail!("Unsupported architecture for rootless APT PHP: {other}"),
        }
        .to_owned();
        let source = fs::read_to_string(SOURCES).with_context(|| {
            format!("The runner must provision the signed PHP package source at {SOURCES}")
        })?;
        let state = Library::global_cache_root()?.join("php-apt");
        let mut runtime = Self {
            version: String::new(),
            debian_version: String::new(),
            minor,
            architecture,
            source,
            state,
        };
        runtime.update()?;
        let package = format!("php{}-cli", runtime.minor);
        let mut args = runtime.apt_options();
        args.extend(["madison".to_owned(), package]);
        let output = cmd("apt-cache", args)
            .env("LC_ALL", "C")
            .read()
            .context("Failed to list available PHP packages")?;
        let (version, debian_version) = select_version(requested, &output)?;
        runtime.version = version;
        runtime.debian_version = debian_version;
        Ok(runtime)
    }

    fn apt_options(&self) -> Vec<String> {
        [
            format!("Dir::Etc::sourcelist={SOURCES}"),
            "Dir::Etc::sourceparts=-".to_owned(),
            format!("Dir::State::lists={}", self.state.join("lists").display()),
            format!("Dir::Cache={}", self.state.join("cache").display()),
            "Acquire::Retries=2".to_owned(),
            "Acquire::http::Timeout=30".to_owned(),
            "Acquire::https::Timeout=30".to_owned(),
            "APT::Update::Error-Mode=any".to_owned(),
        ]
        .into_iter()
        .flat_map(|option| ["-o".to_owned(), option])
        .collect()
    }

    fn update(&self) -> Result<()> {
        let mut updated = UPDATED
            .lock()
            .map_err(|_| anyhow::anyhow!("PHP APT metadata lock poisoned"))?;
        if updated.contains(&self.state) {
            return Ok(());
        }
        fs::create_dir_all(self.state.join("lists/partial"))?;
        fs::create_dir_all(self.state.join("cache/archives/partial"))?;
        let mut lock = LockFile::open(&self.state.join("update.lock"))?;
        lock.lock()?;
        let mut args = self.apt_options();
        args.push("update".to_owned());
        cmd("apt-get", args)
            .env("LC_ALL", "C")
            .stderr_to_stdout()
            .read()
            .context("Failed to refresh signed PHP package metadata in the user cache")?;
        updated.insert(self.state.clone());
        Ok(())
    }

    pub fn update_hash(&self, sha: &mut Sha256) {
        // Bump the profile when changing packages, configuration, or image ABI.
        for value in [
            "php-apt-ubuntu24.04-v1",
            &self.architecture,
            &self.debian_version,
            &self.source,
        ] {
            sha.update(value.as_bytes());
            sha.update([0]);
        }
    }

    fn packages(&self) -> Vec<String> {
        let mut suffixes = vec!["cli", "common", "mbstring", "xml", "readline"];
        if Version::parse(&self.version).unwrap() < Version::new(8, 5, 0) {
            suffixes.push("opcache");
        }
        if self.minor == "7.4" {
            suffixes.push("json");
        }
        suffixes
            .into_iter()
            .map(|suffix| format!("php{}-{suffix}={}", self.minor, self.debian_version))
            .collect()
    }

    pub fn install(&self, tool: &super::Php, _task: &ProgressTask) -> Result<()> {
        let directory = PathBuf::from(tool.directory());
        let mut args = self.apt_options();
        args.push("download".to_owned());
        args.extend(self.packages());
        // APT verifies downloads against its signed repository metadata.
        tool.run_command(cmd("apt-get", args))
            .context("Failed to download the selected PHP packages; the repository may have superseded this version. Choose an available version")?;
        let root = directory.join("root");
        fs::create_dir_all(&root)?;
        for file in fs::read_dir(&directory)? {
            let path = file?.path();
            if path.extension().is_some_and(|ext| ext == "deb") {
                tool.run_command(cmd(
                    "dpkg-deb",
                    ["--extract".as_ref(), path.as_os_str(), root.as_os_str()],
                ))?;
                fs::remove_file(path)?;
            }
        }
        configure(&directory, &self.minor)?;
        #[cfg(unix)]
        {
            let bin = directory.join("bin");
            fs::create_dir_all(&bin)?;
            let link = bin.join("php");
            if link.symlink_metadata().is_ok() {
                fs::remove_file(&link)?;
            }
            std::os::unix::fs::symlink(format!("../root/usr/bin/php{}", self.minor), link)?;
        }
        tool.run_command(cmd("php", ["-r", "foreach (['ctype','iconv','phar','tokenizer','mbstring','dom','SimpleXML','xmlwriter','openssl','json'] as $extension) { if (!extension_loaded($extension)) { fwrite(STDERR, 'Missing PHP extension: '.$extension.PHP_EOL); exit(1); } }"]))
            .context("The extracted PHP runtime could not load its required extensions. The runner image must provide the matching shared libraries")?;
        fs::write(directory.join("apt-version"), &self.debian_version)?;
        Ok(())
    }
}

fn requested_minor(requested: &str) -> Result<String> {
    let parts: Vec<_> = requested.split('.').collect();
    ensure!(
        (parts.len() == 2 || parts.len() == 3)
            && parts.iter().all(|part| !part.is_empty()
                && part.bytes().all(|b| b.is_ascii_digit())
                && (part.len() == 1 || !part.starts_with('0'))),
        "PHP version must be major.minor or major.minor.patch (for example, 8.4 or 8.4.27)"
    );
    let minor = format!("{}.{}", parts[0], parts[1]);
    let parsed = Version::parse(&format!("{minor}.0"))?;
    ensure!(
        parsed >= Version::new(7, 4, 0),
        "Rootless APT PHP supports PHP 7.4 and newer"
    );
    if parts.len() == 3 {
        Version::parse(requested)?;
    }
    Ok(minor)
}

fn select_version(requested: &str, output: &str) -> Result<(String, String)> {
    let minor = requested_minor(requested)?;
    let exact = requested.split('.').count() == 3;
    let mut available = Vec::new();
    for line in output.lines() {
        let Some(debian) = line.split('|').nth(1).map(str::trim) else {
            continue;
        };
        let upstream = debian
            .split_once(':')
            .map_or(debian, |(_, version)| version)
            .split('-')
            .next()
            .unwrap_or("");
        if let Ok(version) = Version::parse(upstream) {
            if version.pre.is_empty()
                && version.build.is_empty()
                && format!("{}.{}", version.major, version.minor) == minor
            {
                available.push((version, debian.to_owned()));
            }
        }
    }
    // apt-cache madison already orders Debian revisions; keep that ordering for
    // equal upstream versions, including revisions carrying security backports.
    available.sort_by(|a, b| b.0.cmp(&a.0));
    if let Some((version, debian)) = available
        .iter()
        .find(|(version, _)| !exact || version.to_string() == requested)
    {
        return Ok((version.to_string(), debian.clone()));
    }
    let versions: Vec<_> = available
        .iter()
        .map(|(version, _)| version.to_string())
        .collect();
    bail!("PHP {requested} is not available from the runner's APT source. Available versions: {}. Choose one of these versions; old patches may have been removed by the repository", if versions.is_empty() { "none".to_owned() } else { versions.join(", ") })
}

fn configure(directory: &Path, minor: &str) -> Result<()> {
    let root = directory.join("root");
    let extension_dir = fs::read_dir(root.join("usr/lib/php"))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .find(|path| path.join("tokenizer.so").is_file())
        .context("Downloaded PHP common package has no tokenizer extension")?;
    let scan = directory.join("conf.d");
    fs::create_dir_all(&scan)?;
    for package in fs::read_dir(root.join("usr/share"))? {
        let package = package?;
        if !package
            .file_name()
            .to_string_lossy()
            .starts_with(&format!("php{minor}-"))
            || !package.file_type()?.is_dir()
        {
            continue;
        }
        for group in fs::read_dir(package.path())? {
            let group = group?;
            if !group.file_type()?.is_dir() {
                continue;
            }
            for ini in fs::read_dir(group.path())? {
                let ini = ini?;
                if ini.path().extension().is_none_or(|ext| ext != "ini") {
                    continue;
                }
                let contents = fs::read_to_string(ini.path())?;
                let priority = contents
                    .lines()
                    .find_map(|line| {
                        line.trim()
                            .strip_prefix("; priority=")
                            .and_then(|value| value.trim().parse::<u8>().ok())
                    })
                    .unwrap_or(20);
                fs::write(
                    scan.join(format!(
                        "{priority:02}-{}",
                        ini.file_name().to_string_lossy()
                    )),
                    contents,
                )?;
            }
        }
    }
    let escaped = extension_dir
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    fs::write(
        directory.join("php.ini"),
        format!("short_open_tag=Off\nextension_dir=\"{escaped}\"\n"),
    )?;
    Ok(())
}

#[cfg(test)]
pub(super) fn test_runtime(version: &str, revision: &str) -> AptPhp {
    AptPhp {
        version: version.into(),
        debian_version: format!("{version}-{revision}+ubuntu24.04.1"),
        minor: requested_minor(version).unwrap(),
        architecture: "arm64".into(),
        source: "signed test source".into(),
        state: PathBuf::from("/unused"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_latest_patch_and_keeps_debian_epoch_and_revision() {
        let listing = "php7.4-cli | 1:7.4.33-31+ubuntu24.04.1+deb.sury.org+1 | ppa\nphp7.4-cli | 1:7.4.32-1 | ppa";
        assert_eq!(
            select_version("7.4", listing).unwrap(),
            (
                "7.4.33".into(),
                "1:7.4.33-31+ubuntu24.04.1+deb.sury.org+1".into()
            )
        );
    }

    #[test]
    fn unavailable_exact_version_does_not_fall_back() {
        let err = select_version("8.4.18", "php8.4-cli | 8.4.27-1 | ppa")
            .unwrap_err()
            .to_string();
        assert!(err.contains("8.4.18"));
        assert!(err.contains("8.4.27"));
        assert_eq!(
            select_version(
                "8.4.18",
                "php8.4-cli | 8.4.27-1 | ppa\nphp8.4-cli | 8.4.18-2 | ppa"
            )
            .unwrap()
            .0,
            "8.4.18"
        );
    }

    #[test]
    fn rejects_invalid_version_arguments() {
        for version in [
            "system",
            "8",
            "8.4;id",
            "--help",
            "8.4.*",
            "8.4.1-rc1",
            "08.4",
            "8.4/../../",
            "7.3",
        ] {
            assert!(requested_minor(version).is_err(), "{version}");
        }
    }

    #[test]
    fn generates_private_configuration_with_module_priorities() {
        let dir = tempfile::tempdir().unwrap();
        // Exercise backslash escaping on Unix as well as Windows.
        let directory = dir.path().join(r"php\cache with spaces");
        let root = directory.join("root");
        fs::create_dir_all(root.join("usr/lib/php/20240924")).unwrap();
        fs::write(root.join("usr/lib/php/20240924/tokenizer.so"), "").unwrap();
        let modules = root.join("usr/share/php8.4-common/common");
        fs::create_dir_all(&modules).unwrap();
        fs::write(modules.join("pdo.ini"), "; priority=10\nextension=pdo.so\n").unwrap();
        fs::write(
            modules.join("tokenizer.ini"),
            "; priority=20\nextension=tokenizer.so\n",
        )
        .unwrap();
        configure(&directory, "8.4").unwrap();
        assert!(directory.join("conf.d/10-pdo.ini").is_file());
        assert!(directory.join("conf.d/20-tokenizer.ini").is_file());
        assert!(fs::read_to_string(directory.join("php.ini"))
            .unwrap()
            .contains(&format!(
                "extension_dir={:?}",
                root.join("usr/lib/php/20240924").to_string_lossy()
            )));
    }

    #[test]
    fn package_profile_accounts_for_json_and_builtin_opcache() {
        let php74 = test_runtime("7.4.33", "31").packages();
        assert!(php74
            .iter()
            .any(|package| package.starts_with("php7.4-json=")));
        assert!(php74
            .iter()
            .any(|package| package.starts_with("php7.4-opcache=")));
        let php85 = test_runtime("8.5.12", "1").packages();
        assert!(!php85.iter().any(|package| package.contains("-opcache=")));
        assert!(!php85.iter().any(|package| package.contains("-json=")));
    }
}
