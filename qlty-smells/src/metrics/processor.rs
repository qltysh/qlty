use super::Results;
use anyhow::Result;
use qlty_analysis::utils::fs::path_to_string;
use qlty_analysis::Report;
use qlty_types::analysis::v1::{ComponentType, Stats};
use std::path::PathBuf;

pub struct Processor {
    results: Results,
    stats: Vec<Stats>,
}

impl Processor {
    pub fn new(results: Results) -> Self {
        Self {
            results: results.clone(),
            stats: results.stats.clone(),
        }
    }

    pub fn compute(&mut self) -> Result<Report> {
        self.aggregate();

        Ok(Report {
            stats: self.stats.clone(),
            ..Default::default()
        })
    }

    fn aggregate(&mut self) {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");

        for file_stat in &self.results.stats.clone() {
            let mut directories = vec![];
            let mut directory = PathBuf::from(file_stat.path.clone());

            while let Some(parent) = directory.parent() {
                if parent == current_dir || parent == PathBuf::from("") {
                    break;
                }

                directories.push(parent.to_path_buf());
                directory = parent.to_path_buf();
            }

            for directory in directories {
                let directory_name = path_to_string(directory.file_name().unwrap_or_default());

                let directory_stat = Stats {
                    kind: ComponentType::Directory.into(),
                    name: directory_name,
                    fully_qualified_name: path_to_string(&directory),
                    path: path_to_string(&directory),
                    ..Default::default()
                };

                self.add(directory_stat, file_stat);
            }
        }
    }

    fn add(&mut self, target: Stats, stats: &Stats) {
        let existing = self.stats.iter_mut().find(|s| {
            s.kind == target.kind && s.fully_qualified_name == target.fully_qualified_name
        });

        if let Some(existing) = existing {
            *existing = existing.to_owned() + stats.to_owned();
        } else {
            self.stats.push(target + stats.to_owned());
        }
    }
}
