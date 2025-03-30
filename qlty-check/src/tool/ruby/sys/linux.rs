use crate::{
    tool::{
        installations::{initialize_installation, write_to_file},
        ruby::PlatformRuby,
    },
    ui::{ProgressBar, ProgressTask},
    Tool,
};
use anyhow::{bail, Result};
use ar::Entry;
use chrono::Utc;
use qlty_analysis::{join_path_string, utils::fs::path_to_string};
use qlty_types::analysis::v1::Installation;
use std::{
    collections::HashMap,
    io::{BufReader, Cursor, Read},
    path::Path,
};
use tracing::debug;

#[cfg(target_arch = "x86_64")]
const ARCH: &str = "amd64";

#[cfg(target_arch = "aarch64")]
const ARCH: &str = "arm64";

const DEBIAN_DATA_TAR_XZ: &[u8; 11] = b"data.tar.xz";

#[derive(Debug, Clone, Default)]
pub struct RubyLinux {}

impl PlatformRuby for RubyLinux {
    fn post_install(&self, tool: &dyn Tool, task: &ProgressTask) -> Result<()> {
        task.set_message("Setting up Ruby on Linux");
        self.rewrite_binstubs(tool)?;
        self.install_dependency_deb(
            tool,
            "o/openssl/libssl1.1_1.1.1n-0+deb10u3",
            vec![
                DependencyFile::both("libssl.so.1.1"),
                DependencyFile::both("libcrypto.so.1.1"),
            ],
        )?;
        self.install_dependency_deb(
            tool,
            "libx/libxcrypt/libcrypt1_4.4.18-4",
            vec![DependencyFile::new("libcrypt.so.1.1.0", "libcrypt.so.1")],
        )?;

        Ok(())
    }

    fn extra_env_paths(&self, tool: &dyn Tool) -> Vec<String> {
        vec![join_path_string!(tool.directory(), "bin")]
    }

    fn extra_env_vars(&self, tool: &dyn Tool, env: &mut HashMap<String, String>) {
        self.insert_rubylib_env(tool, env);
        env.insert(
            "LD_LIBRARY_PATH".to_string(),
            join_path_string!(tool.directory(), "lib"),
        );
        env.insert(
            "PKG_CONFIG_PATH".to_string(),
            join_path_string!(tool.directory(), "lib", "pkgconfig"),
        );
    }

    fn platform_directory(&self, _tool: &dyn Tool) -> String {
        format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
    }
}

#[derive(Debug, Clone, Default)]
struct DependencyFile {
    source: String,
    dest: String,
}

impl DependencyFile {
    fn new(source: &str, dest: &str) -> Self {
        Self {
            source: source.to_string(),
            dest: dest.to_string(),
        }
    }

    fn both(source: &str) -> Self {
        Self {
            source: source.to_string(),
            dest: source.to_string(),
        }
    }
}

impl RubyLinux {
    fn install_dependency_deb(
        &self,
        tool: &dyn Tool,
        package: &str,
        extract_filenames: Vec<DependencyFile>,
    ) -> Result<()> {
        let url = format!(
            "https://ftp.debian.org/debian/pool/main/{}_{}.deb",
            package, ARCH
        );
        let mut installation = initialize_installation(tool);
        installation.download_url = Some(url.to_string());
        installation.download_file_type = Some(".deb".to_string());
        installation.download_binary_name = Some(package.to_string());

        let result = ureq::get(url.as_str()).call();
        Self::finalize_installation(&mut installation, &result);

        match result {
            Ok(response) => {
                self.extract_dependency_deb_archive(
                    response.into_reader(),
                    url,
                    tool.directory(),
                    extract_filenames,
                )?;
            }
            Err(err) => {
                bail!("Failed to download dependency: {}: {:?}", package, err);
            }
        }

        Ok(())
    }

    fn finalize_installation(
        installation: &mut Installation,
        result: &std::result::Result<ureq::Response, ureq::Error>,
    ) {
        installation.finished_at = Some(Utc::now().into());
        match result {
            Ok(_) => {
                installation.download_success = Some(true);
            }
            Err(err) => {
                installation.download_success = Some(false);
                installation.stderr = Some(format!("{:?}", err));
            }
        }

        write_to_file(installation);
    }

    fn extract_dependency_deb_archive(
        &self,
        reader: impl Read,
        source_url: String,
        directory: String,
        extract_filenames: Vec<DependencyFile>,
    ) -> Result<()> {
        let mut deb_archive = ar::Archive::new(reader);
        debug!("Downloaded {}: {:?}", source_url, deb_archive.variant());
        while let Some(entry) = deb_archive.next_entry() {
            self.try_extract_package_data(&directory, &extract_filenames, entry?)?;
        }

        let result = extract_filenames
            .iter()
            .all(|d| Path::new(&directory).join("lib").join(&d.dest).exists());

        if !result {
            bail!("Failed to extract OpenSSL libraries");
        }

        Ok(())
    }

    fn try_extract_package_data(
        &self,
        directory: &String,
        extract_filenames: &Vec<DependencyFile>,
        entry: Entry<impl Read>,
    ) -> Result<()> {
        if entry.header().identifier() != DEBIAN_DATA_TAR_XZ {
            return Ok(());
        }
        debug!("Found data.tar.xz in dependency, extracting...");

        // decompress xz
        let mut tar_data: Vec<u8> = Vec::new();
        let mut buf_reader = BufReader::new(entry);
        lzma_rs::xz_decompress(&mut buf_reader, &mut tar_data)
            .map_err(|e| anyhow::anyhow!("Failed to decompress XZ data: {}", e))?;
        let cursor = Cursor::new(tar_data);

        // extract matching extract_filenames from tar
        let mut data_archive = tar::Archive::new(cursor);
        for mut entry in data_archive.entries_with_seek()?.flatten() {
            let path = path_to_string(entry.path()?);

            // Extract the filename from the path
            let filename = match path.split('/').next_back() {
                Some(filename) => filename,
                None => bail!("Invalid path with no filename component: {}", path),
            };

            let filename_matches = extract_filenames
                .iter()
                .find(|name| name.source == filename)
                .cloned();
            if let Some(dep_file) = filename_matches {
                let dest = join_path_string!(directory, "lib", dep_file.dest);
                debug!("Extracting dependency: {:?} -> {:?}", path, dest);
                entry.unpack(dest)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        tool::{
            ruby::{sys::linux::RubyLinux, PlatformRuby},
            ToolType,
        },
        Tool,
    };

    use qlty_analysis::utils::fs::path_to_string;
    use std::{env::join_paths, path::PathBuf};
    use tempfile::TempDir;

    #[derive(Debug, Clone, Default)]
    struct TestTool {
        version: String,
        directory: PathBuf,
    }

    impl Tool for TestTool {
        fn name(&self) -> String {
            "test".to_string()
        }

        fn version(&self) -> Option<String> {
            Some(self.version.clone())
        }

        fn tool_type(&self) -> crate::tool::ToolType {
            ToolType::Runtime
        }

        fn version_command(&self) -> Option<String> {
            None
        }

        fn clone_box(&self) -> Box<dyn Tool> {
            Box::new(self.clone())
        }

        fn directory(&self) -> String {
            path_to_string(&self.directory)
        }
    }

    #[test]
    fn test_different_version_directory() {
        let tempdir = TempDir::new().unwrap();
        std::fs::create_dir_all(tempdir.path().join("lib/ruby/site_ruby/9.9.0+1")).unwrap();
        let tool = TestTool {
            directory: tempdir.path().to_path_buf(),
            version: "9.9.9".to_string(),
        };
        let mut env = std::collections::HashMap::new();
        let runtime = RubyLinux::default();
        let platform_dir = runtime.platform_directory(&tool);
        runtime.extra_env_vars(&tool, &mut env);
        assert_eq!(
            *env.get("PKG_CONFIG_PATH").unwrap(),
            format!("{}/lib/pkgconfig", path_to_string(tempdir.path()))
        );
        assert_eq!(
            *env.get("RUBYLIB").unwrap(),
            path_to_string(
                join_paths(vec![
                    tempdir.path().join("lib/ruby/site_ruby/9.9.0+1"),
                    tempdir
                        .path()
                        .join(format!("lib/ruby/site_ruby/9.9.0+1/{}", platform_dir)),
                    tempdir.path().join("lib/ruby/site_ruby"),
                    tempdir.path().join("lib/ruby/vendor_ruby/9.9.0"),
                    tempdir
                        .path()
                        .join(format!("lib/ruby/vendor_ruby/9.9.0/{}", platform_dir)),
                    tempdir.path().join("lib/ruby/vendor_ruby"),
                    tempdir.path().join("lib/ruby/9.9.0"),
                    tempdir
                        .path()
                        .join(format!("lib/ruby/9.9.0/{}", platform_dir)),
                    tempdir.path().join("lib/ruby/"),
                ])
                .unwrap()
            )
        );
        drop(tempdir);
    }

    #[test]
    fn test_extra_env_vars() {
        let tempdir = TempDir::new().unwrap();
        let tool = TestTool {
            directory: tempdir.path().to_path_buf(),
            version: "9.9.9".to_string(),
        };
        let mut env = std::collections::HashMap::new();
        let runtime = RubyLinux::default();
        runtime.extra_env_vars(&tool, &mut env);
        assert_eq!(
            *env.get("LD_LIBRARY_PATH").unwrap(),
            format!("{}/lib", path_to_string(tempdir.path()))
        );
        assert_eq!(
            *env.get("PKG_CONFIG_PATH").unwrap(),
            format!("{}/lib/pkgconfig", path_to_string(tempdir.path()))
        );
        assert_eq!(
            *env.get("RUBYLIB").unwrap(),
            path_to_string(
                join_paths(vec![
                    tempdir.path().join("lib/ruby/site_ruby/9.9.0"),
                    tempdir.path().join(format!(
                        "lib/ruby/site_ruby/9.9.0/{}",
                        runtime.platform_directory(&tool)
                    )),
                    tempdir.path().join("lib/ruby/site_ruby"),
                    tempdir.path().join("lib/ruby/vendor_ruby/9.9.0"),
                    tempdir.path().join(format!(
                        "lib/ruby/vendor_ruby/9.9.0/{}",
                        runtime.platform_directory(&tool)
                    )),
                    tempdir.path().join("lib/ruby/vendor_ruby"),
                    tempdir.path().join("lib/ruby/9.9.0"),
                    tempdir.path().join(format!(
                        "lib/ruby/9.9.0/{}",
                        runtime.platform_directory(&tool)
                    )),
                    tempdir.path().join("lib/ruby/"),
                ])
                .unwrap()
            )
        );
        drop(tempdir);
    }
}
