use crate::code::node_source;
use crate::code::File;
use tree_sitter::Node;

pub const COMMON_CLASS_QUERY: &str = r#"
[
    (class_declaration
        name: (type_identifier) @name)
    (interface_declaration
        name: (type_identifier) @name)
] @definition.class
"#;

pub const COMMON_FUNCTION_DECLARATION_QUERY: &str = r#"
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
"#;

pub const COMMON_FIELD_QUERY: &str = r#"
[
    (public_field_definition
        name: (property_identifier) @name
        type: (type_annotation) @type)

    (member_expression
        object: (this) @receiver
        property: (_) @name)
] @field
"#;

pub struct TypeScriptCommon {
    language: tree_sitter::Language,
}

impl TypeScriptCommon {
    pub const SELF: &'static str = "this";
    pub const BINARY: &'static str = "binary_expression";
    pub const BREAK: &'static str = "break_statement";
    pub const CALL: &'static str = "call_expression";
    pub const CALL_EXPRESSION: &'static str = "call_expression";
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
    pub const INTERFACE_DECLARATION: &'static str = "interface_declaration";
    pub const MEMBER_EXPRESSION: &'static str = "member_expression";
    pub const PRIVATE_FIELD: &'static str = "private_field_definition";
    pub const PROGRAM: &'static str = "program";
    pub const PROPERTY_SIGNATURE: &'static str = "property_signature";
    pub const PROTECTED_FIELD: &'static str = "protected_field_definition";
    pub const PUBLIC_FIELD: &'static str = "public_field_definition";
    pub const RETURN: &'static str = "return_statement";
    pub const STRING: &'static str = "string";
    pub const SWITCH: &'static str = "switch_statement";
    pub const TEMPLATE_STRING: &'static str = "template_string";
    pub const TERNARY: &'static str = "ternary_expression";
    pub const WHILE: &'static str = "while_statement";
    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";

    pub fn new(language: &tree_sitter::Language) -> Self {
        let language = language.clone();

        Self { language }
    }

    pub fn language(&self) -> tree_sitter::Language {
        self.language.clone()
    }

    pub fn self_keyword(&self) -> Option<&str> {
        Some(Self::SELF)
    }

    pub fn class_query(&self) -> tree_sitter::Query {
        tree_sitter::Query::new(&self.language, COMMON_CLASS_QUERY).unwrap()
    }

    pub fn function_declaration_query(&self) -> tree_sitter::Query {
        tree_sitter::Query::new(
            &self.language,
            &format!(
                "[{}] @definition.function",
                COMMON_FUNCTION_DECLARATION_QUERY
            ),
        )
        .unwrap()
    }

    pub fn field_query(&self) -> tree_sitter::Query {
        tree_sitter::Query::new(&self.language, COMMON_FIELD_QUERY).unwrap()
    }

    pub fn constructor_names(&self) -> Vec<&str> {
        vec!["constructor"]
    }

    pub fn if_nodes(&self) -> Vec<&str> {
        vec![Self::IF]
    }

    pub fn else_nodes(&self) -> Vec<&str> {
        vec![Self::ELSE]
    }

    pub fn conditional_assignment_nodes(&self) -> Vec<&str> {
        vec!["augmented_assignment_expression"]
    }

    pub fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::PROGRAM]
    }

    pub fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::SWITCH]
    }

    pub fn case_nodes(&self) -> Vec<&str> {
        vec!["switch_case", "switch_default"]
    }

    pub fn ternary_nodes(&self) -> Vec<&str> {
        vec![Self::TERNARY]
    }

    pub fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::FOR, Self::FOR_IN, Self::WHILE, Self::DO]
    }

    pub fn except_nodes(&self) -> Vec<&str> {
        vec![Self::CATCH]
    }

    pub fn try_expression_nodes(&self) -> Vec<&str> {
        vec!["try_statement"]
    }

    pub fn jump_nodes(&self) -> Vec<&str> {
        vec![Self::BREAK, Self::CONTINUE]
    }

    pub fn return_nodes(&self) -> Vec<&str> {
        vec![Self::RETURN]
    }

    pub fn binary_nodes(&self) -> Vec<&str> {
        vec![Self::BINARY]
    }

    pub fn boolean_operator_nodes(&self) -> Vec<&str> {
        vec![Self::AND, Self::OR]
    }

    pub fn short_circuit_operators(&self) -> Vec<&str> {
        vec![Self::AND, Self::OR, "??"]
    }

    pub fn field_nodes(&self) -> Vec<&str> {
        vec![
            Self::MEMBER_EXPRESSION,
            Self::PUBLIC_FIELD,
            Self::PRIVATE_FIELD,
            Self::PROTECTED_FIELD,
            Self::PROPERTY_SIGNATURE,
        ]
    }

    pub fn call_nodes(&self) -> Vec<&str> {
        vec![Self::CALL]
    }

    pub fn function_nodes(&self) -> Vec<&str> {
        vec![Self::FUNCTION_DECLARATION, "method_definition"]
    }

    pub fn closure_nodes(&self) -> Vec<&str> {
        vec!["arrow_function"]
    }

    pub fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::COMMENT]
    }

    pub fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING, Self::TEMPLATE_STRING]
    }

    pub fn iterator_method_identifiers(&self) -> Vec<&str> {
        vec![
            "map",
            "filter",
            "forEach",
            "some",
            "every",
            "find",
            "reduce",
            "findIndex",
            "flat",
            "flatMap",
        ]
    }

    pub fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        let function_node = node.child_by_field_name("function");
        match function_node.as_ref().map(|n| n.kind()) {
            Some(Self::IDENTIFIER) => (
                Some(Self::SELF.to_string()),
                get_node_source_or_default(function_node, source_file),
            ),
            Some(Self::MEMBER_EXPRESSION) => {
                let (receiver, object) =
                    self.field_identifiers(source_file, &function_node.unwrap());

                (Some(receiver), object)
            }
            Some(Self::CALL_EXPRESSION) => (
                Some(get_node_source_or_default(function_node, source_file)),
                "<UNKNOWN>".to_string(),
            ),
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }

    pub fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        let object_node = node.child_by_field_name("object");
        let property_node = node.child_by_field_name("property");

        match (&object_node, &property_node) {
            (Some(obj), Some(prop)) if obj.kind() == Self::MEMBER_EXPRESSION => {
                let object_source =
                    get_node_source_or_default(obj.child_by_field_name("property"), source_file);
                let property_source = get_node_source_or_default(Some(*prop), source_file);
                (object_source, property_source)
            }
            (Some(obj), Some(prop)) => (
                get_node_source_or_default(Some(*obj), source_file),
                get_node_source_or_default(Some(*prop), source_file),
            ),
            _ => ("<UNKNOWN>".to_string(), "<UNKNOWN>".to_string()),
        }
    }
}

fn get_node_source_or_default(node: Option<Node>, source_file: &File) -> String {
    node.as_ref()
        .map(|n| node_source(n, source_file))
        .unwrap_or("<UNKNOWN>".to_string())
}
