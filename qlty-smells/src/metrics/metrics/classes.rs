use qlty_analysis::code::{matches_count, File, NodeFilter};
use tree_sitter::Node;

pub fn count<'a>(source_file: &'a File, node: &Node<'a>, filter: &NodeFilter) -> usize {
    matches_count(
        source_file.language().class_query(),
        node,
        "definition.class",
        filter,
        source_file,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    mod elixir {
        use super::*;

        #[test]
        fn counts_modules_as_classes() {
            let source_file = File::from_string(
                "elixir",
                "defmodule A do\n def a, do: 1\nend\n\ndefmodule B do\n def b, do: 2\nend\n",
            );
            assert_eq!(
                2,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }
    }
}
