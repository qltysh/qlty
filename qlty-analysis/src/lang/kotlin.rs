use crate::code::node_source;
use crate::code::File;
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
(class_declaration
    (type_identifier) @name) @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
(function_declaration
	(simple_identifier) @name
    (function_value_parameters) @parameters) @definition.function
"#;

const FIELD_QUERY: &str = r#"
[
    (class_parameter
        (simple_identifier) @name) @field
    (class_body
        (property_declaration
            (variable_declaration) @name) @field)
    (class_body
        (property_declaration
            (multi_variable_declaration
            (variable_declaration
                (simple_identifier) @name))) @field)
    (
        (this_expression)
        (navigation_suffix
            (simple_identifier) @name)) @field
]"#;

pub struct Kotlin {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl Kotlin {
    pub const NAME: &'static str = "kotlin";
    pub const THIS: &'static str = "this";
    pub const THIS_EXPRESSION: &'static str = "this_expression";
    pub const IF: &'static str = "if_expression";
    pub const SOURCE_FILE: &'static str = "source_file";
    pub const WHEN: &'static str = "when_expression";
    pub const FOR: &'static str = "for_statement";
    pub const WHILE: &'static str = "while_statement";
    pub const DO_WHILE: &'static str = "do_while_statement";
    pub const JUMP: &'static str = "jump_expression";
    pub const SIMPLE_IDENTIFIER: &'static str = "simple_identifier";
    pub const NAVIGATION_EXPRESSION: &'static str = "navigation_expression";
    pub const CONJUNCTION_EXPRESSION: &'static str = "conjunction_expression";
    pub const DISJUNCTION_EXPRESSION: &'static str = "disjunction_expression";
    pub const CALL_EXPRESSION: &'static str = "call_expression";
    pub const WHEN_ENTRY: &'static str = "when_entry";
    pub const LAMBDA_LITERAL: &'static str = "lambda_literal";
    pub const LINE_COMMENT: &'static str = "line_comment";
    pub const MULTILINE_COMMENT: &'static str = "multiline_comment";
    pub const STRING_LITERAL: &'static str = "string_literal";
    pub const FUNCTION_DECLARATION: &'static str = "function_declaration";
    pub const CATCH_BLOCK: &'static str = "catch_block";
    pub const TRY_EXPRESSION: &'static str = "try_expression";
    pub const PRIMARY_CONSTRUCTOR: &'static str = "primary_constructor";
    pub const SECONDARY_CONSTRUCTOR: &'static str = "secondary_constructor";
    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";
}

impl Default for Kotlin {
    fn default() -> Self {
        let language = tree_sitter_kotlin::language();

        Self {
            class_query: tree_sitter::Query::new(&language, CLASS_QUERY).unwrap(),
            function_declaration_query: tree_sitter::Query::new(
                &language,
                FUNCTION_DECLARATION_QUERY,
            )
            .unwrap(),
            field_query: tree_sitter::Query::new(&language, FIELD_QUERY).unwrap(),
        }
    }
}

impl Language for Kotlin {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn self_keyword(&self) -> Option<&str> {
        Some(Self::THIS)
    }

    fn binary_nodes(&self) -> Vec<&str> {
        vec![Self::CONJUNCTION_EXPRESSION, Self::DISJUNCTION_EXPRESSION]
    }

    fn boolean_operator_nodes(&self) -> Vec<&str> {
        vec![Self::AND, Self::OR]
    }

    fn call_nodes(&self) -> Vec<&str> {
        vec![Self::CALL_EXPRESSION]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec![Self::WHEN_ENTRY]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![Self::LAMBDA_LITERAL]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::LINE_COMMENT, Self::MULTILINE_COMMENT]
    }

    // no ternary operator
    fn conditional_assignment_nodes(&self) -> Vec<&str> {
        vec![]
    }

    // there isn't a else node/clause according to tree-sitter-kotlin
    fn else_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![Self::CATCH_BLOCK]
    }

    fn field_nodes(&self) -> Vec<&str> {
        vec![Self::THIS_EXPRESSION]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::FUNCTION_DECLARATION]
    }

    fn if_nodes(&self) -> Vec<&str> {
        vec![Self::IF]
    }

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::SOURCE_FILE]
    }

    fn jump_nodes(&self) -> Vec<&str> {
        vec![Self::JUMP]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::FOR, Self::WHILE, Self::DO_WHILE]
    }

    fn return_nodes(&self) -> Vec<&str> {
        vec![Self::JUMP]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING_LITERAL]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::WHEN]
    }

    fn ternary_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![Self::TRY_EXPRESSION]
    }

    fn constructor_names(&self) -> Vec<&str> {
        vec![Self::PRIMARY_CONSTRUCTOR, Self::SECONDARY_CONSTRUCTOR]
    }

    fn class_query(&self) -> &tree_sitter::Query {
        &self.class_query
    }

    // is_decorator_function there seem to be some decorator patterns but no nodes in treesitter
    // https://reflectoring.io/kotlin-design-patterns/#:~:text=the%20Printer%20interface.-,Decorator%20Pattern,pattern%20using%20interfaces%20and%20classes.

    fn function_declaration_query(&self) -> &tree_sitter::Query {
        &self.function_declaration_query
    }

    fn field_query(&self) -> &tree_sitter::Query {
        &self.field_query
    }

    fn iterator_method_identifiers(&self) -> Vec<&str> {
        vec![
            "all",
            "any",
            "count",
            "dropLastWhile",
            "dropWhile",
            "filter",
            "filterNot",
            "find",
            "findLast",
            "forEach",
            "map",
            "none",
            "partition",
            "reduce",
            "reduceRight",
            "scan",
        ]
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        let function_node = node.child(0).unwrap();
        let function_kind = function_node.kind();

        match function_kind {
            Self::SIMPLE_IDENTIFIER => (
                Some("".to_string()),
                node_source(&function_node, source_file),
            ),
            Self::NAVIGATION_EXPRESSION => {
                let (receiver, object) = self.field_identifiers(source_file, &function_node);

                (Some(receiver), object)
            }
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        if node.kind() == Self::THIS_EXPRESSION {
            let parent = node.parent().unwrap();

            if parent.kind() == Self::NAVIGATION_EXPRESSION {
                let navigation_suffix = parent.child(parent.child_count() - 1).unwrap();
                (
                    Self::THIS.to_string(),
                    node_source(
                        &navigation_suffix
                            .child(navigation_suffix.child_count() - 1)
                            .unwrap(),
                        source_file,
                    ),
                )
            } else {
                // handle `this` without navigation expression
                ("".to_string(), "".to_string())
            }
        } else {
            // node.child_count() - 1 in case there are any inline comments before the call obj
            // example
            // foo.bar
            //     .baz(1) // this comment is child(1)
            //     .bax(param)
            (
                node_source(&node.child(0).unwrap(), source_file),
                node_source(
                    &node
                        .child(node.child_count() - 1)
                        .expect("No child node found in navigation_suffix")
                        .child(1)
                        .unwrap(),
                    source_file,
                ),
            )
        }
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_kotlin::language()
    }

    fn has_field_names(&self) -> bool {
        true
    }

    fn sanitize_parameter_name(&self, parameter_name: String) -> Option<String> {
        Some(String::from(parameter_name.split(':').next().unwrap()))
    }

    fn function_name_node<'a>(&'a self, node: &'a Node) -> Node<'a> {
        node.child(0).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use tree_sitter::Tree;

    #[test]
    fn mutually_exclusive() {
        let lang = Kotlin::default();
        let mut kinds: Vec<&str> = vec![];

        kinds.extend(lang.if_nodes());
        kinds.extend(lang.else_nodes());
        kinds.extend(lang.conditional_assignment_nodes());
        kinds.extend(lang.switch_nodes());
        kinds.extend(lang.case_nodes());
        kinds.extend(lang.ternary_nodes());
        kinds.extend(lang.loop_nodes());
        kinds.extend(lang.except_nodes());
        kinds.extend(lang.try_expression_nodes());
        kinds.extend(lang.return_nodes());
        kinds.extend(lang.binary_nodes());
        kinds.extend(lang.field_nodes());
        kinds.extend(lang.call_nodes());
        kinds.extend(lang.function_nodes());
        kinds.extend(lang.closure_nodes());
        kinds.extend(lang.comment_nodes());
        kinds.extend(lang.string_nodes());
        kinds.extend(lang.boolean_operator_nodes());

        let unique: HashSet<_> = kinds.iter().cloned().collect();
        assert_eq!(unique.len(), kinds.len());
    }

    #[test]
    fn field_identifier_read() {
        let source_file = File::from_string(Kotlin::NAME, "this.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let field = root_node.named_child(0).unwrap();
        let language = Kotlin::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("this".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_write() {
        let source_file = File::from_string(Kotlin::NAME, "this.foo = 1");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Kotlin::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("this".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_collaborator() {
        let source_file = File::from_string(Kotlin::NAME, "other.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let field = root_node.named_child(0).unwrap();
        let language = Kotlin::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("other".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier() {
        let source_file = File::from_string(Kotlin::NAME, "foo()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Kotlin::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_attribute() {
        let source_file = File::from_string(Kotlin::NAME, "foo.bar()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Kotlin::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".to_string()), "bar".to_string())
        );
    }

    fn call_node(tree: &Tree) -> Node {
        let root_node = tree.root_node();
        root_node.named_child(0).unwrap()
    }
}
