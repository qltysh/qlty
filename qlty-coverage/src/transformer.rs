use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use qlty_analysis::utils::fs::path_to_string;
use qlty_types::tests::v1::{CoverageMetadata, CoverageSummary, FileCoverage};
use std::{
    env::current_dir,
    fmt::Debug,
    path::{PathBuf, MAIN_SEPARATOR},
};

pub trait Transformer: Debug + Send + Sync + 'static {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage>;
    fn clone_box(&self) -> Box<dyn Transformer>;
}

impl Clone for Box<dyn Transformer> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct ComputeSummary {}

impl ComputeSummary {
    pub fn new() -> Self {
        Self {}
    }
}

impl Transformer for ComputeSummary {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut covered = 0;
        let mut missed = 0;
        let mut omit = 0;

        for hit in &file_coverage.hits {
            match hit {
                -1 => omit += 1,
                0 => missed += 1,
                _ => covered += 1,
            }
        }

        let mut file_coverage = file_coverage;

        file_coverage.summary = Some(CoverageSummary {
            covered,
            missed,
            omit,
            total: covered + missed + omit,
        });

        Some(file_coverage)
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AppendMetadata {
    metadata: CoverageMetadata,
}

impl AppendMetadata {
    pub fn new(metadata: &CoverageMetadata) -> Self {
        Self {
            metadata: metadata.clone(),
        }
    }
}

impl Transformer for AppendMetadata {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut file_coverage = file_coverage;
        file_coverage.build_id = self.metadata.build_id.clone();
        file_coverage.tag = self.metadata.tag.clone();
        file_coverage.branch = self.metadata.branch.clone();
        file_coverage.commit_sha = Some(self.metadata.commit_sha.clone());
        file_coverage.uploaded_at = self.metadata.uploaded_at;

        if self.metadata.pull_request_number != String::default() {
            file_coverage.pull_request_number = Some(self.metadata.pull_request_number.clone());
        }

        Some(file_coverage)
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct IgnorePaths {
    glob_set: GlobSet,
}

impl IgnorePaths {
    pub fn new(paths: &[String]) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();

        for glob in paths {
            builder.add(Glob::new(glob)?);
        }

        Ok(Self {
            glob_set: builder.build()?,
        })
    }
}

impl Transformer for IgnorePaths {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        if self.glob_set.is_match(&file_coverage.path) {
            None
        } else {
            Some(file_coverage)
        }
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AddPrefix {
    prefix: String,
}

impl AddPrefix {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_owned(),
        }
    }
}

impl Transformer for AddPrefix {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut file_coverage = file_coverage;
        file_coverage.path = format!("{}{}", self.prefix, file_coverage.path);
        Some(file_coverage)
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct StripPrefix {
    prefix: String,
}

impl Default for StripPrefix {
    fn default() -> Self {
        Self {
            prefix: format!(
                "{}{}",
                path_to_string(current_dir().unwrap_or_else(|_| PathBuf::from("."))),
                MAIN_SEPARATOR
            ),
        }
    }
}

impl StripPrefix {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }
}

impl Transformer for StripPrefix {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut file_coverage = file_coverage;
        file_coverage.path = file_coverage.path.replacen(&self.prefix, "", 1);
        Some(file_coverage)
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct StripDotSlashPrefix;

impl Transformer for StripDotSlashPrefix {
    fn transform(&self, file_coverage: FileCoverage) -> Option<FileCoverage> {
        let mut file_coverage = file_coverage;
        if file_coverage.path.starts_with("./") {
            file_coverage.path = file_coverage.path[2..].to_string();
        }
        Some(file_coverage)
    }

    fn clone_box(&self) -> Box<dyn Transformer> {
        Box::new(Self)
    }
}
