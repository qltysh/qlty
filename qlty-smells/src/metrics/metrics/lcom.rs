use qlty_analysis::code::{capture_by_name, capture_source, File, NodeFilter, Visitor};
use qlty_analysis::Language;
use std::collections::HashSet;
use tree_sitter::{Node, TreeCursor};

pub fn count<'a>(source_file: &'a File, node: &Node<'a>, filter: &NodeFilter) -> usize {
    let language = source_file.language();

    let query = if let Some(implementation_query) = language.implementation_query() {
        implementation_query
    } else {
        language.class_query()
    };

    let capture_name = if language.implementation_query().is_some() {
        "reference.implementation"
    } else {
        "definition.class"
    };

    let mut cursor = tree_sitter::QueryCursor::new();
    cursor.set_match_limit(qlty_analysis::code::QUERY_MATCH_LIMIT as u32);

    let all_matches = cursor.matches(query, *node, source_file.contents.as_bytes());

    let mut results = vec![0];

    for class_match in all_matches {
        let name = capture_source(query, "name", &class_match, source_file);

        let class_capture = capture_by_name(query, capture_name, &class_match);

        if filter.exclude(&class_capture.node) {
            continue;
        }

        let result = count_groups(source_file, &name, &class_capture.node, filter);
        results.push(result);
    }

    *results.iter().max().unwrap_or(&0)
}

fn count_groups<'a>(
    source_file: &'a File,
    class_name: &str,
    node: &Node<'a>,
    filter: &NodeFilter,
) -> usize {
    let language = source_file.language();
    let function_query = language.function_declaration_query();

    let mut query_cursor = tree_sitter::QueryCursor::new();
    query_cursor.set_match_limit(qlty_analysis::code::QUERY_MATCH_LIMIT as u32);

    let all_matches = query_cursor.matches(function_query, *node, source_file.contents.as_bytes());

    let mut groups = vec![];

    for function_match in all_matches {
        let function_capture =
            capture_by_name(function_query, "definition.function", &function_match);

        if filter.exclude(&function_capture.node) {
            continue;
        }

        if !language.is_instance_method(source_file, &function_capture.node) {
            continue;
        }

        let name = capture_source(function_query, "name", &function_match, source_file);

        if is_constructor(language, class_name, &name) {
            continue;
        }

        let mut group = Group::default();
        group.functions.insert(name);
        group.merge(&NamedReferences::process(
            source_file,
            &function_capture.node,
        ));

        if !group.fields.is_empty() || group.functions.len() > 1 {
            merge_groups(&mut groups, group);
        }
    }

    groups.len()
}

fn merge_groups(groups: &mut Vec<Group>, mut group: Group) {
    let mut i = groups.len();

    // Iterate in reverse order so that we can remove elements from the vector
    while i > 0 {
        i -= 1;

        let existing_group: &Group = groups.get(i).unwrap();

        if existing_group.intersects(&group) {
            group.merge(existing_group);
            groups.remove(i);
        }
    }

    groups.push(group);
}

#[allow(clippy::borrowed_box)]
fn is_constructor(
    language: &Box<dyn Language + Sync>,
    class_name: &str,
    function_name: &str,
) -> bool {
    if language.constructor_names().contains(&function_name)
        || language.destructor_names().contains(&function_name)
        || class_name == function_name
    {
        return true;
    }

    language
        .constructor_names()
        .iter()
        .any(|constructor_name| function_name.starts_with(constructor_name))
}

pub struct NamedReferences<'a> {
    group: Group,
    source_file: &'a File,
}

#[derive(Debug, Clone, Default)]
pub struct Group {
    fields: HashSet<String>,
    functions: HashSet<String>,
}

impl Group {
    fn merge(&mut self, other: &Self) {
        for field in other.fields.iter() {
            self.fields.insert(field.to_string());
        }

        for function in other.functions.iter() {
            self.functions.insert(function.to_string());
        }
    }

    fn intersects(&self, other: &Self) -> bool {
        !self.fields.is_disjoint(&other.fields) || !self.functions.is_disjoint(&other.functions)
    }
}

impl<'a> Visitor for NamedReferences<'a> {
    fn language(&self) -> &Box<dyn Language + Sync> {
        self.source_file.language()
    }

    fn visit_call(&mut self, cursor: &mut TreeCursor) {
        let (receiver, name) = self.call_identifiers(&cursor.node());

        if receiver.as_deref() == self.language().self_keyword() {
            self.group.functions.insert(name);
        }

        self.process_children(cursor);
    }

    fn visit_field(&mut self, cursor: &mut TreeCursor) {
        let (receiver, name) = self.field_identifiers(&cursor.node());

        if self.language().self_keyword().is_some()
            && receiver == self.language().self_keyword().unwrap()
        {
            self.group.fields.insert(name);
        }

        self.process_children(cursor);
    }
}

impl<'a> NamedReferences<'a> {
    pub fn process(source_file: &'a File, node: &Node<'a>) -> Group {
        let mut refrences = Self {
            source_file,
            group: Group::default(),
        };
        refrences.process_node(&mut node.walk());
        refrences.group
    }

    fn field_identifiers(&self, node: &Node) -> (String, String) {
        self.language().field_identifiers(self.source_file, node)
    }

    fn call_identifiers(&self, node: &Node) -> (Option<String>, String) {
        self.language().call_identifiers(self.source_file, node)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use qlty_analysis::code::NodeFilterBuilder;

    #[test]
    fn lcom_outside_class() {
        let source_file = File::from_string(
            "python",
            r#"
            def foo(self):
                self.bar()
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

    #[test]
    fn lcom_empty() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def foo():
                    pass
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

    #[test]
    fn lcom_call_collaborator() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def foo():
                    bar.baz()
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

    #[test]
    fn lcom_call_self() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def foo(self):
                    self.baz()
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
    fn lcom_field() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def foo(self):
                    self.baz
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
    fn lcom_filter() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def foo(self):
                    self.baz
        "#,
        );

        let tree = source_file.parse();

        let patterns = vec!["(_)".to_string()];
        let builder = NodeFilterBuilder::for_patterns(source_file.language(), patterns);
        let filter = builder.build(&source_file, &tree);

        assert_eq!(
            0,
            count(&source_file, &source_file.parse().root_node(), &filter)
        );
    }

    #[test]
    fn lcom_field_write() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def foo(self):
                    self.baz = 1
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
    fn lcom_ignore_constructor() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def __init__(self):
                    self.baz()
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

    #[test]
    fn lcom_ignore_constructor_prefix() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def __init__foo(self):
                    self.baz()
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

    #[test]
    fn lcom_ignore_constructor_class_name() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def Klass(self):
                    self.baz()
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

    #[test]
    fn lcom_ignore_class_method() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                @classmethod
                def foo(cls):
                    self.baz()
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

    #[test]
    fn lcom_ignore_non_self_araps() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def foo(banana):
                    self.baz()
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

    #[test]
    fn lcom_multiple() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def foo(self):
                    self.aaa()

                def bar(self):
                    self.bbb()
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
    fn lcom_multiple_with_field() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def foo(self):
                    self.aaa()
                    self.field

                def bar(self):
                    self.bbb()
                    self.field
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
    fn lcom_max_classes() {
        let source_file = File::from_string(
            "python",
            r#"
            class KlassA:
                def foo(self):
                    self.aaa()

                def bar(self):
                    self.bbb()

            class KlassB:
                def foo(self):
                    self.baz()

                def bar(self):
                    self.baz()
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
    fn lcom_groups() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def foo(self):
                    self.baz()

                def bar(self):
                    self.baz()
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
    fn lcom_transitive_groups() {
        let source_file = File::from_string(
            "python",
            r#"
            class Klass:
                def foo(self):
                    self.aaa()

                def bar(self):
                    self.aaa()
                    self.bbb()

                def baz(self):
                    self.bbb()
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
    fn lcom_outside_class_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            function foo() {
                this.bar();
            }
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

    #[test]
    fn lcom_empty_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            class Klass {
                foo() {}
            }
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

    #[test]
    fn lcom_call_collaborator_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            class Klass {
                foo() {
                    bar.baz();
                }
            }
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

    #[test]
    fn lcom_call_self_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            class Klass {
                foo() {
                    this.baz();
                }
            }
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
    fn lcom_field_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            class Klass {
                foo() {
                    this.baz;
                }
            }
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
    fn lcom_filter_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            class Klass {
                foo() {
                    this.baz;
                }
            }
            "#,
        );

        let tree = source_file.parse();

        let patterns = vec!["(_)".to_string()];
        let builder = NodeFilterBuilder::for_patterns(source_file.language(), patterns);
        let filter = builder.build(&source_file, &tree);

        assert_eq!(
            0,
            count(&source_file, &source_file.parse().root_node(), &filter)
        );
    }

    #[test]
    fn lcom_field_write_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            class Klass {
                foo() {
                    this.baz = 1;
                }
            }
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
    fn lcom_multiple_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            class Klass {
                foo() {
                    this.aaa();
                }

                bar() {
                    this.bbb();
                }
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
    fn lcom_multiple_with_field_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            class Klass {
                foo() {
                    this.aaa();
                    this.field;
                }

                bar() {
                    this.bbb();
                    this.field;
                }
            }
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
    fn lcom_max_classes_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            class KlassA {
                foo() {
                    this.aaa();
                }

                bar() {
                    this.bbb();
                }
            }

            class KlassB {
                foo() {
                    this.baz();
                }

                bar() {
                    this.baz();
                }
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
    fn lcom_groups_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            class Klass {
                foo() {
                    this.baz();
                }

                bar() {
                    this.baz();
                }
            }
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
    fn lcom_transitive_groups_typescript() {
        let source_file = File::from_string(
            "typescript",
            r#"
            class Klass {
                foo() {
                    this.aaa();
                }

                bar() {
                    this.aaa();
                    this.bbb();
                }

                baz() {
                    this.bbb();
                }
            }
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
}
