use crate::code::node_source;
use crate::code::File;
use crate::lang::Language;
use anyhow::Context;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
[
    (class_declaration
        name: (name) @name)
    (trait_declaration
        name: (name) @name)
    (interface_declaration
        name: (name) @name)
] @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
[
    (method_declaration
        name: (name) @name
        parameters: (_) @parameters)
    (function_definition
        name: (name) @name
        parameters: (_) @parameters)
    (assignment_expression
        left:
            name: (_) @name
        right: [
            (arrow_function
                parameters: (_) @parameters)
            (anonymous_function_creation_expression
                parameters: (_) @parameters)
        ])
] @definition.function
"#;

const FIELD_QUERY: &str = r#"
[
    (class_declaration
        name: (name)
        body: (declaration_list
            (property_declaration
                (property_element
                    (variable_name (_) @name)
                    (property_initializer
                    	(_) @field)
                )
            )
        )
    )
    (assignment_expression
      left: (member_access_expression
          object: (_) @obj.name
          name: (_) @name
          (#eq? @obj.name "$this")
      )
      right: (_) @field
    )
    (member_access_expression
        object: (_) @obj.name
        name: (_) @name
        (#eq? @obj.name "$this")
    ) @field
]
"#;

pub struct Php {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl Php {
    pub const NAME: &'static str = "php";
    pub const THIS: &'static str = "$this";

    // Constants for different node types
    pub const BINARY_EXPRESSION: &'static str = "binary_expression";
    pub const BOOLEAN_OPERATOR: &'static str = "boolean_operator";
    pub const CALL_EXPRESSION: &'static str = "function_call_expression";
    pub const CASE_STATEMENT: &'static str = "case_statement";
    pub const ARROW_FUNCTION: &'static str = "arrow_function";
    pub const ANONYMOUS_FUNCTION: &'static str = "anonymous_function_creation_expression";
    pub const COMMENT: &'static str = "comment";
    pub const CONDITIONAL_ASSIGNMENT: &'static str = "conditional_assignment";
    pub const CATCH_CLAUSE: &'static str = "catch_clause";
    pub const ELSE_CLAUSE: &'static str = "else_clause";
    pub const ELSE_IF_CLAUSE: &'static str = "else_if_clause";
    pub const FUNCTION_DEFINITION: &'static str = "function_definition";
    pub const METHOD_DECLARATION: &'static str = "method_declaration";
    pub const IF_STATEMENT: &'static str = "if_statement";
    pub const IDENTIFIER: &'static str = "identifier";
    pub const WHILE_STATEMENT: &'static str = "while_statement";
    pub const FOR_STATEMENT: &'static str = "for_statement";
    pub const FOREACH_STATEMENT: &'static str = "foreach_statement";
    pub const DO_WHILE_STATEMENT: &'static str = "do_statement";
    pub const SWITCH_STATEMENT: &'static str = "switch_statement";
    pub const RETURN_STATEMENT: &'static str = "return_statement";
    pub const BREAK_STATEMENT: &'static str = "break_statement";
    pub const CONTINUE_STATEMENT: &'static str = "continue_statement";
    pub const TERNARY_EXPRESSION: &'static str = "conditional_assignment";
    pub const TRY_STATEMENT: &'static str = "try_statement";
    pub const STRING: &'static str = "string";
    pub const ENCASPED_STRING: &'static str = "encapsed_string";
    pub const PROGRAM: &'static str = "program";
    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";
    pub const CONSTRUCTOR_NAME: &'static str = "__construct";
    pub const MEMBER_CALL_EXPRESSION: &'static str = "member_call_expression";
    pub const MEMBER_ACCESS_EXPRESSION: &'static str = "member_access_expression";
}

impl Default for Php {
    fn default() -> Self {
        let language = tree_sitter_php::language_php();

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

impl Language for Php {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn self_keyword(&self) -> Option<&str> {
        Some(Self::THIS)
    }

    fn binary_nodes(&self) -> Vec<&str> {
        vec![Self::BINARY_EXPRESSION]
    }

    fn boolean_operator_nodes(&self) -> Vec<&str> {
        vec![Self::AND, Self::OR]
    }

    fn call_nodes(&self) -> Vec<&str> {
        vec![Self::CALL_EXPRESSION]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec![Self::CASE_STATEMENT]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![Self::ARROW_FUNCTION, Self::ANONYMOUS_FUNCTION]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::COMMENT]
    }

    fn conditional_assignment_nodes(&self) -> Vec<&str> {
        vec![Self::CONDITIONAL_ASSIGNMENT]
    }

    fn else_nodes(&self) -> Vec<&str> {
        vec![Self::ELSE_CLAUSE]
    }

    fn elsif_nodes(&self) -> Vec<&str> {
        vec![Self::ELSE_IF_CLAUSE]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![Self::CATCH_CLAUSE]
    }

    fn field_nodes(&self) -> Vec<&str> {
        vec![Self::MEMBER_CALL_EXPRESSION, Self::MEMBER_ACCESS_EXPRESSION]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::FUNCTION_DEFINITION, Self::METHOD_DECLARATION]
    }

    fn if_nodes(&self) -> Vec<&str> {
        vec![Self::IF_STATEMENT]
    }

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::PROGRAM]
    }

    fn jump_nodes(&self) -> Vec<&str> {
        vec![Self::BREAK_STATEMENT, Self::CONTINUE_STATEMENT]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![
            Self::WHILE_STATEMENT,
            Self::FOR_STATEMENT,
            Self::FOREACH_STATEMENT,
            Self::DO_WHILE_STATEMENT,
        ]
    }

    fn return_nodes(&self) -> Vec<&str> {
        vec![Self::RETURN_STATEMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING, Self::ENCASPED_STRING]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::SWITCH_STATEMENT]
    }

    fn ternary_nodes(&self) -> Vec<&str> {
        vec![Self::TERNARY_EXPRESSION]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![Self::TRY_STATEMENT]
    }

    fn constructor_names(&self) -> Vec<&str> {
        vec![Self::CONSTRUCTOR_NAME]
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        let function_node = node.child(0).unwrap();
        let function_kind = function_node.kind();

        match function_kind {
            Self::CALL_EXPRESSION => (
                None,
                node_source(&function_node.child(0).unwrap(), source_file),
            ),
            Self::MEMBER_CALL_EXPRESSION => {
                let object = function_node.child(0).unwrap();
                let object_name = node_source(&object, source_file);
                let method = function_node.child(2).unwrap();
                let method_name = node_source(&method, source_file);
                (Some(object_name), method_name)
            }
            _ => (Some("<UNKNOWN>".to_string()), String::new()),
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        (
            node_source(&node.child(0).unwrap(), source_file),
            node_source(
                &node
                    .child_by_field_name("name")
                    .with_context(|| format!("file_path: {:?}", source_file.path))
                    .unwrap(),
                source_file,
            ),
        )
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_php::language_php()
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
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use tree_sitter::Tree;

    #[test]
    fn mutually_exclusive() {
        let php = Php::default();
        let mut kinds: Vec<&str> = vec![];

        kinds.extend(php.binary_nodes());
        kinds.extend(php.boolean_operator_nodes());
        kinds.extend(php.call_nodes());
        kinds.extend(php.case_nodes());
        kinds.extend(php.closure_nodes());
        kinds.extend(php.comment_nodes());
        kinds.extend(php.conditional_assignment_nodes());
        kinds.extend(php.else_nodes());
        kinds.extend(php.except_nodes());
        kinds.extend(php.field_nodes());
        kinds.extend(php.function_nodes());
        kinds.extend(php.if_nodes());
        kinds.extend(php.invisible_container_nodes());
        kinds.extend(php.jump_nodes());
        kinds.extend(php.loop_nodes());
        kinds.extend(php.return_nodes());
        kinds.extend(php.string_nodes());
        kinds.extend(php.switch_nodes());
        kinds.extend(php.try_expression_nodes());

        let unique: HashSet<_> = kinds.iter().cloned().collect();
        assert_eq!(unique.len(), kinds.len());
    }

    #[test]
    fn field_identifier_read() {
        let source_file = File::from_string(Php::NAME, "<?php $foobar->foo;?>");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(1).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Php::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("$foobar".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_write() {
        let source_file = File::from_string(Php::NAME, "<?php $foobar->foo = 1;?>");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(1).unwrap();
        let field = expression.named_child(0).unwrap().named_child(0).unwrap();
        let language = Php::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("$foobar".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn this_field_identifier_read() {
        let source_file = File::from_string(Php::NAME, "<?php $this->foo;?>");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(1).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Php::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("$this".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn this_field_identifier_write() {
        let source_file = File::from_string(Php::NAME, "<?php $this->foo = 1;?>");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(1).unwrap();
        let field = expression.named_child(0).unwrap().named_child(0).unwrap();
        let language = Php::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("$this".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier() {
        let source_file = File::from_string(Php::NAME, "<?php foo(); ?>");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Php::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (None, "foo".to_string())
        );
    }

    #[test]
    fn call_attribute() {
        let source_file = File::from_string(Php::NAME, "<?php $foo->bar() ?>");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Php::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("$foo".to_string()), "bar".to_string())
        );
    }

    #[test]
    fn this_attribute() {
        let source_file = File::from_string(Php::NAME, "<?php $this->bar() ?>");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Php::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("$this".to_string()), "bar".to_string())
        );
    }

    fn call_node(tree: &Tree) -> Node<'_> {
        let root_node = tree.root_node();
        root_node.named_child(1).unwrap()
    }
}
