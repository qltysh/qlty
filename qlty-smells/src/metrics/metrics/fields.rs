use qlty_analysis::code::File;
use qlty_analysis::code::{capture_by_name, capture_source, NodeFilter};
use std::collections::HashSet;
use tree_sitter::Node;

pub const QUERY_MATCH_LIMIT: usize = 1024;

pub fn count<'a>(source_file: &'a File, node: &Node<'a>, filter: &NodeFilter) -> usize {
    let language = source_file.language();
    let query = language.field_query();

    let mut query_cursor = tree_sitter::QueryCursor::new();
    query_cursor.set_match_limit(QUERY_MATCH_LIMIT as u32);

    let all_matches = query_cursor.matches(query, *node, source_file.contents.as_bytes());

    // For Java, we need to count field declarations individually, not deduplicate by name
    // For other languages like Rust, we deduplicate field accesses by name
    let is_java = language.name() == "java";
    let mut fields = HashSet::new();
    let mut field_count = 0;

    for field_match in all_matches {
        let name = capture_source(query, "name", &field_match, source_file);
        let field_capture = capture_by_name(query, "field", &field_match);

        if filter.exclude(&field_capture.node) {
            continue;
        }

        if let Some(parent) = field_capture.node.parent() {
            // In some languages, field nodes appear within call nodes. We don't want to count those.
            if !language.call_nodes().contains(&parent.kind()) {
                if is_java {
                    // For Java field declarations, count each declaration individually
                    field_count += 1;
                } else {
                    // For other languages (field accesses), deduplicate by name
                    fields.insert(name);
                }
            }
        } else if is_java {
            // For Java field declarations, count each declaration individually
            field_count += 1;
        } else {
            // For other languages (field accesses), deduplicate by name
            fields.insert(name);
        }
    }

    if is_java {
        field_count
    } else {
        fields.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod rust {
        use super::*;

        #[test]
        fn struct_declaration() {
            let source_file = File::from_string(
                "rust",
                r#"
                struct Foo {
                    bar: i32,
                    baz: i32
                }

                fn do_something() {
                    let foo = Foo { bar: 42, baz: 0 };
                    "{} {}", foo.bar, foo.baz);
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

        #[test]
        fn read() {
            let source_file = File::from_string(
                "rust",
                r#"
                self.foo;
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn write() {
            let source_file = File::from_string(
                "rust",
                r#"
                self.foo = 1;
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn unique() {
            let source_file = File::from_string(
                "rust",
                r#"
                self.foo = 1;
                self.foo;
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn multiple() {
            let source_file = File::from_string(
                "rust",
                r#"
                self.foo;
                self.bar;
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

        #[test]
        fn ignore_collaborators() {
            let source_file = File::from_string(
                "rust",
                r#"
                other.foo = 1;
                other.bar;
                "#,
            );
            assert_eq!(
                0,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }
    }

    mod java {
        use super::*;

        #[test]
        fn multiple_classes_same_field_names() {
            let source_file = File::from_string(
                "java",
                r#"
                class BooleanLogic {
                    int foo;
                    int bar;
                }

                class BooleanLogic1 {
                    boolean foo;
                    boolean bar;
                    boolean baz;
                    boolean qux;
                }
                "#,
            );
            assert_eq!(
                6,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn single_class() {
            let source_file = File::from_string(
                "java",
                r#"
                class MyClass {
                    private int field1;
                    public String field2;
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

    mod kotlin {
        use super::*;

        #[test]
        fn class_declaration() {
            let source_file = File::from_string(
                "kotlin",
                r#"
                class Shark {
                    var name: String = ""
                    var age: Int = 0
                }

                fun doSomething() {
                    val shark = Shark()
                    shark.name = "Sammy"
                    shark.age = 5
                    println(shark.name)
                    println(shark.age)
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

        #[test]
        fn read() {
            let source_file = File::from_string(
                "kotlin",
                r#"
                this.foo
            "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn write() {
            let source_file = File::from_string(
                "kotlin",
                r#"
                this.foo = 1
            "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn unique() {
            let source_file = File::from_string(
                "kotlin",
                r#"
                this.foo = 1
                this.foo
            "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn multiple() {
            let source_file = File::from_string(
                "kotlin",
                r#"
                this.foo
                this.bar
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

        #[test]
        fn ignore_collaborators() {
            let source_file = File::from_string(
                "kotlin",
                r#"
                other.foo = 1
                other.bar
            "#,
            );
            assert_eq!(
                0,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }
    }
}
