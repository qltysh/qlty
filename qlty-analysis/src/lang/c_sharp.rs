use crate::code::node_source;
use crate::code::File;
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
[
  (class_declaration 
    name: (identifier) @name)
  (interface_declaration 
    name: (identifier) @name)
] @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
[
    (method_declaration
        name: (identifier) @name
        parameters: (_) @parameters)
    (constructor_declaration
        name: (identifier) @name
        parameters: (_) @parameters)
] @definition.function
"#;

const FIELD_QUERY: &str = r#"
(field_declaration
    declarator: (variable_declarator
        name: (identifier) @name)) @field
"#;

pub struct CSharp {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl CSharp {
    pub const SELF: &'static str = "this";
    pub const BINARY: &'static str = "binary_expression";
    pub const BLOCK: &'static str = "block";
    pub const BREAK: &'static str = "break_statement";
    pub const CATCH: &'static str = "catch_clause";
    pub const CASE: &'static str = "switch_block_statement_group";
    pub const LINE_COMMENT: &'static str = "line_comment";
    pub const BLOCK_COMMENT: &'static str = "block_comment";
    pub const CONTINUE: &'static str = "continue_statement";
    pub const DO: &'static str = "do_statement";
    pub const FIELD_ACCESS: &'static str = "field_access";
    pub const FIELD_DECLARATION: &'static str = "field_declaration";
    pub const FOR_IN: &'static str = "enhanced_for_statement";
    pub const FOR: &'static str = "for_statement";
    pub const METHOD_DECLARATION: &'static str = "method_declaration";
    pub const METHOD_INVOCATION: &'static str = "method_invocation";
    pub const IDENTIFIER: &'static str = "identifier";
    pub const IF: &'static str = "if_statement";
    pub const LAMBDA: &'static str = "lambda_expression";
    pub const PROGRAM: &'static str = "program";
    pub const RETURN: &'static str = "return_statement";
    pub const STRING: &'static str = "string_literal";
    pub const SWITCH: &'static str = "switch_expression";
    pub const TEMPLATE_STRING: &'static str = "template_expression";
    pub const TERNARY: &'static str = "ternary_expression";
    pub const TRY: &'static str = "try_statement";
    pub const TRY_WITH_RESOURCES: &'static str = "try_with_resources_statement";
    pub const WHILE: &'static str = "while_statement";

    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";
}

impl Default for CSharp {
    fn default() -> Self {
        let language = tree_sitter_c_sharp::language();

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

impl Language for CSharp {
    fn name(&self) -> &str {
        "c#"
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
        vec![Self::PROGRAM]
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
        vec![Self::FOR, Self::FOR_IN, Self::WHILE, Self::DO]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![Self::CATCH]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![Self::TRY, Self::TRY_WITH_RESOURCES]
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
        vec![Self::METHOD_INVOCATION]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::METHOD_DECLARATION]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![Self::LAMBDA]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::LINE_COMMENT, Self::BLOCK_COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING, Self::TEMPLATE_STRING]
    }

    fn is_jump_label(&self, node: &Node) -> bool {
        node.kind() == Self::IDENTIFIER
    }

    fn has_labeled_jumps(&self) -> bool {
        true
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        match node.kind() {
            Self::METHOD_INVOCATION => {
                let (receiver, object) = self.field_identifiers(source_file, node);

                (Some(receiver), object)
            }
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        let object_node = node.child_by_field_name("object");
        let property_node = node
            .child_by_field_name("name")
            .or_else(|| node.child_by_field_name("field"));

        match (&object_node, &property_node) {
            (Some(obj), Some(prop)) if obj.kind() == Self::FIELD_ACCESS => {
                let object_source =
                    get_node_source_or_default(obj.child_by_field_name("field"), source_file);
                let property_source = get_node_source_or_default(Some(*prop), source_file);
                (object_source, property_source)
            }
            (Some(obj), Some(prop)) => (
                get_node_source_or_default(Some(*obj), source_file),
                get_node_source_or_default(Some(*prop), source_file),
            ),
            (None, Some(prop)) => (
                Self::SELF.to_owned(),
                get_node_source_or_default(Some(*prop), source_file),
            ),
            _ => ("<UNKNOWN>".to_string(), "<UNKNOWN>".to_string()),
        }
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_c_sharp::language()
    }
}

fn get_node_source_or_default(node: Option<Node>, source_file: &File) -> String {
    node.as_ref()
        .map(|n| node_source(n, source_file))
        .unwrap_or("<UNKNOWN>".to_string())
}
