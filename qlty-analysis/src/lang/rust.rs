use crate::code::File;
use crate::code::{child_source, node_source};
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
[
    (struct_item
        name: (type_identifier) @name)

    (enum_item
        name: (type_identifier) @name)

    (union_item
        name: (type_identifier) @name)
] @definition.class
"#;

const IMPLEMENTATION_QUERY: &str = r#"
[
    (impl_item
        type: (generic_type
                type: (type_identifier) @name))

    (impl_item
        type: (type_identifier) @name)
] @reference.implementation
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
(function_item
    name: (identifier) @name
    parameters: (_) @parameters) @definition.function
"#;

const FIELD_QUERY: &str = r#"
[
    (field_declaration
        name: (field_identifier) @name) @field
    (field_expression
        value: (self) @receiver
        field: (_) @name) @field
]
"#;

pub struct Rust {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
    pub implementation_query: tree_sitter::Query,
}

impl Rust {
    pub const SELF: &'static str = "self";

    pub const IDENTIFIER: &'static str = "identifier";
    pub const BINARY: &'static str = "binary_expression";
    pub const BREAK: &'static str = "break_expression";
    pub const CALL: &'static str = "call_expression";
    pub const CLOSURE: &'static str = "closure_expression";
    pub const CONTINUE: &'static str = "continue_expression";
    pub const ELSE: &'static str = "else_clause";
    pub const FOR: &'static str = "for_expression";
    pub const FUNCTION: &'static str = "function_item";
    pub const IF: &'static str = "if_expression";
    pub const LOOP: &'static str = "loop_expression";
    pub const MATCH: &'static str = "match_expression";
    pub const WHILE: &'static str = "while_expression";
    pub const SOURCE_FILE: &'static str = "source_file";
    pub const LINE_COMMENT: &'static str = "line_comment";
    pub const BLOCK_COMMENT: &'static str = "block_comment";
    pub const STRING: &'static str = "string_literal";
    pub const RAW_STRING: &'static str = "raw_string_literal";
    pub const RETURN: &'static str = "return_expression";
    pub const FIELD_EXPRESSION: &'static str = "field_expression";
    pub const SCOPED_EXPRESSION: &'static str = "scoped_identifier";
    pub const SELF_PARAMETER: &'static str = "self_parameter";

    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";
}

impl Default for Rust {
    fn default() -> Self {
        let language = tree_sitter_rust::language();

        Self {
            class_query: tree_sitter::Query::new(&language, CLASS_QUERY).unwrap(),
            function_declaration_query: tree_sitter::Query::new(
                &language,
                FUNCTION_DECLARATION_QUERY,
            )
            .unwrap(),
            field_query: tree_sitter::Query::new(&language, FIELD_QUERY).unwrap(),
            implementation_query: tree_sitter::Query::new(&language, IMPLEMENTATION_QUERY).unwrap(),
        }
    }
}

impl Language for Rust {
    fn name(&self) -> &str {
        "rust"
    }

    fn self_keyword(&self) -> Option<&str> {
        Some(Self::SELF)
    }

    fn class_query(&self) -> &tree_sitter::Query {
        &self.class_query
    }

    fn function_declaration_query(&self) -> &tree_sitter::Query {
        &self.function_declaration_query
    }

    fn field_query(&self) -> &tree_sitter::Query {
        &self.field_query
    }

    fn implementation_query(&self) -> Option<&tree_sitter::Query> {
        Some(&self.implementation_query)
    }

    fn constructor_names(&self) -> Vec<&str> {
        vec!["new", "default"]
    }

    fn destructor_names(&self) -> Vec<&str> {
        vec!["drop"]
    }

    fn is_instance_method(&self, _file: &File, node: &Node) -> bool {
        let parameters = node.child_by_field_name("parameters").unwrap();

        if let Some(first_parameter) = parameters.named_child(0) {
            first_parameter.kind() == Self::SELF_PARAMETER
        } else {
            false
        }
    }

    fn if_nodes(&self) -> Vec<&str> {
        vec![Self::IF]
    }

    fn else_nodes(&self) -> Vec<&str> {
        vec![Self::ELSE]
    }

    fn conditional_assignment_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::SOURCE_FILE]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::MATCH]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec!["match_arm"]
    }

    fn ternary_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::FOR, Self::WHILE, Self::LOOP]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec!["try_expression"]
    }

    fn jump_nodes(&self) -> Vec<&str> {
        vec![Self::BREAK, Self::CONTINUE]
    }

    fn return_nodes(&self) -> Vec<&str> {
        vec![Self::RETURN]
    }

    fn binary_nodes(&self) -> Vec<&str> {
        vec![Self::BINARY]
    }

    fn field_nodes(&self) -> Vec<&str> {
        vec![Self::FIELD_EXPRESSION]
    }

    fn call_nodes(&self) -> Vec<&str> {
        vec![Self::CALL]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::FUNCTION]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![Self::CLOSURE]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::LINE_COMMENT, Self::BLOCK_COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING, Self::RAW_STRING]
    }

    fn boolean_operator_nodes(&self) -> Vec<&str> {
        vec![Self::AND, Self::OR]
    }

    fn iterator_method_identifiers(&self) -> Vec<&str> {
        vec![
            "filter",
            "map",
            "any",
            "all",
            "find",
            "position",
            "fold",
            "scan",
            "for_each",
            "filter_map",
            "flat_map",
            "inspect",
            "partition",
            "max_by",
            "min_by",
            "take_while",
            "skip_while",
            "try_fold",
            "try_for_each",
        ]
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        let function_node = node.child_by_field_name("function").unwrap();
        let function_kind = function_node.kind();

        match function_kind {
            Self::IDENTIFIER => (
                Some("".to_string()),
                node_source(&function_node, source_file),
            ),
            Self::FIELD_EXPRESSION => {
                let (receiver, object) = self.field_identifiers(source_file, &function_node);

                (Some(receiver), object)
            }
            Self::SCOPED_EXPRESSION => {
                let receiver =
                    if let Some(receiver_node) = function_node.child_by_field_name("path") {
                        node_source(&receiver_node, source_file)
                    } else {
                        Self::SELF.to_string()
                    };

                (
                    Some(receiver),
                    child_source(&function_node, "name", source_file),
                )
            }
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        (
            child_source(node, "value", source_file),
            child_source(node, "field", source_file),
        )
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_rust::language()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use tree_sitter::Tree;

    #[test]
    fn mutually_exclusive() {
        let lang = Rust::default();
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
        kinds.extend(lang.jump_nodes());
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
        let source_file = File::from_string("rust", "self.foo;");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Rust::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_write() {
        let source_file = File::from_string("rust", "self.foo = 1;");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let assignment = expression.named_child(0).unwrap();
        let field = assignment.named_child(0).unwrap();
        let language = Rust::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_collaborator() {
        let source_file = File::from_string("rust", "other.foo;");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Rust::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("other".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier() {
        let source_file = File::from_string("rust", "foo();");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Rust::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_self() {
        let source_file = File::from_string("rust", "self.foo();");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Rust::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("self".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_field() {
        let source_file = File::from_string("rust", "foo.bar();");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Rust::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".to_string()), "bar".to_string())
        );
    }

    #[test]
    fn call_scoped() {
        let source_file = File::from_string("rust", "Foo::bar();");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Rust::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("Foo".to_string()), "bar".to_string())
        );
    }

    fn call_node(tree: &Tree) -> Node<'_> {
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        expression.named_child(0).unwrap()
    }
}
