use anyhow::{anyhow, bail, Context, Result};
use console::style;
use std::fs;
use std::{fmt::Debug, sync::Arc};
use toml_edit::{value, DocumentMut, Item};

use crate::{configure_proxy_options, Workspace};

#[derive(Debug, Default)]
struct RemoteHeadRetriever;

trait HeadRetriever: Debug + Send + Sync {
    fn remote_fetch_heads(&self, source_url: &str) -> Result<Vec<String>>;
}

impl HeadRetriever for RemoteHeadRetriever {
    fn remote_fetch_heads(&self, source_url: &str) -> Result<Vec<String>> {
        let mut names = vec![];
        let mut remote = git2::Remote::create_detached(source_url)?;

        let mut proxy_options = git2::ProxyOptions::new();
        configure_proxy_options!(proxy_options);
        let callbacks = git2::RemoteCallbacks::new();

        remote.connect_auth(git2::Direction::Fetch, Some(callbacks), Some(proxy_options))?;

        for head in remote.list()? {
            names.push(head.name().to_string());
        }

        Ok(names)
    }
}

#[derive(Debug)]
pub struct SourceUpgrade {
    heads_retriever: Arc<dyn HeadRetriever>,
}

impl Default for SourceUpgrade {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceUpgrade {
    pub fn new() -> Self {
        Self {
            heads_retriever: Arc::<RemoteHeadRetriever>::default(),
        }
    }

    pub fn run(&self) -> Result<()> {
        let workspace = Workspace::new()?;
        let contents = fs::read_to_string(workspace.config_path()?)?;
        let output = self.update_config_tag(contents)?;

        if let Some(output) = output {
            fs::write(workspace.config_path()?, output)?;

            println!(
                "{}, successfully upgraded to the latest version of sources",
                style("Congrats!").green().bold(),
            );
        } else {
            println!(
                "{} You're already on the latest version of sources",
                style("Congrats!").green().bold(),
            );
        }

        Ok(())
    }

    fn update_config_tag(&self, contents: String) -> Result<Option<String>> {
        let mut document = contents.parse::<DocumentMut>().expect("Invalid config doc");
        let mut tags_updated = false;

        if let Some(source_array) = &mut document.get("source") {
            let mut source_array = source_array
                .clone()
                .into_array_of_tables()
                .ok()
                .with_context(|| "Source is not an array")?;

            for source in source_array.iter_mut() {
                if let Some(repository) = source.get("repository") {
                    if let Some(tag) = source.get("tag") {
                        let tag = tag.as_str().unwrap();

                        let latest_source_tag = match self.fetch_source_ref(repository) {
                            Ok(latest_tag) => latest_tag,
                            Err(_) => continue,
                        };

                        if tag != latest_source_tag {
                            tags_updated = true;
                            source["tag"] = value(latest_source_tag.to_string());
                        }
                    }
                }
            }

            if tags_updated {
                document["source"] = Item::ArrayOfTables(source_array.clone());
                Ok(Some(document.to_string()))
            } else {
                Ok(None)
            }
        } else if let Some(sources) = document.get("sources") {
            let mut sources_table = sources
                .clone()
                .into_table()
                .ok()
                .with_context(|| "Sources is not a table")?;

            for (_, source) in sources_table.iter_mut() {
                if let Some(repository) = source.get("repository") {
                    if let Some(tag) = source.get("tag") {
                        let tag = tag.as_str().unwrap();

                        let latest_source_tag = match self.fetch_source_ref(repository) {
                            Ok(latest_tag) => latest_tag,
                            Err(_) => continue,
                        };

                        if tag != latest_source_tag {
                            tags_updated = true;
                            source["tag"] = value(latest_source_tag.to_string());
                        }
                    }
                }
            }

            if tags_updated {
                document["sources"] = Item::Table(sources_table);
                Ok(Some(document.to_string()))
            } else {
                Ok(None)
            }
        } else {
            Err(anyhow!("No sources found in config"))
        }
    }

    fn fetch_source_ref(&self, repository: &Item) -> Result<String> {
        let source_url = repository.as_str().unwrap().to_string();
        let mut semvers = vec![];

        for name in self.heads_retriever.remote_fetch_heads(&source_url)? {
            let (version_string, stripped_v_prefix) =
                if let Some(stripped) = name.strip_prefix("refs/tags/v") {
                    (stripped, true)
                } else if let Some(stripped) = name.strip_prefix("refs/tags/") {
                    (stripped, false)
                } else {
                    continue;
                };

            if let Ok(version) = semver::Version::parse(version_string) {
                semvers.push((version, stripped_v_prefix));
            }
        }

        if semvers.is_empty() {
            bail!("No semver tags found in default source");
        }

        semvers.sort_by(|a, b| a.0.cmp(&b.0));
        let (latest_version, had_v_prefix) = semvers.last().unwrap();

        let result = if *had_v_prefix {
            format!("v{}", latest_version)
        } else {
            latest_version.to_string()
        };

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Default)]
    struct MockHeadRetriever {
        head_list: Vec<String>,
    }

    impl HeadRetriever for MockHeadRetriever {
        fn remote_fetch_heads(&self, _source_url: &str) -> Result<Vec<String>> {
            Ok(self.head_list.clone())
        }
    }

    #[test]
    fn test_update_config_tag_with_source() {
        let input = r#"
            [[source]]
            repository = "https://github.com/qltysh/qlty"
            tag = "v1.0.0"
        "#;

        let head = "refs/tags/v1.2.3".to_string();
        let result = SourceUpgrade {
            heads_retriever: Arc::new(MockHeadRetriever {
                head_list: vec![head.clone()],
            }),
        }
        .update_config_tag(input.to_string());

        assert!(result.is_ok());

        let updated_contents = r#"
            [[source]]
            repository = "https://github.com/qltysh/qlty"
            tag = "v1.2.3"
        "#;
        assert_eq!(result.unwrap().unwrap(), updated_contents);
    }

    #[test]
    fn test_update_config_tag_with_sources() {
        let input = r#"
            [sources.default]
            repository = "https://github.com/qltysh/qlty"
            tag = "v0.101.0"
        "#;

        let head = "refs/tags/v1.2.3".to_string();
        let result = SourceUpgrade {
            heads_retriever: Arc::new(MockHeadRetriever {
                head_list: vec![head.clone()],
            }),
        }
        .update_config_tag(input.to_string());

        assert!(result.is_ok());

        let updated_contents = r#"
            [sources.default]
            repository = "https://github.com/qltysh/qlty"
            tag = "v1.2.3"
        "#;

        assert_eq!(result.unwrap().unwrap(), updated_contents);
    }

    #[test]
    fn test_update_config_tag_with_no_sources() {
        let input = r#"
            [no_sources]
            tag = "v1.0.0"
        "#;

        let head = "refs/tags/v1.2.3".to_string();
        let result = SourceUpgrade {
            heads_retriever: Arc::new(MockHeadRetriever {
                head_list: vec![head.clone()],
            }),
        }
        .update_config_tag(input.to_string());

        assert!(result.is_err());
    }

    #[test]
    fn test_update_config_tag_with_no_repo_source() {
        let input = r#"
            [[source]]
            tag = "v1.0.0"
        "#;

        let head = "refs/tags/v1.2.3".to_string();
        let result = SourceUpgrade {
            heads_retriever: Arc::new(MockHeadRetriever {
                head_list: vec![head.clone()],
            }),
        }
        .update_config_tag(input.to_string());

        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_update_config_tag_nothing_to_update() {
        let input = r#"
            [[source]]
            repository = "default"
            tag = "v1.0.0"
        "#;

        let head = "refs/tags/v1.0.0".to_string();
        let result = SourceUpgrade {
            heads_retriever: Arc::new(MockHeadRetriever {
                head_list: vec![head.clone()],
            }),
        }
        .update_config_tag(input.to_string());

        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_update_config_tag_multiple_source() {
        let input = r#"
            [[source]]
            repository = "default"
            tag = "v1.0.0"

            [[source]]
            directory = "asda"
            tag = "v1.0.1"
        "#;

        let head = "refs/tags/v1.0.1".to_string();
        let result = SourceUpgrade {
            heads_retriever: Arc::new(MockHeadRetriever {
                head_list: vec![head.clone()],
            }),
        }
        .update_config_tag(input.to_string());

        let output = r#"
            [[source]]
            repository = "default"
            tag = "v1.0.1"

            [[source]]
            directory = "asda"
            tag = "v1.0.1"
        "#;

        assert_eq!(result.unwrap().unwrap(), output);
    }

    #[test]
    fn test_update_config_tag_multiple_sources() {
        let input = r#"
            [sources.default]
            repository = "default"
            tag = "v1.0.0"

            [sources.local]
            directory = "asda"
            tag = "v1.0.1"
        "#;

        let head = "refs/tags/v1.0.1".to_string();
        let result = SourceUpgrade {
            heads_retriever: Arc::new(MockHeadRetriever {
                head_list: vec![head.clone()],
            }),
        }
        .update_config_tag(input.to_string());

        let output = r#"
            [sources.default]
            repository = "default"
            tag = "v1.0.1"

            [sources.local]
            directory = "asda"
            tag = "v1.0.1"
        "#;

        assert_eq!(result.unwrap().unwrap(), output);
    }

    #[test]
    fn test_update_config_tag_with_source_without_v() {
        let input = r#"
            [[source]]
            repository = "https://github.com/qltysh/qlty"
            tag = "v1.0.0"
        "#;

        let head = "refs/tags/1.2.3".to_string();
        let result = SourceUpgrade {
            heads_retriever: Arc::new(MockHeadRetriever {
                head_list: vec![head.clone()],
            }),
        }
        .update_config_tag(input.to_string());

        assert!(result.is_ok());

        let output = r#"
            [[source]]
            repository = "https://github.com/qltysh/qlty"
            tag = "1.2.3"
        "#;

        assert_eq!(result.unwrap().unwrap(), output);
    }

    #[test]
    fn test_update_config_tag_with_source_without_semver() {
        let input = r#"
            [[source]]
            repository = "https://github.com/qltysh/qlty"
            tag = "v1.0.0"
        "#;

        let head = "refs/tags/foo".to_string();
        let result = SourceUpgrade {
            heads_retriever: Arc::new(MockHeadRetriever {
                head_list: vec![head.clone()],
            }),
        }
        .update_config_tag(input.to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_update_config_tag_without_ref_tag() {
        let input = r#"
            [[source]]
            repository = "https://github.com/qltysh/qlty"
            tag = "v1.0.0"
        "#;

        let head = "refs/pull/v1.2.3".to_string();
        let result = SourceUpgrade {
            heads_retriever: Arc::new(MockHeadRetriever {
                head_list: vec![head.clone()],
            }),
        }
        .update_config_tag(input.to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_update_config_tag_with_ref_tag_non_semver() {
        let input = r#"
            [[source]]
            repository = "https://github.com/qltysh/qlty"
            tag = "v1.0.0"
        "#;

        let head = "refs/tags/vfoo".to_string();
        let result = SourceUpgrade {
            heads_retriever: Arc::new(MockHeadRetriever {
                head_list: vec![head.clone()],
            }),
        }
        .update_config_tag(input.to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}
