pub mod boolean_logic;
pub mod file_complexity;
pub mod function_complexity;
pub mod nested_control;
pub mod parameters;
pub mod returns;

use qlty_analysis::{code::File, utils::fs::path_to_string};
use qlty_types::{
    analysis::v1::{Category, Issue, Location},
    language_enum_from_name,
};
use std::sync::Arc;
use tree_sitter::Node;

const TOOL: &str = "qlty";
const DRIVER: &str = "structure";
const CONTEXT_LINES: usize = 10;

pub fn issue_for(source_file: &Arc<File>, node: &Node) -> Issue {
    let path = source_file.path.to_string_lossy();
    let file_contents = source_file.contents.as_bytes();
    let snippet = std::str::from_utf8(&file_contents[node.start_byte()..node.end_byte()])
        .unwrap_or("")
        .to_string();

    Issue {
        snippet,
        snippet_with_context: snippet_with_context(source_file, node, CONTEXT_LINES),
        language: language_enum_from_name(source_file.language().name()).into(),
        tool: TOOL.to_string(),
        driver: DRIVER.to_string(),
        category: Category::Structure.into(),
        location: Some(Location {
            path: path_to_string(path.as_ref()),
            range: Some(node.range().into()),
        }),
        ..Default::default()
    }
}

fn snippet_with_context(source_file: &Arc<File>, node: &Node, context_lines: usize) -> String {
    let file_contents_str = std::str::from_utf8(source_file.contents.as_bytes()).unwrap_or("");
    if file_contents_str.is_empty() {
        return "".to_string();
    }

    let lines: Vec<&str> = file_contents_str.lines().collect();

    let start_line = node.start_position().row;
    let end_line = node.end_position().row;

    let start_context_index = if start_line > context_lines {
        start_line - context_lines
    } else {
        0
    };
    let end_context_index = if end_line + context_lines < lines.len() {
        end_line + context_lines
    } else {
        lines.len() - 1
    };

    lines[start_context_index..=end_context_index].join("\n")
}
