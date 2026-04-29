use qlty_analysis::code::{matches_count, File, NodeFilter};
use tree_sitter::Node;

pub fn count<'a>(source_file: &'a File, node: &Node<'a>, filter: &NodeFilter) -> usize {
    matches_count(
        source_file.language().class_query(),
        node,
        "definition.class",
        filter,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    mod scala {
        use super::*;

        #[test]
        fn class_object_trait_and_case_class_each_count() {
            let source_file = File::from_string(
                "scala",
                r#"
class Foo
object Bar
trait Baz
case class Qux(x: Int)
"#,
            );
            assert_eq!(
                4,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }
    }
}
