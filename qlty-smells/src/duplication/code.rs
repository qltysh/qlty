use md5::Context;
use qlty_analysis::code::{DigestDef, File};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;

const CONTEXT_LINES: usize = 10;

#[derive(Debug, Serialize)]
pub struct Node {
    pub kind: String,
    pub mass: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    pub children: Vec<Arc<Node>>,

    #[serde(with = "DigestDef")]
    pub structural_hash: md5::Digest,

    #[serde(with = "DigestDef")]
    pub source_hash: md5::Digest,
}

#[derive(Debug)]
pub struct NodeWithFile {
    pub source_file: Arc<File>,
    pub node: Arc<Node>,
}

impl NodeWithFile {
    pub fn snippet(&self) -> String {
        let file_source = self.source_file.contents.as_bytes();
        std::str::from_utf8(&file_source[self.node.start_byte..self.node.end_byte])
            .unwrap_or("")
            .to_string()
    }

    pub fn snippet_with_context(&self) -> String {
        let file_contents_str =
            std::str::from_utf8(self.source_file.contents.as_bytes()).unwrap_or("");
        if file_contents_str.is_empty() {
            return "".to_string();
        }

        let lines: Vec<&str> = file_contents_str.lines().collect();

        let start_line = self.node.start_line;
        let end_line = self.node.end_line;

        let start_context_index = if start_line > CONTEXT_LINES {
            start_line - CONTEXT_LINES
        } else {
            0
        };
        let end_context_index = if end_line + CONTEXT_LINES < lines.len() {
            end_line + CONTEXT_LINES
        } else {
            lines.len() - 1
        };

        lines[start_context_index..=end_context_index].join("\n")
    }
}

impl Node {
    pub fn lines_count(&self) -> usize {
        self.end_line - self.start_line + 1
    }

    pub fn all_structural_subhashes(&self) -> HashSet<md5::Digest> {
        let mut result = HashSet::new();

        for child in &self.children {
            result.insert(child.structural_hash);

            for structural_hash in child.all_structural_subhashes() {
                result.insert(structural_hash);
            }
        }

        result
    }
}

pub fn build_node(
    source_file: &File,
    current: &tree_sitter::Node,
    children: Vec<Arc<Node>>,
) -> Arc<Node> {
    Arc::new(Node {
        kind: String::from(current.kind()),
        mass: 1 + children.iter().map(|x| x.mass).sum::<usize>(),

        start_line: current.start_position().row + 1,
        end_line: current.end_position().row + 1,

        start_byte: current.start_byte(),
        end_byte: current.end_byte(),

        structural_hash: compute_structural_hash(
            current.kind(),
            children.iter().map(|x| x.structural_hash).collect(),
        ),
        source_hash: md5::compute(current.utf8_text(source_file.contents.as_bytes()).unwrap()),
        children,
    })
}

fn compute_structural_hash(node_kind: &str, children: Vec<md5::Digest>) -> md5::Digest {
    let mut digest_context = Context::new();
    digest_context.consume(node_kind);

    for child in children {
        digest_context.consume(child.0);
    }

    digest_context.compute()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn structural_hash() {
        let digest = compute_structural_hash("node_kind", vec![]);
        insta::assert_debug_snapshot!(digest, @"4712bb4037140c10c916b6c9b566fb25");
    }

    #[test]
    fn structural_hash_with_children() {
        let digest = compute_structural_hash("node_kind", vec![]);
        let digest = compute_structural_hash("node_kind", vec![digest]);
        insta::assert_debug_snapshot!(digest, @"81fc77a3a24c51aec303615956fb579f");
    }
}
