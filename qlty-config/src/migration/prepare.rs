use crate::migration::classic::{ClassicConfig, FetchItem};
use crate::QltyConfig;
use anyhow::Result;
use std::collections::HashMap;
use url::Url;

pub fn get_plugins_fetch_items(
    classic_config: &ClassicConfig,
    qlty_config: &QltyConfig,
) -> Result<HashMap<String, Vec<FetchItem>>> {
    let mut fetch_items = HashMap::new();

    for fetch_item in classic_config.fetch_items() {
        if find_plugin_for_fetch_item(qlty_config, &fetch_item, &mut fetch_items)? {
            continue;
        }

        return Err(anyhow::anyhow!(format!(
            "Could not find a plugin for fetch item: {:?}",
            fetch_item
        )));
    }

    Ok(fetch_items)
}

fn find_plugin_for_fetch_item(
    qlty_config: &QltyConfig,
    fetch_item: &FetchItem,
    fetch_items: &mut HashMap<String, Vec<FetchItem>>,
) -> Result<bool> {
    if check_plugins_by_config_files(qlty_config, fetch_item, fetch_items)? {
        return Ok(true);
    }
    if check_plugins_by_url_file_name(qlty_config, fetch_item, fetch_items)? {
        return Ok(true);
    }
    if check_plugins_by_path(qlty_config, fetch_item, fetch_items)? {
        return Ok(true);
    }
    if check_plugins_by_url(qlty_config, fetch_item, fetch_items)? {
        return Ok(true);
    }
    Ok(false)
}

fn check_plugins_by_config_files(
    qlty_config: &QltyConfig,
    fetch_item: &FetchItem,
    fetch_items: &mut HashMap<String, Vec<FetchItem>>,
) -> Result<bool> {
    let mut found = false;
    for (plugin_name, plugin) in qlty_config.plugins.definitions.clone() {
        if plugin.config_files.contains(&fetch_item.path) {
            fetch_items
                .entry(plugin_name.clone())
                .or_default()
                .push(fetch_item.clone());
            found = true;
        }
    }
    Ok(found)
}

fn check_plugins_by_url_file_name(
    qlty_config: &QltyConfig,
    fetch_item: &FetchItem,
    fetch_items: &mut HashMap<String, Vec<FetchItem>>,
) -> Result<bool> {
    let mut found = false;
    for (plugin_name, plugin) in qlty_config.plugins.definitions.clone() {
        if let Some(url_file_name) = extract_file_name_from_url(&fetch_item.url) {
            if plugin.config_files.contains(&url_file_name) {
                fetch_items
                    .entry(plugin_name.clone())
                    .or_default()
                    .push(fetch_item.clone());
                found = true;
            }
        }
    }
    Ok(found)
}

fn check_plugins_by_path(
    qlty_config: &QltyConfig,
    fetch_item: &FetchItem,
    fetch_items: &mut HashMap<String, Vec<FetchItem>>,
) -> Result<bool> {
    let mut found = false;
    for plugin_name in qlty_config.plugins.definitions.keys() {
        if fetch_item.path.contains(plugin_name) {
            fetch_items
                .entry(plugin_name.clone())
                .or_default()
                .push(fetch_item.clone());
            found = true;
        }
    }
    Ok(found)
}

fn check_plugins_by_url(
    qlty_config: &QltyConfig,
    fetch_item: &FetchItem,
    fetch_items: &mut HashMap<String, Vec<FetchItem>>,
) -> Result<bool> {
    let mut found = false;
    for plugin_name in qlty_config.plugins.definitions.keys() {
        if fetch_item.url.contains(plugin_name) {
            fetch_items
                .entry(plugin_name.clone())
                .or_default()
                .push(fetch_item.clone());
            found = true;
        }
    }
    Ok(found)
}

fn extract_file_name_from_url(url: &str) -> Option<String> {
    let parsed_url = Url::parse(url).ok()?;
    let segments = parsed_url.path_segments()?;
    segments.last().map(|s| s.to_string())
}
