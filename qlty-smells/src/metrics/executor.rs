use std::collections::HashMap;

use super::{metrics, MetricsMode, Plan, Results};
use qlty_analysis::{
    code::{capture_by_name, capture_source, File},
    utils::fs::path_to_string,
};
use qlty_types::{
    analysis::v1::{ComponentType, Stats},
    language_enum_from_name,
};
use rayon::prelude::*;

#[derive(Debug)]
pub struct Executor {
    plan: Plan,
}

impl Executor {
    pub fn new(plan: &Plan) -> Self {
        Self {
            plan: plan.to_owned(),
        }
    }

    pub fn execute(&mut self) -> Results {
        match self.plan.mode {
            MetricsMode::Files => self.execute_files(),
            MetricsMode::Functions => self.execute_functions(),
        }
    }

    fn execute_files(&mut self) -> Results {
        let stats = self
            .plan
            .source_files
            .clone()
            .into_par_iter()
            .map(|source_file| self.file_stats(&source_file))
            .collect::<Vec<_>>();

        Results { stats }
    }

    fn execute_functions(&mut self) -> Results {
        let stats = self
            .plan
            .source_files
            .clone()
            .into_par_iter()
            .map(|source_file| self.function_stats(&source_file))
            .flatten()
            .collect::<Vec<_>>();

        Results { stats }
    }

    fn file_stats(&self, source_file: &File) -> Stats {
        let tree = source_file.parse();
        let test_filter = self.plan.node_filter_for(source_file, &tree);
        let lines = super::Lines::for_node(source_file, &tree.root_node(), &test_filter);
        let name = path_to_string(source_file.path.file_name().unwrap_or_default());

        Stats {
            kind: ComponentType::File.into(),
            name,
            fully_qualified_name: path_to_string(&source_file.path),
            path: path_to_string(&source_file.path),
            language: language_enum_from_name(source_file.language().name()).into(),
            files: Some(1),
            classes: Some(metrics::classes(source_file, &tree.root_node(), &test_filter) as u32),
            functions: Some(
                metrics::functions(source_file, &tree.root_node(), &test_filter) as u32,
            ),
            fields: Some(metrics::fields(source_file, &tree.root_node(), &test_filter) as u32),
            cyclomatic: Some(
                metrics::cyclomatic(source_file, &tree.root_node(), &test_filter) as u32,
            ),
            complexity: Some(
                metrics::complexity(source_file, &tree.root_node(), &test_filter) as u32,
            ),
            lcom4: Some(metrics::lcom4(source_file, &tree.root_node(), &test_filter) as u32),
            lines: Some(lines.total as u32),
            code_lines: Some(lines.code_lines as u32),
            comment_lines: Some(lines.comment_lines as u32),
            blank_lines: Some(lines.blank_lines as u32),
            ..Default::default()
        }
    }

    fn function_stats(&self, source_file: &File) -> Vec<Stats> {
        let tree = source_file.parse();
        let node = tree.root_node();

        let test_filter = self.plan.node_filter_for(source_file, &tree);

        let language = source_file.language();

        let class_query = language.class_query();

        let mut cursor = tree_sitter::QueryCursor::new();
        cursor.set_match_limit(qlty_analysis::code::QUERY_MATCH_LIMIT as u32);

        let all_matches = cursor.matches(class_query, node, source_file.contents.as_bytes());

        let mut class_definitions = HashMap::new();

        for class_match in all_matches {
            let class_capture = capture_by_name(class_query, "definition.class", &class_match);
            let name = capture_source(class_query, "name", &class_match, source_file);
            class_definitions.insert(class_capture.node.range(), name);
        }

        if let Some(implementation_query) = language.implementation_query() {
            let mut cursor = tree_sitter::QueryCursor::new();
            cursor.set_match_limit(qlty_analysis::code::QUERY_MATCH_LIMIT as u32);

            let all_matches =
                cursor.matches(implementation_query, node, source_file.contents.as_bytes());

            for implementation_match in all_matches {
                let implementation_capture = capture_by_name(
                    implementation_query,
                    "reference.implementation",
                    &implementation_match,
                );
                let name = capture_source(
                    implementation_query,
                    "name",
                    &implementation_match,
                    source_file,
                );
                class_definitions.insert(implementation_capture.node.range(), name);
            }
        }

        let function_query = language.function_declaration_query();

        let mut cursor = tree_sitter::QueryCursor::new();
        cursor.set_match_limit(qlty_analysis::code::QUERY_MATCH_LIMIT as u32);

        let all_matches = cursor.matches(function_query, node, source_file.contents.as_bytes());

        let mut stats = vec![];

        for function_match in all_matches {
            let function_capture =
                capture_by_name(function_query, "definition.function", &function_match);
            let node = function_capture.node;

            if test_filter.exclude(&node) {
                continue;
            }

            let name = capture_source(function_query, "name", &function_match, source_file);

            let operator = match language.is_instance_method(source_file, &function_capture.node) {
                true => "#",
                false => "::",
            };

            let mut containers = class_definitions
                .iter()
                .filter(|(range, _)| {
                    range.start_byte <= node.range().start_byte
                        && range.end_byte >= node.range().end_byte
                })
                .collect::<Vec<(&tree_sitter::Range, &String)>>();

            containers.sort_by(|a, b| a.0.start_byte.cmp(&b.0.start_byte).reverse());
            let class_name = containers.first().map(|(_, name)| name);

            let full_name = match class_name {
                Some(name_string) => format!("{}{}{}", name_string, operator, name),
                None => name.clone(),
            };

            let test_filter = self.plan.node_filter_for(source_file, &tree);
            let lines = super::Lines::for_node(source_file, &node, &test_filter);

            stats.push(Stats {
                kind: ComponentType::Function.into(),
                name,
                fully_qualified_name: full_name,
                path: path_to_string(&source_file.path),
                language: language_enum_from_name(source_file.language().name()).into(),
                fields: Some(metrics::fields(source_file, &node, &test_filter) as u32),
                cyclomatic: Some(metrics::cyclomatic(source_file, &node, &test_filter) as u32),
                complexity: Some(metrics::complexity(source_file, &node, &test_filter) as u32),
                lines: Some(lines.total as u32),
                code_lines: Some(lines.code_lines as u32),
                comment_lines: Some(lines.comment_lines as u32),
                blank_lines: Some(lines.blank_lines as u32),
                ..Default::default()
            });
        }

        stats
    }
}
