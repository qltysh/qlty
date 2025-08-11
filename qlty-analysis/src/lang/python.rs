use crate::code::File;
use crate::code::{child_source, node_source};
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
(class_definition
    name: (identifier) @name) @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
(function_definition
    name: (identifier) @name
    parameters: (_) @parameters) @definition.function
"#;

const FIELD_QUERY: &str = r#"
(
    (attribute
        object: (identifier) @receiver
        attribute: (_) @name) @field
    (#eq? @receiver "self")
)
"#;

pub struct Python {
    pub class_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
}

impl Python {
    pub const SELF: &'static str = "self";

    pub const ATTRIBUTE: &'static str = "attribute";
    pub const BOOLEAN: &'static str = "boolean_operator";
    pub const BREAK: &'static str = "break_statement";
    pub const CALL: &'static str = "call";
    pub const COMMENT: &'static str = "comment";
    pub const CONDITIONAL_EXPRESSION: &'static str = "conditional_expression"; // ternary
    pub const CONTINUE: &'static str = "continue_statement";
    pub const ELIF: &'static str = "elif_clause";
    pub const ELSE: &'static str = "else_clause";
    pub const EXCEPT: &'static str = "except_clause";
    pub const FOR: &'static str = "for_statement";
    pub const FUNCTION: &'static str = "function_definition";
    pub const IDENTIFIER: &'static str = "identifier";
    pub const IF: &'static str = "if_statement";
    pub const LAMBDA: &'static str = "lambda";
    pub const MATCH: &'static str = "match_statement";
    pub const MODULE: &'static str = "module";
    pub const RETURN: &'static str = "return_statement";
    pub const STRING: &'static str = "string";
    pub const WHILE: &'static str = "while_statement";

    pub const AND: &'static str = "and";
    pub const OR: &'static str = "or";
}

impl Default for Python {
    fn default() -> Self {
        let language = tree_sitter_python::language();

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

impl Language for Python {
    fn name(&self) -> &str {
        "python"
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
        vec!["__init__"]
    }

    fn destructor_names(&self) -> Vec<&str> {
        vec!["__del__"]
    }

    fn is_instance_method(&self, file: &File, node: &Node) -> bool {
        let parameters = node.child_by_field_name("parameters").unwrap();

        if let Some(first_parameter) = parameters.named_child(0) {
            first_parameter.kind() == Self::IDENTIFIER
                && node_source(&first_parameter, file) == Self::SELF
        } else {
            false
        }
    }

    fn if_nodes(&self) -> Vec<&str> {
        vec![Self::IF]
    }

    fn elsif_nodes(&self) -> Vec<&str> {
        vec![Self::ELIF]
    }

    fn else_nodes(&self) -> Vec<&str> {
        vec![Self::ELSE]
    }

    fn conditional_assignment_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::MODULE]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::MATCH]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec!["case_clause"]
    }

    fn ternary_nodes(&self) -> Vec<&str> {
        vec![Self::CONDITIONAL_EXPRESSION]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::FOR, Self::WHILE]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![Self::EXCEPT]
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
        vec![Self::BOOLEAN]
    }

    fn boolean_operator_nodes(&self) -> Vec<&str> {
        vec![Self::AND, Self::OR]
    }

    fn field_nodes(&self) -> Vec<&str> {
        vec![Self::ATTRIBUTE]
    }

    fn call_nodes(&self) -> Vec<&str> {
        vec![Self::CALL]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::FUNCTION]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![Self::LAMBDA]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING]
    }

    fn all_operators(&self) -> Vec<&str> {
        // Import | DOT | From | COMMA | As | STAR | GTGT | Assert | COLONEQ | Return | Def
        // | Del | Raise | Pass | Break | Continue | If | Elif | Else | Async | For | In
        // | While | Try | Except | Finally | With | DASHGT | EQ | Global | Exec | AT | Not
        // | And | Or | PLUS | DASH | SLASH | PERCENT | SLASHSLASH | STARSTAR | PIPE | AMP
        // | CARET | LTLT | TILDE | LT | LTEQ | EQEQ | BANGEQ | GTEQ | GT | LTGT | Is | PLUSEQ
        // | DASHEQ | STAREQ | SLASHEQ | ATEQ | SLASHSLASHEQ | PERCENTEQ | STARSTAREQ | GTGTEQ
        // | LTLTEQ | AMPEQ | CARETEQ | PIPEEQ | Yield | Await | Await2 | Print
        vec![Self::ATTRIBUTE, Self::IDENTIFIER]
    }

    fn all_operands(&self) -> Vec<&str> {
        vec![
            "false",
            "float",
            "identifier",
            "integer",
            "none",
            "string",
            "true",
        ]
    }

    fn is_decorator_function(&self, node: &Node) -> bool {
        let body = node.child_by_field_name("body").unwrap();

        if body.named_child_count() == 2 {
            let first_child = body.named_child(0).unwrap();
            let second_child = body.named_child(1).unwrap();

            first_child.kind() == Self::FUNCTION && second_child.kind() == Self::RETURN
        } else {
            false
        }
    }

    fn iterator_method_identifiers(&self) -> Vec<&str> {
        vec!["filter", "map", "any", "all"]
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        let function_node = node.child_by_field_name("function").unwrap();
        let function_kind = function_node.kind();

        match function_kind {
            Self::IDENTIFIER => (
                Some(Self::SELF.to_string()),
                node_source(&function_node, source_file),
            ),
            Self::ATTRIBUTE => {
                let (receiver, object) = self.field_identifiers(source_file, &function_node);

                (Some(receiver), object)
            }
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        (
            child_source(node, "object", source_file),
            child_source(node, "attribute", source_file),
        )
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_python::language()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use tree_sitter::Tree;

    #[test]
    fn mutually_exclusive() {
        let lang = Python::default();
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
        let source_file = File::from_string("python", "self.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Python::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_write() {
        let source_file = File::from_string("python", "self.foo = 1");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let assignment = expression.named_child(0).unwrap();
        let field = assignment.named_child(0).unwrap();
        let language = Python::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_collaborator() {
        let source_file = File::from_string("python", "other.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Python::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("other".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier() {
        let source_file = File::from_string("python", "foo()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Python::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("self".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_attribute() {
        let source_file = File::from_string("python", "foo.bar()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Python::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".to_string()), "bar".to_string())
        );
    }

    #[test]
    fn decorator() {
        let source_file = File::from_string(
            "python",
            r#"
            def foo():
                def bar():
                    pass
                return bar
        "#,
        );
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let function = root_node.named_child(0).unwrap();
        let language = Python::default();

        assert_eq!(language.is_decorator_function(&function), true);
    }

    #[test]
    fn not_decorator() {
        let source_file = File::from_string(
            "python",
            r#"
            def foo():
                x = 1 # Invalidates decorator exception
                def bar():
                    pass
                return bar
        "#,
        );
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let function = root_node.named_child(0).unwrap();
        let language = Python::default();

        assert_eq!(language.is_decorator_function(&function), false);
    }

    fn call_node(tree: &Tree) -> Node<'_> {
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        expression.named_child(0).unwrap()
    }
}
