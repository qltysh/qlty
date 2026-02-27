mod rubocop_enabler;

use super::renderer::PluginActivation;
use crate::initializer::Settings;
use anyhow::Result;
use qlty_config::config::{CustomPluginEnabler, PluginDef};
use rubocop_enabler::RubocopEnabler;

pub trait CustomEnabler {
    fn enable(&self, plugin_activation: &PluginActivation) -> Result<PluginActivation>;
}

pub fn custom_enabler_for(
    custom_enabler: CustomPluginEnabler,
    settings: &Settings,
    plugin_def: &PluginDef,
) -> impl CustomEnabler {
    match custom_enabler {
        CustomPluginEnabler::Rubocop => RubocopEnabler::new(settings.clone(), plugin_def.clone()),
    }
}
