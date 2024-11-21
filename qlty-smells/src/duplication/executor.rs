use super::Node;
use super::NodeVisitor;
use super::NodeWithFile;
use super::Plan;
use qlty_analysis::utils::fs::path_to_string;
use qlty_analysis::{
    code::{File, NodeFilterBuilder, Visitor},
    Report,
};
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use qlty_types::calculate_effort_minutes;
use qlty_types::language_enum_from_name;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

const BASE_EFFORT_MINUTES: u32 = 10;
const EFFORT_MINUTES_PER_VALUE_DELTA: u32 = 5;
const MAX_OTHER_LOCATIONS: usize = 21;

#[derive(Debug)]
pub struct Executor {
    plan: Plan,
    pub nodes_by_hash: HashMap<md5::Digest, Vec<NodeWithFile>>,
}

impl Executor {
    pub fn new(plan: &Plan) -> Self {
        Self {
            plan: plan.clone(),
            nodes_by_hash: HashMap::new(),
        }
    }

    pub fn execute(&mut self) {
        self.index();
        self.prune();
    }

    pub fn index(&mut self) {
        let files_to_results: Vec<_> = self
            .plan
            .source_files
            .clone()
            .into_par_iter()
            .map(|source_file| {
                let tree = source_file.parse();
                let root = tree.root_node();

                let filter_builder = NodeFilterBuilder::for_patterns(
                    source_file.language(),
                    self.plan
                        .get_language(&source_file.language_name)
                        .filters
                        .clone(),
                );

                let mut visitor = NodeVisitor {
                    depth: 0,
                    source_file: &source_file,
                    stack: vec![vec![]],
                    nodes: vec![],
                    filter: &filter_builder.build(&source_file, &tree),
                };

                visitor.process_node(&mut root.walk());

                (source_file.clone(), visitor.nodes)
            })
            .collect();

        for (source_file, nodes) in files_to_results {
            self.index_file(source_file, &nodes);
        }
    }

    pub fn index_file(&mut self, source_file: Arc<File>, nodes: &[Arc<Node>]) {
        let nodes_threshold = self
            .plan
            .get_language(&source_file.language_name)
            .nodes_threshold;

        for node in &mut nodes.iter() {
            if node.mass >= nodes_threshold {
                self.insert(source_file.clone(), node.clone());
            }
        }
    }

    fn insert(&mut self, source_file: Arc<File>, node: Arc<Node>) {
        let node_with_file = NodeWithFile {
            source_file,
            node: node.clone(),
        };

        self.nodes_by_hash
            .entry(node.structural_hash)
            .or_default()
            .push(node_with_file);
    }

    pub fn prune(&mut self) {
        self.prune_singletons();
        self.prune_subtrees();
    }

    fn prune_singletons(&mut self) {
        self.nodes_by_hash.retain(|_, nodes| nodes.len() > 1);
    }

    fn prune_subtrees(&mut self) {
        let mut subtrees_to_prune: HashSet<md5::Digest> = HashSet::new();

        for node_group in self.nodes_by_hash.values() {
            // Optimize by considering structural hashes of only the first occurrence because
            // all occurrences have the same structure and therefore the same subhashes
            let node = &node_group[0];

            // If we've already marked the structure of the current node for pruning,
            // we can save time by not repeatively marking it's sub-structures for pruning
            if !subtrees_to_prune.contains(&node.node.structural_hash) {
                for subtree_hash in node.node.all_structural_subhashes() {
                    subtrees_to_prune.insert(subtree_hash);
                }
            }
        }

        for subtree_hash in subtrees_to_prune {
            self.nodes_by_hash.remove(&subtree_hash);
        }
    }

    pub fn report(&self) -> Report {
        Report {
            issues: self.issues(),
            ..Default::default()
        }
    }

    fn issues(&self) -> Vec<Issue> {
        let mut issues = vec![];

        for (structural_hash, nodes) in self.nodes_by_hash.iter() {
            let first_node = &nodes[0];
            let language_plan = self
                .plan
                .get_language(&first_node.source_file.language_name);

            let identical_lines_threshold = language_plan.identical_lines_threshold;
            let similar_lines_threshold = language_plan.similar_lines_threshold;
            let lines_count = first_node.node.lines_count() as u32;

            let identical = nodes[1..]
                .iter()
                .all(|x| x.node.source_hash == first_node.node.source_hash);

            let lines_threshold = if identical {
                if let Some(identical_lines_threshold) = identical_lines_threshold {
                    identical_lines_threshold
                } else {
                    continue;
                }
            } else if let Some(similar_lines_threshold) = similar_lines_threshold {
                similar_lines_threshold
            } else {
                continue;
            } as u32;

            if lines_count < lines_threshold {
                continue;
            }

            for (i, node) in nodes.iter().enumerate() {
                let other_locations = nodes
                    .par_iter()
                    .take(nodes.len().min(MAX_OTHER_LOCATIONS))
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, node)| Location {
                        path: path_to_string(&node.source_file.path),
                        range: Some(Range {
                            start_line: node.node.start_line as u32,
                            end_line: node.node.end_line as u32,
                            ..Default::default()
                        }),
                    })
                    .collect::<Vec<_>>();

                let rule_key = if identical {
                    "identical-code"
                } else {
                    "similar-code"
                };

                let message = format!(
                    "Found {} lines of {} code in {} locations (mass = {})",
                    node.node.end_line - node.node.start_line + 1,
                    if identical { "identical" } else { "similar" },
                    nodes.len(),
                    first_node.node.mass
                );

                let location = Some(Location {
                    path: path_to_string(&node.source_file.path),
                    range: Some(Range {
                        start_line: node.node.start_line as u32,
                        end_line: node.node.end_line as u32,
                        start_byte: Some(node.node.start_byte as u32),
                        end_byte: Some(node.node.end_byte as u32),
                        ..Default::default()
                    }),
                });

                let value_delta = lines_count - lines_threshold;

                let mut issue = Issue {
                    tool: "qlty".to_string(),
                    driver: "duplication".to_string(),
                    rule_key: rule_key.to_string(),
                    category: Category::Duplication.into(),
                    language: language_enum_from_name(node.source_file.language().name()).into(),
                    location,
                    snippet: node.snippet(),
                    snippet_with_context: node.snippet_with_context(),
                    level: Level::Medium.into(),
                    message: message.to_string(),
                    value: lines_count,
                    value_delta,
                    effort_minutes: calculate_effort_minutes(
                        value_delta,
                        BASE_EFFORT_MINUTES,
                        EFFORT_MINUTES_PER_VALUE_DELTA,
                    ),
                    other_locations,
                    mode: language_plan.issue_mode as i32,
                    ..Default::default()
                };

                issue.set_property_string("node_kind", first_node.node.kind.clone());
                issue.set_property_string("structural_hash", format!("{:x}", structural_hash));
                issue.set_property_number("mass", first_node.node.mass as f64);
                issue.set_property_bool("identical", identical);

                issues.push(issue);
            }
        }

        self.apply_issues_transfomers(issues)
    }

    fn apply_issues_transfomers(&self, issues: Vec<Issue>) -> Vec<Issue> {
        let transformers = &self.plan.transformers;
        issues
            .into_par_iter()
            .filter_map(|issue| {
                let mut transformed_issue = Some(issue);
                for transformer in transformers {
                    if let Some(issue) = transformed_issue {
                        transformed_issue = transformer.transform(issue);
                    } else {
                        return None;
                    }
                }
                transformed_issue
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::duplication::{Planner, Settings};
    use anyhow::Result;
    use std::path::PathBuf;

    const SIMPLE_CODE: &str = "
        if (x) {
            return 'foo';
        } else {
            return 'bar';
        }
    ";

    const DUPLICATE_CODE: &str = "
        if (x) {
            return 'foo';
        } else {
            return 'bar';
        }

        if (y) {
            return 'baz';
        } else {
            return 'bom';
        }
    ";

    #[test]
    fn process_node_basic() -> Result<()> {
        let source_file = Arc::new(File {
            language_name: "javascript".to_string(),
            path: PathBuf::from("file.js"),
            contents: String::from("return 1;"),
            digest: md5::compute(String::from("return 1;")),
        });

        let plan = Planner::new(
            &qlty_config::QltyConfig::default(),
            &Settings::default(),
            vec![source_file.clone()],
        )?
        .compute()?;

        let mut executor = Executor::new(&plan);
        executor.index();

        insta::assert_yaml_snapshot!(indexed_node_types(&executor), @r"
        - - number
        - - program
        - - return_statement
        ");

        Ok(())
    }

    #[test]
    fn process_node_full() -> Result<()> {
        let source_file = Arc::new(File {
            language_name: "javascript".to_string(),
            path: PathBuf::from("file.js"),
            contents: String::from(SIMPLE_CODE),
            digest: md5::compute(SIMPLE_CODE),
        });

        let plan = Planner::new(
            &qlty_config::QltyConfig::default(),
            &Settings::default(),
            vec![source_file.clone()],
        )?
        .compute()?;

        let mut executor = Executor::new(&plan);
        executor.index();

        insta::assert_yaml_snapshot!(indexed_node_types(&executor), @r"
        - - else_clause
        - - identifier
        - - if_statement
        - - parenthesized_expression
        - - program
        - - return_statement
          - return_statement
        - - statement_block
          - statement_block
        - - string
          - string
        - - string_fragment
          - string_fragment
        ");

        Ok(())
    }

    #[test]
    fn prune_singletons() -> Result<()> {
        let source_file = Arc::new(File {
            language_name: "javascript".to_string(),
            path: PathBuf::from("file.js"),
            contents: String::from(SIMPLE_CODE),
            digest: md5::compute(SIMPLE_CODE),
        });

        let plan = Planner::new(
            &qlty_config::QltyConfig::default(),
            &Settings::default(),
            vec![source_file.clone()],
        )?
        .compute()?;

        let mut executor = Executor::new(&plan);
        executor.index();
        executor.prune_singletons();

        insta::assert_yaml_snapshot!(indexed_node_types(&executor), @r"
        - - return_statement
          - return_statement
        - - statement_block
          - statement_block
        - - string
          - string
        - - string_fragment
          - string_fragment
        ");

        Ok(())
    }

    #[test]
    fn prune_subtrees() -> Result<()> {
        let source_file = Arc::new(File {
            language_name: "javascript".to_string(),
            path: PathBuf::from("file.js"),
            contents: String::from(DUPLICATE_CODE),
            digest: md5::compute(DUPLICATE_CODE),
        });

        let plan = Planner::new(
            &qlty_config::QltyConfig::default(),
            &Settings::default(),
            vec![source_file.clone()],
        )?
        .compute()?;

        let mut executor = Executor::new(&plan);
        executor.execute();

        insta::assert_yaml_snapshot!(indexed_node_types(&executor), @r"
        - - if_statement
          - if_statement
        ");

        Ok(())
    }

    fn indexed_node_types(executor: &Executor) -> Vec<Vec<&str>> {
        let mut actual: Vec<Vec<&str>> = executor
            .nodes_by_hash
            .values()
            .map(|nodes| nodes.iter().map(|node| node.node.kind.as_str()).collect())
            .collect();
        actual.sort();
        actual
    }
}
