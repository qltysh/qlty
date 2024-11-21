use super::{Renderer, Settings};
use anyhow::{bail, Result};
use qlty_config::{config::Builder, sources::SourcesList};

const DEFAULT_SOURCE_REPOSITORY: &str = "https://github.com/qltysh/qlty.git";

#[derive(Debug, Clone, Default)]
pub struct SourceSpec {
    pub name: String,
    pub target: String,
    pub reference: Option<SourceRefSpec>,
}

#[derive(Debug, Clone)]
pub enum SourceRefSpec {
    Branch(String),
    Tag(String),
}

impl SourceSpec {
    pub fn new(source_str: &str) -> Result<Self> {
        let parts: Vec<&str> = source_str.splitn(2, '=').collect();

        if parts.len() != 2 || parts[1].is_empty() || parts[0].is_empty() {
            bail!("Invalid source format. Use name=url or name=directory");
        }

        Ok(Self {
            name: parts[0].to_string(),
            target: parts[1].to_string(),
            reference: None,
        })
    }

    pub fn is_repository(&self) -> bool {
        self.target.starts_with("https://") || self.target.starts_with("git@")
    }
}

pub fn source_specs_from_settings(settings: &Settings) -> Result<Vec<SourceSpec>> {
    let mut sources = vec![];

    if !settings.skip_default_source {
        sources.push(SourceSpec {
            name: "default".to_string(),
            target: DEFAULT_SOURCE_REPOSITORY.to_string(),
            reference: Some(SourceRefSpec::Tag(fetch_source_ref(
                DEFAULT_SOURCE_REPOSITORY.to_string(),
            )?)),
        });
    };

    if let Some(source) = settings.source.clone() {
        if source.is_repository() {
            sources.push(SourceSpec {
                name: source.name,
                target: source.target.clone(),
                reference: Some(SourceRefSpec::Tag(fetch_source_ref(
                    source.target.to_string(),
                )?)),
            })
        } else {
            sources.push(SourceSpec {
                name: source.name,
                target: source.target,
                reference: None,
            });
        }
    }

    Ok(sources)
}

pub fn sources_list_from_settings(
    settings: &Settings,
    specs: &[SourceSpec],
) -> Result<SourcesList> {
    Builder::sources_list_from_qlty_toml(
        &Renderer::new(specs, &[]).render()?,
        &settings.workspace.library()?,
    )
}

pub fn fetch_source_ref(repository: String) -> Result<String> {
    let mut remote = git2::Remote::create_detached(repository)?;
    remote.connect(git2::Direction::Fetch)?;

    let mut semvers = vec![];

    for head in remote.list()? {
        let name = head.name();

        if name.starts_with("refs/tags/") {
            let tag_name = name.trim_start_matches("refs/tags/");

            if tag_name.starts_with('v') {
                let version_string = tag_name.trim_start_matches('v');
                if let Ok(version) = semver::Version::parse(version_string) {
                    semvers.push(version);
                }
            }
        }
    }

    if semvers.is_empty() {
        bail!("No semver tags found in default source");
    }

    semvers.sort();
    let latest = semvers.last().unwrap();
    let latest_version = format!("v{}", latest);
    Ok(latest_version)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_source_spec() {
        let source = SourceSpec::new("name=./directory").unwrap();
        assert_eq!(source.name, "name");
        assert_eq!(source.target, "./directory");

        let source = SourceSpec::new("name=https://github.com/foo/bar").unwrap();
        assert_eq!(source.name, "name");
        assert_eq!(source.target, "https://github.com/foo/bar");
    }

    #[test]
    fn test_source_spec_invalid() {
        let source = SourceSpec::new("name=").unwrap_err();
        assert_eq!(
            source.to_string(),
            "Invalid source format. Use name=url or name=directory"
        );

        let source = SourceSpec::new("name").unwrap_err();
        assert_eq!(
            source.to_string(),
            "Invalid source format. Use name=url or name=directory"
        );
    }
}
