use super::InvocationPlan;
use crate::cache::IssueCache;
use crate::cache::IssuesCacheHit;
use crate::executor::staging_area::StagingArea;
use crate::tool::Tool;
use crate::Settings;
use qlty_analysis::workspace_entries::TargetMode;
use qlty_config::config::issue_transformer::IssueTransformer;
use qlty_config::{QltyConfig, Workspace};
use qlty_types::analysis::v1::ExecutionVerb;
use qlty_types::analysis::v1::Level;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Plan {
    pub verb: ExecutionVerb,
    pub target_mode: TargetMode,
    pub settings: Settings,
    pub config: QltyConfig,
    pub workspace: Workspace,
    pub jobs: usize,
    pub staging_area: StagingArea,
    pub issue_cache: IssueCache,
    pub hits: Vec<IssuesCacheHit>,
    pub transformers: Vec<Box<dyn IssueTransformer>>,
    pub invocations: Vec<InvocationPlan>,
    pub fail_level: Option<Level>,
}

impl Plan {
    pub fn workspace_entry_paths(&self) -> Vec<PathBuf> {
        self.invocations
            .iter()
            .flat_map(|invocation_plan| invocation_plan.workspace_entry_paths())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    }

    pub fn is_empty(&self) -> bool {
        self.invocations.is_empty() && self.hits.is_empty()
    }

    pub fn description(&self) -> String {
        match self.target_mode {
            TargetMode::All => " all targets".to_string(),
            TargetMode::Sample(ref samples) => format!(" {} samples", samples),
            TargetMode::Paths(ref paths) => format!(" {} paths", paths),
            TargetMode::UpstreamDiff(ref upstream) => format!(" vs. {}", upstream),
            TargetMode::HeadDiff => " vs. HEAD".to_string(),
            _ => "".to_string(),
        }
    }

    pub fn progress_increments(&self) -> u64 {
        self.invocations
            .iter()
            .map(|i| i.workspace_entries.len() as u64)
            .sum::<u64>()
    }

    pub fn tools(&self) -> Vec<(String, Box<dyn Tool>)> {
        // Start with the tools for every planned invocation
        let tools = self
            .invocations
            .iter()
            .map(|i| i.tool.clone())
            .collect::<Vec<_>>();

        // Collect the runtimes from the tools which use them
        let tool_runtimes = tools
            .iter()
            .filter_map(|t| t.runtime().to_owned())
            .collect::<Vec<_>>();

        // Add the runtimes to the list of tools
        let all_tools = tools
            .iter()
            .chain(tool_runtimes.iter())
            .cloned()
            .collect::<Vec<_>>();

        // Deduplicate tools by name
        let unique_tools: HashMap<String, Box<dyn Tool>> = all_tools
            .into_iter()
            .map(|t| (t.name().to_string(), t))
            .collect();

        // Return the tools as a vector of tuples so we can sort them
        let mut tools_tuples = unique_tools.into_iter().collect::<Vec<_>>();

        // Sort the tools, putting runtimes and downloads before packages
        tools_tuples.sort_by(|a, b| {
            let a_type = a.1.tool_type();
            let b_type = b.1.tool_type();

            if a_type == b_type {
                a.0.cmp(&b.0)
            } else {
                a_type.cmp(&b_type)
            }
        });

        tools_tuples
    }
}
