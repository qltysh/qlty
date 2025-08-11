use crate::code::File;
use crate::code::{child_source, node_source};
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
[
    (class
        name: (_) @name)

    (class_declaration
        name: (_) @name)
] @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
[
    (method_definition
        name: (property_identifier) @name
        parameters: (_) @parameters)

    (function_expression
            name: (identifier) @name
            parameters: (_) @parameters)

    (function_declaration
        name: (identifier) @name
        parameters: (_) @parameters)

    (generator_function
        name: (identifier) @name
        parameters: (_) @parameters)

    (generator_function_declaration
        name: (identifier) @name
        parameters: (_) @parameters)

    (lexical_declaration
        (variable_declarator
            name: (identifier) @name
            value: [
                (arrow_function
                    parameters: (_) @parameters)
                (function_expression
                    parameters: (_) @parameters)
            ]))

    (variable_declaration
        (variable_declarator
            name: (identifier) @name
            value: [
                (arrow_function
                    parameters: (_) @parameters)
                (function_expression
                    parameters: (_) @parameters)
            ]))

    (assignment_expression
        left: [
            (identifier) @name
            (member_expression
                property: (property_identifier) @name)
        ]
        right: [
            (arrow_function
                parameters: (_) @parameters)
            (function_expression
                parameters: (_) @parameters)
        ])

    (pair
        key: (property_identifier) @name
        value: [
            (arrow_function
                parameters: (_) @parameters)
            (function_expression
                parameters: (_) @parameters)
        ])
    (jsx_attribute
      (property_identifier) @name
      (jsx_expression
        (arrow_function
          (formal_parameters) @parameters)))
] @definition.function
"#;

const FIELD_QUERY: &str = r#"
[
    (class_declaration
        name: (identifier)
        body: (class_body
            member: (field_definition
                property: (property_identifier) @name)))

    (member_expression
        object: (this) @receiver
        property: (_) @name)
] @field
"#;

pub struct JavaScript {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl JavaScript {
    pub const SELF: &'static str = "this";

    pub const BINARY: &'static str = "binary_expression";
    pub const BREAK: &'static str = "break_statement";
    pub const CALL: &'static str = "call_expression";
    pub const CATCH: &'static str = "catch_clause";
    pub const COMMENT: &'static str = "comment";
    pub const CONTINUE: &'static str = "continue_statement";
    pub const DO: &'static str = "do_statement";
    pub const ELSE: &'static str = "else_clause";
    pub const FOR_IN: &'static str = "for_in_statement";
    pub const FOR: &'static str = "for_statement";
    pub const FUNCTION_DECLARATION: &'static str = "function_declaration";
    pub const IDENTIFIER: &'static str = "identifier";
    pub const IF: &'static str = "if_statement";
    pub const MEMBER_EXPRESSION: &'static str = "member_expression";
    pub const PROGRAM: &'static str = "program";
    pub const RETURN: &'static str = "return_statement";
    pub const STRING: &'static str = "string";
    pub const SWITCH: &'static str = "switch_statement";
    pub const TEMPLATE_STRING: &'static str = "template_string";
    pub const TERNARY: &'static str = "ternary_expression";
    pub const WHILE: &'static str = "while_statement";

    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";
}

impl Default for JavaScript {
    fn default() -> Self {
        let language = tree_sitter_javascript::language();

        Self {
            class_query: tree_sitter::Query::new(&language, CLASS_QUERY).unwrap(),
            field_query: tree_sitter::Query::new(&language, FIELD_QUERY).unwrap(),
            function_declaration_query: tree_sitter::Query::new(
                &language,
                FUNCTION_DECLARATION_QUERY,
            )
            .unwrap(),
        }
    }
}

impl Language for JavaScript {
    fn name(&self) -> &str {
        "javascript"
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

    fn constructor_names(&self) -> Vec<&str> {
        vec!["constructor"]
    }

    fn if_nodes(&self) -> Vec<&str> {
        vec![Self::IF]
    }

    fn else_nodes(&self) -> Vec<&str> {
        vec![Self::ELSE]
    }

    fn conditional_assignment_nodes(&self) -> Vec<&str> {
        vec!["augmented_assignment_expression"]
    }

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::PROGRAM]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::SWITCH]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec!["switch_case", "switch_default"]
    }

    fn ternary_nodes(&self) -> Vec<&str> {
        vec![Self::TERNARY]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::FOR, Self::FOR_IN, Self::WHILE, Self::DO]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![Self::CATCH]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![]
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

    fn boolean_operator_nodes(&self) -> Vec<&str> {
        vec![Self::AND, Self::OR]
    }

    fn field_nodes(&self) -> Vec<&str> {
        vec![Self::MEMBER_EXPRESSION]
    }

    fn call_nodes(&self) -> Vec<&str> {
        vec![Self::CALL]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::FUNCTION_DECLARATION]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING, Self::TEMPLATE_STRING]
    }

    fn iterator_method_identifiers(&self) -> Vec<&str> {
        vec![
            "map",
            "filter",
            "forEach",
            "reduce",
            "some",
            "every",
            "find",
            "findIndex",
            "flat",
            "flatMap",
        ]
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        let function_node = node.child_by_field_name("function").unwrap();
        let function_kind = function_node.kind();

        match function_kind {
            Self::IDENTIFIER => (
                Some(Self::SELF.to_string()),
                node_source(&function_node, source_file),
            ),
            Self::MEMBER_EXPRESSION => {
                let (receiver, object) = self.field_identifiers(source_file, &function_node);

                (Some(receiver), object)
            }
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }
    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        (
            child_source(node, "object", source_file),
            child_source(node, "property", source_file),
        )
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_javascript::language()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use tree_sitter::Tree;

    #[test]
    fn mutually_exclusive() {
        let lang = JavaScript::default();
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
        let source_file = File::from_string("javascript", "self.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = JavaScript::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_write() {
        let source_file = File::from_string("javascript", "self.foo = 1");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let assignment = expression.named_child(0).unwrap();
        let field = assignment.named_child(0).unwrap();
        let language = JavaScript::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_collaborator() {
        let source_file = File::from_string("javascript", "other.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = JavaScript::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("other".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier() {
        let source_file = File::from_string("javascript", "foo()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = JavaScript::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("this".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_member() {
        let source_file = File::from_string("javascript", "foo.bar()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = JavaScript::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".into()), "bar".into())
        );
    }

    fn call_node(tree: &Tree) -> Node<'_> {
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        expression.named_child(0).unwrap()
    }
}
