use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::anyhow;
use clap::Args;
use qlty_config::Library;
use qlty_slop_one::{render_summary, render_text, Document, Evaluator, JevProvider, Options};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct SlopOne {
    /// Source files to evaluate
    pub files: Vec<PathBuf>,

    /// Emit one JSON result object
    #[arg(long)]
    pub json: bool,

    /// Use existing cached analyses; make no API calls
    #[arg(long)]
    pub offline: bool,

    /// Include test files and Rust inline test modules (excluded by default)
    #[arg(long)]
    pub include_tests: bool,

    /// Include generated, dependency, documentation, and test files; also include inline tests
    #[arg(long)]
    pub include_excluded: bool,

    /// Directory for cached Jev answers (default: ~/.qlty/cache/slop-one)
    #[arg(long)]
    pub cache_dir: Option<PathBuf>,

    /// Maximum estimated API spend for this invocation in USD
    #[arg(long, default_value_t = 1.0)]
    pub budget: f64,

    /// Jev provider: typesafe (direct) or vercel (AI Gateway)
    #[arg(long, value_enum, default_value_t = Provider::Typesafe)]
    pub provider: Provider,

    /// Show model metadata without evaluating files
    #[arg(long)]
    pub model_info: bool,

    /// Number of positive and negative factors in text output
    #[arg(long, default_value_t = 3)]
    pub top: usize,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum Provider {
    Typesafe,
    Vercel,
}

impl SlopOne {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        if !(1..=15).contains(&self.top) {
            return Err(CommandError::InvalidOptions {
                message: "--top must be between 1 and 15".to_string(),
            });
        }
        if self.files.is_empty() && !self.model_info {
            return Err(CommandError::InvalidOptions {
                message: "provide at least one source file, or --model-info".to_string(),
            });
        }
        let cache_dir = match &self.cache_dir {
            Some(dir) => dir.clone(),
            None => Library::global_cache_root()?.join("slop-one"),
        };
        let evaluator = Evaluator::new(Options {
            cache_dir,
            budget_usd: self.budget,
            provider: match self.provider {
                Provider::Typesafe => JevProvider::TypeSafe,
                Provider::Vercel => JevProvider::Vercel,
            },
            offline: self.offline,
            include_tests: self.include_tests,
            include_excluded: self.include_excluded,
        })?;
        if self.model_info && self.files.is_empty() {
            println!("{}", serde_json::to_string_pretty(&evaluator.model_info())?);
            return CommandSuccess::ok();
        }
        let mut seen = HashSet::new();
        let mut results = vec![];
        for path in &self.files {
            let normalized = std::path::absolute(path)?;
            if !seen.insert(normalized) {
                continue;
            }
            results.push(evaluator.evaluate(path));
        }
        let document = Document::new(evaluator.model_info(), results, evaluator.usage());
        if self.json {
            println!("{}", serde_json::to_string_pretty(&document)?);
        } else {
            let mut sections: Vec<String> = document
                .results
                .iter()
                .map(|result| render_text(result, self.top))
                .collect();
            sections.push(render_summary(&document.summary));
            println!("{}", sections.join("\n\n"));
        }
        if document.errors > 0 {
            return Err(CommandError::Unknown {
                source: anyhow!("{} file(s) could not be evaluated", document.errors),
            });
        }
        Ok(CommandSuccess {
            fail: document.passed == Some(false),
            ..Default::default()
        })
    }
}

