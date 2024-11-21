use super::download::{system_arch, system_os, Download};
use crate::Tool;
use qlty_config::config::{DownloadDef, System};

pub trait RunnableArchive: Tool {
    fn download(&self) -> Download {
        let plugin = self.plugin().unwrap();

        Download::new(
            &DownloadDef {
                strip_components: plugin.strip_components.unwrap_or(0),
                systems: vec![System {
                    url: plugin.runnable_archive_url.clone().unwrap(),
                    cpu: system_arch(),
                    os: system_os(),
                }],
                ..Default::default()
            },
            &plugin.runnable_archive_url.clone().unwrap(),
            &plugin.version.clone().unwrap(),
        )
    }
}
