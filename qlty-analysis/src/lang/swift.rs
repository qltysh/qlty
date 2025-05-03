use crate::code::node_source;
use crate::code::File;
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
(class_declaration
    name: (type_identifier) @name) @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
(function_declaration
    name: (simple_identifier) @name) @definition.function
"#;

const FIELD_QUERY: &str = r#"
(property_declaration
    (pattern) @name) @field
"#;

pub struct Swift {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl Swift {
    pub const SELF: &'static str = "self";
    pub const BINARY: &'static str = "binary_expression";
    pub const BLOCK: &'static str = "code_block";
    pub const BREAK: &'static str = "break_statement";
    pub const CASE: &'static str = "switch_entry";
    pub const LINE_COMMENT: &'static str = "comment";
    pub const MULTILINE_COMMENT: &'static str = "multiline_comment";
    pub const CONTINUE: &'static str = "continue_statement";
    pub const FIELD_ACCESS: &'static str = "navigation_expression";
    pub const FIELD_DECLARATION: &'static str = "property_declaration";
    pub const FOR: &'static str = "for_statement";
    pub const NAVIGATION_SUFFIX: &'static str = "navigation_suffix";
    pub const NAVIGATION_EXPRESSION: &'static str = "navigation_expression";
    pub const FUNCTION_DECLARATION: &'static str = "function_declaration";
    pub const CALL_EXPRESSION: &'static str = "call_expression";
    pub const SIMPLE_IDENTIFIER: &'static str = "simple_identifier";
    pub const IF: &'static str = "if_statement";
    pub const LAMBDA: &'static str = "lambda_literal";
    pub const SOURCE_FILE: &'static str = "source_file";
    pub const RETURN: &'static str = "return_statement";
    pub const STRING_LITERAL: &'static str = "line_str_text";
    pub const RAW_STRING_LITERAL: &'static str = "raw_str_part";
    pub const MULTILINE_STRING_LITERAL: &'static str = "multi_line_str_text";
    pub const SWITCH: &'static str = "switch_statement";
    pub const TERNARY: &'static str = "ternary_expression";
    pub const TRY: &'static str = "do_statement";
    pub const CATCH: &'static str = "catch_clause";
    pub const WHILE: &'static str = "while_statement";
    pub const REPEAT_WHILE: &'static str = "repeat_while_statement";
    pub const DEFER: &'static str = "defer";

    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";
}

impl Default for Swift {
    fn default() -> Self {
        let language = tree_sitter_swift::language();

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

impl Language for Swift {
    fn name(&self) -> &str {
        "swift"
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

    fn if_nodes(&self) -> Vec<&str> {
        vec![Self::IF]
    }

    fn block_nodes(&self) -> Vec<&str> {
        vec![Self::BLOCK]
    }

    fn conditional_assignment_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::SOURCE_FILE]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::SWITCH]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec![Self::CASE]
    }

    fn ternary_nodes(&self) -> Vec<&str> {
        vec![Self::TERNARY]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::FOR, Self::WHILE, Self::REPEAT_WHILE]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![Self::CATCH]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![Self::TRY]
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
        vec![Self::FIELD_DECLARATION]
    }

    fn call_nodes(&self) -> Vec<&str> {
        vec![Self::CALL_EXPRESSION]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::FUNCTION_DECLARATION]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![Self::LAMBDA]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::LINE_COMMENT, Self::MULTILINE_COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![
            Self::STRING_LITERAL,
            Self::RAW_STRING_LITERAL,
            Self::MULTILINE_STRING_LITERAL,
        ]
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        match node.kind() {
            Self::CALL_EXPRESSION => {
                let function_node = node.child(0).unwrap();

                if function_node.kind() == Self::NAVIGATION_EXPRESSION {
                    // Method call on object (e.g., obj.method())
                    let object_node = function_node.child(0).unwrap();
                    let object_source = node_source(&object_node, source_file);

                    // Extract the method name from the navigation suffix
                    let suffix_node = function_node.child_by_field_name("suffix").unwrap();
                    let method_name_node = suffix_node.child(1).unwrap(); // The identifier after the dot
                    let method_name = node_source(&method_name_node, source_file);

                    (Some(object_source), method_name)
                } else {
                    // Simple function call
                    let function_name = node_source(&function_node, source_file);
                    (Some(Self::SELF.to_string()), function_name)
                }
            }
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        if node.kind() == Self::NAVIGATION_EXPRESSION {
            let object_node = node.child(0).unwrap();
            let object_source = node_source(&object_node, source_file);

            let suffix_node = node.child_by_field_name("suffix").unwrap();
            let property_node = suffix_node.child(1).unwrap(); // Identifier after the dot
            let property_name = node_source(&property_node, source_file);

            (object_source, property_name)
        } else {
            ("<UNKNOWN>".to_string(), "<UNKNOWN>".to_string())
        }
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_swift::language()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn mutually_exclusive() {
        let lang = Swift::default();
        let mut kinds: Vec<&str> = vec![];

        kinds.extend(lang.if_nodes());
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
        kinds.extend(lang.block_nodes());

        let unique: HashSet<_> = kinds.iter().cloned().collect();
        assert_eq!(unique.len(), kinds.len());
    }

    #[test]
    fn field_identifier_read() {
        let source_file = File::from_string("swift", "self.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let language = Swift::default();

        assert_eq!(
            language.field_identifiers(&source_file, &expression),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_collaborator() {
        let source_file = File::from_string("swift", "other.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let language = Swift::default();

        assert_eq!(
            language.field_identifiers(&source_file, &expression),
            ("other".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier() {
        let source_file = File::from_string("swift", "foo()");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let language = Swift::default();

        assert_eq!(
            language.call_identifiers(&source_file, &expression),
            (Some("self".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_member() {
        let source_file = File::from_string("swift", "foo.bar()");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let language = Swift::default();

        assert_eq!(
            language.call_identifiers(&source_file, &expression),
            (Some("foo".to_string()), "bar".to_string())
        );
    }

    #[test]
    fn nested_field_access() {
        let source_file = File::from_string("swift", "obj.nestedObj.oneMoreObj");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let language = Swift::default();

        assert_eq!(
            language.field_identifiers(&source_file, &expression),
            ("obj.nestedObj".to_string(), "oneMoreObj".to_string())
        );
    }
}
