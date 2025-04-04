use super::renderer::PluginActivation;
use super::scanner::InstalledPlugin;
use super::sources::{source_specs_from_settings, sources_list_from_settings};
use super::{Renderer, Scanner, Settings, SourceSpec};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone)]
pub struct Initializer {
    settings: Settings,
    source_specs: Vec<SourceSpec>,
    scanner: Scanner,
    pub plugins: Vec<InstalledPlugin>,
}

impl Initializer {
    pub fn new(settings: Settings) -> Result<Self> {
        let source_specs = source_specs_from_settings(&settings)?;
        let sources_list = sources_list_from_settings(&settings, &source_specs)?;

        Ok(Self {
            settings: settings.clone(),
            source_specs: source_specs.clone(),
            scanner: Scanner::new(settings, &source_specs, Box::new(sources_list))?,
            plugins: vec![],
        })
    }

    pub fn prepare(&mut self) -> Result<()> {
        if !self.settings.skip_plugins {
            self.scanner.prepare()?;
        }

        Ok(())
    }

    pub fn compute(&mut self) -> Result<()> {
        if self.settings.skip_plugins {
            return Ok(());
        }

        let spinner_style = ProgressStyle::with_template(" ∟ {prefix:.bold} {msg} {spinner}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        let progress = ProgressBar::new(1).with_style(spinner_style);
        progress.set_prefix("Scanning");
        progress.set_message("0 files");

        self.scanner.scan(&progress)?;
        progress.finish_and_clear();

        self.plugins = self.scanner.plugins.clone();
        Ok(())
    }

    pub fn qlty_toml(&self) -> Result<String> {
        let mut plugin_activations = vec![];

        for installed_plugin in &self.plugins {
            plugin_activations.push(PluginActivation {
                name: installed_plugin.name.clone(),
                drivers: installed_plugin.enabled_drivers.clone(),
                version: Some(installed_plugin.version.clone()),
                package_file: installed_plugin.package_file.clone(),
                package_filters: installed_plugin.package_filters.clone(),
                prefix: installed_plugin.prefix.clone(),
                mode: installed_plugin.mode,
            });
        }

        Renderer::new(&self.source_specs, &plugin_activations).render()
    }

    pub fn write(&self) -> Result<()> {
        self.write_gitignore()?;
        self.write_qlty_toml()?;
        self.copy_configs()
    }

    fn write_gitignore(&self) -> Result<()> {
        let library = self.settings.workspace.library()?;
        std::fs::write(
            library.gitignore_path(),
            include_str!("./templates/gitignore.txt"),
        )?;
        Ok(())
    }

    fn write_qlty_toml(&self) -> Result<()> {
        let library = self.settings.workspace.library()?;
        std::fs::write(library.qlty_config_path(), self.qlty_toml()?)?;
        Ok(())
    }

    fn copy_configs(&self) -> Result<()> {
        let library = self.settings.workspace.library()?;

        for installed_plugin in &self.plugins {
            for config_file in &installed_plugin.config_files {
                let file_name = config_file.path.file_name().unwrap();
                let destination = library.configs_dir().join(file_name);
                config_file.write_to(&destination)?;
            }
        }

        Ok(())
    }
}
