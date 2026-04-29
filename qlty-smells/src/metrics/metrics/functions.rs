use qlty_analysis::code::{matches_count, File, NodeFilter};
use tree_sitter::Node;

pub fn count<'a>(source_file: &'a File, node: &Node<'a>, filter: &NodeFilter) -> usize {
    matches_count(
        source_file.language().function_declaration_query(),
        node,
        "definition.function",
        filter,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    mod scala {
        use super::*;

        #[test]
        fn primary_constructor_not_counted_aux_constructor_counted() {
            let source_file = File::from_string(
                "scala",
                r#"
class Foo(val x: Int) {
  def this() = this(0)
  def doStuff(): Int = x + 1
}
"#,
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
