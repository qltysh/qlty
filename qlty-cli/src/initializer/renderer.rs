use crate::initializer::SourceRefSpec;

use super::SourceSpec;
use anyhow::{bail, Result};
use itertools::Itertools;

#[derive(Debug, Clone, Default)]
pub struct PluginActivation {
    pub name: String,
    pub version: Option<String>,
    pub drivers: Vec<String>,
    pub package_file: Option<String>,
    pub package_filters: Vec<String>,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Renderer {
    sources: Vec<SourceSpec>,
    plugins: Vec<PluginActivation>,
}

impl Renderer {
    pub fn new(sources: &[SourceSpec], plugins: &[PluginActivation]) -> Self {
        Self {
            sources: sources.to_vec(),
            plugins: plugins.to_vec(),
        }
    }

    pub fn render(&self) -> Result<String> {
        let parts = vec![
            Some(include_str!("./templates/qlty.toml").to_owned()),
            self.sources()?,
            self.plugins()?,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        Ok(parts.iter().join("\n"))
    }

    fn sources(&self) -> Result<Option<String>> {
        if self.sources.is_empty() {
            return Ok(None);
        }

        let rendered_sources = self
            .sources
            .iter()
            .map(|source| self.render_source(source))
            .collect::<Vec<_>>();

        Ok(Some(self.render_stanzas(rendered_sources)?))
    }

    fn render_stanzas(&self, groups: Vec<Result<String>>) -> Result<String> {
        let groups = groups.into_iter().collect::<Result<Vec<_>>>()?;
        Ok(groups.join(""))
    }

    fn render_source(&self, source: &SourceSpec) -> Result<String> {
        if source.is_default() {
            self.render_default_source()
        } else if source.is_repository() {
            self.render_repository_source(source)
        } else {
            self.render_directory_source(source)
        }
    }

    fn render_default_source(&self) -> Result<String> {
        Ok(include_str!("./templates/source_default.toml").to_owned())
    }

    fn render_repository_source(&self, source: &SourceSpec) -> Result<String> {
        let mut template = include_str!("./templates/source_git.toml").to_owned();
        template = template.replace("{name}", &source.name);
        template = template.replace("{repository}", source.target.as_ref().unwrap());

        Ok(match &source.reference {
            Some(SourceRefSpec::Branch(branch)) => template
                .replace("{reference_type}", "branch")
                .replace("{reference}", branch),
            Some(SourceRefSpec::Tag(tag)) => template
                .replace("{reference_type}", "tag")
                .replace("{reference}", tag),
            None => bail!("Invalid source reference"),
        })
    }

    fn render_directory_source(&self, source: &SourceSpec) -> Result<String> {
        let mut template = include_str!("./templates/source_directory.toml").to_owned();
        template = template.replace("{name}", &source.name);
        template = template.replace("{directory}", source.target.as_ref().unwrap());
        Ok(template)
    }

    fn plugins(&self) -> Result<Option<String>> {
        if self.plugins.is_empty() {
            return Ok(None);
        }

        let rendered_plugins = self
            .plugins
            .iter()
            .sorted_by_key(|plugin| (&plugin.name, &plugin.version)) // Sort by name, then version
            .map(|plugin| self.render_plugin(plugin))
            .collect::<Vec<_>>();

        Ok(Some(self.render_stanzas(rendered_plugins)?))
    }

    fn render_plugin(&self, plugin: &PluginActivation) -> Result<String> {
        let mut toml = "".to_string();
        toml.push_str("\n[[plugin]]\n");
        toml.push_str(&format!("name = \"{}\"\n", plugin.name));

        if !plugin.drivers.is_empty() {
            toml.push_str("drivers = [\n");
            for driver in plugin.drivers.iter().sorted() {
                toml.push_str(&format!("  \"{}\",\n", driver));
            }
            toml.push_str("]\n");
        }

        if let Some(version) = &plugin.version {
            if version != "latest" {
                toml.push_str(&format!("version = \"{}\"\n", version));
            }
        }

        if let Some(package_file) = &plugin.package_file {
            toml.push_str(&format!("package_file = \"{}\"\n", package_file));
            toml.push_str(&format!(
                "package_filters = [\"{}\"]\n",
                plugin.package_filters.join("\", \"")
            ));
        }

        if let Some(prefix) = &plugin.prefix {
            toml.push_str(&format!("prefix = \"{}\"\n", prefix));
        }

        Ok(toml)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        let renderer = Renderer::default();
        let toml_string = renderer.render().unwrap();
        let first_line = toml_string.lines().next().unwrap();
        assert_eq!(
            first_line,
            "# This file was automatically generated by `qlty init`."
        );
    }

    #[test]
    fn test_git_source() {
        let renderer = Renderer::new(
            &vec![SourceSpec {
                name: "example".to_string(),
                target: Some("https://github.com/example/example".to_string()),
                reference: Some(SourceRefSpec::Branch("main".to_string())),
                default: false,
            }],
            &vec![],
        );
        assert_eq!(
            strip_default_toml(renderer.render().unwrap()),
            r#"
[[source]]
name = "example"
repository = "https://github.com/example/example"
branch = "main"
"#
            .trim()
        );
    }

    #[test]
    fn test_sources_multiple() {
        let renderer = Renderer::new(
            &vec![
                SourceSpec {
                    name: "source1".to_string(),
                    target: Some("./source1".to_string()),
                    reference: None,
                    default: false,
                },
                SourceSpec {
                    name: "source2".to_string(),
                    target: Some("./source2".to_string()),
                    reference: None,
                    default: false,
                },
            ],
            &vec![],
        );
        assert_eq!(
            strip_default_toml(renderer.render().unwrap()),
            r#"
[[source]]
name = "source1"
directory = "./source1"
[[source]]
name = "source2"
directory = "./source2"
"#
            .trim()
        );
    }

    #[test]
    fn test_invalid_git_source() {
        let renderer = Renderer::new(
            &vec![SourceSpec {
                name: "source".to_string(),
                target: Some("https://github.com/example/example".to_string()),
                reference: None,
                default: false,
            }],
            &vec![],
        );
        assert!(renderer.render().is_err());
    }

    #[test]
    fn test_local_source() {
        let renderer = Renderer::new(
            &vec![SourceSpec {
                name: "local".to_string(),
                target: Some("./dir".to_string()),
                ..Default::default()
            }],
            &vec![],
        );
        assert_eq!(
            strip_default_toml(renderer.render().unwrap()),
            r#"
[[source]]
name = "local"
directory = "./dir"
"#
            .trim()
        );
    }

    #[test]
    fn test_plugin_basic() {
        let renderer = Renderer::new(
            &vec![],
            &vec![PluginActivation {
                name: "foo".to_string(),
                ..Default::default()
            }],
        );

        assert_eq!(
            strip_default_toml(renderer.render().unwrap()),
            r#"
[[plugin]]
name = "foo"
"#
            .trim()
        );
    }

    #[test]
    fn test_plugin_multiple() {
        let renderer = Renderer::new(
            &vec![],
            &vec![
                PluginActivation {
                    name: "foo".to_string(),
                    ..Default::default()
                },
                PluginActivation {
                    name: "bar".to_string(),
                    ..Default::default()
                },
            ],
        );

        assert_eq!(
            strip_default_toml(renderer.render().unwrap()),
            r#"
[[plugin]]
name = "bar"

[[plugin]]
name = "foo"
"#
            .trim()
        );
    }

    #[test]
    fn test_plugin_latest() {
        let renderer = Renderer::new(
            &vec![],
            &vec![PluginActivation {
                name: "foo".to_string(),
                version: Some("latest".to_string()),
                ..Default::default()
            }],
        );

        assert_eq!(
            strip_default_toml(renderer.render().unwrap()),
            r#"
[[plugin]]
name = "foo"
"#
            .trim()
        );
    }

    #[test]
    fn test_plugin_version() {
        let renderer = Renderer::new(
            &vec![],
            &vec![PluginActivation {
                name: "foo".to_string(),
                version: Some("1.0.0".to_string()),
                ..Default::default()
            }],
        );

        assert_eq!(
            strip_default_toml(renderer.render().unwrap()),
            r#"
[[plugin]]
name = "foo"
version = "1.0.0"
"#
            .trim()
        );
    }

    #[test]
    fn test_plugin_drivers() {
        let renderer = Renderer::new(
            &vec![],
            &vec![PluginActivation {
                name: "foo".to_string(),
                drivers: vec!["driver1".to_string()],
                ..Default::default()
            }],
        );

        assert_eq!(
            strip_default_toml(renderer.render().unwrap()),
            r#"
[[plugin]]
name = "foo"
drivers = [
  "driver1",
]
"#
            .trim()
        );
    }

    #[test]
    fn test_plugin_two_drivers() {
        let renderer = Renderer::new(
            &vec![],
            &vec![PluginActivation {
                name: "foo".to_string(),
                drivers: vec!["driver1".to_string(), "driver2".to_string()],
                ..Default::default()
            }],
        );

        assert_eq!(
            strip_default_toml(renderer.render().unwrap()),
            r#"
[[plugin]]
name = "foo"
drivers = [
  "driver1",
  "driver2",
]
"#
            .trim()
        );
    }

    #[test]
    fn test_plugin_package_filters() {
        let renderer = Renderer::new(
            &vec![],
            &vec![PluginActivation {
                name: "foo".to_string(),
                package_file: Some("foo".to_string()),
                package_filters: vec!["filter1".to_string(), "filter2".to_string()],
                ..Default::default()
            }],
        );

        assert_eq!(
            strip_default_toml(renderer.render().unwrap()),
            r#"
[[plugin]]
name = "foo"
package_file = "foo"
package_filters = ["filter1", "filter2"]
"#
            .trim()
        );
    }

    fn strip_default_toml(toml_string: String) -> String {
        let default_toml_string = Renderer::default().render().unwrap();
        toml_string
            .replace(&default_toml_string, "")
            .trim()
            .to_string()
    }
}
