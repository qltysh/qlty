use qlty_analysis::code::{matches_count, File, NodeFilter};
use tree_sitter::Node;

pub fn count<'a>(source_file: &'a File, node: &Node<'a>, filter: &NodeFilter) -> usize {
    matches_count(
        source_file.language().function_declaration_query(),
        node,
        "definition.function",
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
        fn counts_every_definition_form() {
            let source_file = File::from_string(
                "elixir",
                "defmodule M do\n def a(x), do: x\n defp b, do: 1\n defmacro c(y), do: y\nend\n",
            );
            assert_eq!(
                3,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }
    }
}
