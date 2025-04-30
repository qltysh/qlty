use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use qlty_config::Workspace;
use serde_yaml::{Mapping, Value};

#[derive(Args, Debug, Clone)]
pub struct Show {}

impl Show {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;

        let config = workspace.config()?;

        // Normalize the configuration by deeply sorting maps for deterministic output
        let mut value: Value =
            serde_yaml::to_value(config).map_err(|e| CommandError::from(anyhow::anyhow!(e)))?;
        Self::deep_sort(&mut value);

        let yaml_string =
            serde_yaml::to_string(&value).map_err(|e| CommandError::from(anyhow::anyhow!(e)))?;
        println!("{}", yaml_string);
        CommandSuccess::ok()
    }

    fn deep_sort(value: &mut Value) {
        match value {
            Value::Mapping(map) => {
                let mut sorted_map = Mapping::new();
                let mut entries: Vec<_> = map.iter_mut().collect();
                entries.sort_by(|a, b| a.0.as_str().cmp(&b.0.as_str()));
                for (k, v) in entries {
                    Self::deep_sort(v);
                    sorted_map.insert(k.clone(), v.clone());
                }
                *map = sorted_map;
            }
            Value::Sequence(seq) => {
                for item in seq {
                    Self::deep_sort(item);
                }
            }
            _ => {}
        }
    }
}
