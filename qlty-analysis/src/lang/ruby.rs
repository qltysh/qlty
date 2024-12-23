use crate::code::File;
use crate::code::{child_source, node_source};
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
[
(class
    name: [
    (constant) @name
    (scope_resolution
        name: (_) @name)
    ]) @definition.class
(singleton_class
    value: [
    (constant) @name
    (scope_resolution
        name: (_) @name)
    ]) @definition.class
]
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
[
    (method
        name: (_) @name
        parameters: (_)? @parameters)
    (singleton_method
        object: (_)
        name: (_) @name
        parameters: (_)? @parameters)
] @definition.function
"#;

const FIELD_QUERY: &str = r#"
[
    (call
        method: (identifier) @accessor_name
        arguments: (argument_list (simple_symbol) @name)
    (#eq? @accessor_name "attr_accessor"))

    (call
        method: (identifier) @writer_name
        arguments: (argument_list (simple_symbol) @name)
    (#eq? @writer_name "attr_writer"))

    (call
        method: (identifier) @reader_name
        arguments: (argument_list (simple_symbol) @name)
    (#eq? @reader_name "attr_reader"))

    (call
        receiver: (constant) @receiver_name
        method: (identifier) @method_name
        arguments: (argument_list (simple_symbol) @name))
    (instance_variable) @name
    (class_variable) @name
] @field
"#;

pub struct Ruby {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl Ruby {
    pub const BEGIN: &'static str = "begin";
    pub const BINARY: &'static str = "binary";
    pub const BLOCK: &'static str = "block";
    pub const BREAK: &'static str = "break";
    pub const CALL: &'static str = "call";
    pub const CASE: &'static str = "case";
    pub const CLASS_VARIABLE: &'static str = "class_variable";
    pub const COMMENT: &'static str = "comment";
    pub const CONDITIONAL: &'static str = "conditional";
    pub const CONSTANT: &'static str = "constant";
    pub const DO_BLOCK: &'static str = "do_block";
    pub const ELSE: &'static str = "else";
    pub const ELSIF: &'static str = "elsif";
    pub const FOR: &'static str = "for";
    pub const GLOBAL_VARIABLE: &'static str = "global_variable";
    pub const IDENTIFIER: &'static str = "identifier";
    pub const IF: &'static str = "if";
    pub const UNLESS: &'static str = "unless";
    pub const IF_MODIFIER: &'static str = "if_modifier"; // statement _modifier_ aka trailing if
    pub const UNLESS_MODIFIER: &'static str = "unless_modifier"; // statement _modifier_ aka trailing unless
    pub const INITIALIZE: &'static str = "initialize";
    pub const INSTANCE_VARIABLE: &'static str = "instance_variable";
    pub const METHOD_CALL: &'static str = "method_call";
    pub const METHOD: &'static str = "method";
    pub const NEXT: &'static str = "next";
    pub const OPERATOR_ASSIGNMENT: &'static str = "operator_assignment";
    pub const PROGRAM: &'static str = "program";
    pub const RESCUE: &'static str = "rescue";
    pub const RETURN: &'static str = "return";
    pub const SELF: &'static str = "self";
    pub const SINGLETON_METHOD: &'static str = "singleton_method";
    pub const STRING: &'static str = "string";
    pub const UNTIL: &'static str = "until";
    pub const WHEN: &'static str = "when";
    pub const WHILE: &'static str = "while";

    pub const LOGICAL_AND: &'static str = "&&";
    pub const LOGICAL_OR: &'static str = "||";
    pub const AND: &'static str = "and";
    pub const OR: &'static str = "or";
}

impl Default for Ruby {
    fn default() -> Self {
        let language = tree_sitter_ruby::language();

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

impl Language for Ruby {
    fn name(&self) -> &str {
        "ruby"
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

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::PROGRAM]
    }

    fn if_nodes(&self) -> Vec<&str> {
        vec![
            Self::IF,
            Self::UNLESS,
            Self::IF_MODIFIER,
            Self::UNLESS_MODIFIER,
        ]
    }

    fn elsif_nodes(&self) -> Vec<&str> {
        vec![Self::ELSIF]
    }

    fn else_nodes(&self) -> Vec<&str> {
        vec![Self::ELSE]
    }

    fn ternary_nodes(&self) -> Vec<&str> {
        vec![Self::CONDITIONAL]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::CASE]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec![Self::WHEN]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::WHILE, Self::UNTIL, Self::FOR]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![Self::RESCUE]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![Self::BEGIN]
    }

    fn conditional_assignment_nodes(&self) -> Vec<&str> {
        vec![Self::OPERATOR_ASSIGNMENT]
    }

    fn jump_nodes(&self) -> Vec<&str> {
        vec![Self::BREAK, Self::NEXT]
    }

    fn return_nodes(&self) -> Vec<&str> {
        vec![Self::RETURN]
    }

    fn binary_nodes(&self) -> Vec<&str> {
        vec![Self::BINARY]
    }

    fn boolean_operator_nodes(&self) -> Vec<&str> {
        vec![Self::LOGICAL_AND, Self::LOGICAL_OR, Self::AND, Self::OR]
    }

    fn field_nodes(&self) -> Vec<&str> {
        vec![Self::INSTANCE_VARIABLE, Self::CLASS_VARIABLE]
    }

    fn call_nodes(&self) -> Vec<&str> {
        vec![Self::CALL, Self::METHOD_CALL]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::METHOD, Self::SINGLETON_METHOD]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![Self::BLOCK, Self::DO_BLOCK]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING]
    }

    fn constructor_names(&self) -> Vec<&str> {
        vec![Self::INITIALIZE]
    }

    fn iterator_method_identifiers(&self) -> Vec<&str> {
        vec![
            "each",
            "map",
            "collect",
            "select",
            "find_all",
            "reject",
            "find",
            "detect",
            "any?",
            "all?",
            "none?",
            "one?",
            "partition",
            "group_by",
            "each_with_index",
            "reverse_each",
            "each_entry",
            "each_slice",
            "each_cons",
            "flat_map",
            "collect_concat",
            "zip",
            "cycle",
            "lazy",
        ]
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        let function_kind = node.kind();

        match function_kind {
            Self::IDENTIFIER => (Some(Self::SELF.to_string()), node_source(node, source_file)),
            Self::METHOD => {
                let (receiver, object) = self.field_identifiers(source_file, node);

                (Some(receiver), object)
            }
            Self::CALL => {
                let receiver = if node.child_by_field_name("receiver").is_some() {
                    child_source(node, "receiver", source_file)
                } else {
                    Self::SELF.to_string()
                };

                let method = if node.child_by_field_name("method").is_some() {
                    child_source(node, "method", source_file)
                } else if node.child_by_field_name("arguments").is_some() {
                    "call".to_string()
                } else {
                    "<UNKNOWN>".to_string()
                };

                (Some(receiver), method)
            }
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        match node.kind() {
            Self::CLASS_VARIABLE | Self::CONSTANT | Self::GLOBAL_VARIABLE => {
                (Self::SELF.to_string(), node_source(node, source_file))
            }
            Self::INSTANCE_VARIABLE => {
                let source = node_source(node, source_file);
                let modified_source = source.strip_prefix('@').unwrap_or(&source);
                (Self::SELF.to_string(), modified_source.to_string())
            }
            Self::METHOD => {
                let method_node = node.child_by_field_name("method").unwrap();
                (
                    Self::SELF.to_string(),
                    node_source(&method_node, source_file),
                )
            }
            Self::CALL => (
                child_source(node, "receiver", source_file),
                child_source(node, "method", source_file),
            ),
            _ => (node.kind().to_string(), "<UNKNOWN>".to_string()),
        }
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_ruby::language()
    }
}

#[cfg(test)]

mod test {
    use super::*;
    use std::collections::HashSet;
    use tree_sitter::Tree;

    #[test]
    fn mutually_exclusive() {
        let lang = Ruby::default();
        let mut kinds: Vec<&str> = vec![];

        kinds.extend(lang.invisible_container_nodes());
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
        let source_file = File::from_string("ruby", "self.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let language = Ruby::default();

        assert_eq!(
            language.field_identifiers(&source_file, &expression),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_write() {
        let source_file = File::from_string("ruby", "self.foo = 1");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let assignment = root_node.named_child(0).unwrap();
        let field = assignment.child(0).unwrap();
        let language = Ruby::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_collaborator() {
        let source_file = File::from_string("ruby", "other.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let language = Ruby::default();

        assert_eq!(
            language.field_identifiers(&source_file, &expression),
            ("other".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_class_variables() {
        let source_file = File::from_string("ruby", "@@foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let language = Ruby::default();

        assert_eq!(
            language.field_identifiers(&source_file, &expression),
            ("self".to_string(), "@@foo".to_string())
        );
    }

    #[test]
    fn field_identifier_instance_variables() {
        let source_file = File::from_string("ruby", "@foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let language = Ruby::default();

        assert_eq!(
            language.field_identifiers(&source_file, &expression),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_constant() {
        let source_file = File::from_string("ruby", "MY_CONSTANT = 42");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.child(0).unwrap();
        let language = Ruby::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("self".to_string(), "MY_CONSTANT".to_string())
        );
    }

    #[test]
    fn field_identifier_global_variable() {
        let source_file = File::from_string("ruby", "$global_var = 100");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.child(0).unwrap();
        let language = Ruby::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("self".to_string(), "$global_var".to_string())
        );
    }

    #[test]
    fn call_identifier() {
        let source_file = File::from_string("ruby", "foo()");
        let tree = source_file.parse();
        let call = tree.root_node().child(0).unwrap();
        let language = Ruby::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("self".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier_with_module() {
        let source_file = File::from_string("ruby", "Module::Class.method()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Ruby::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("Module::Class".into()), "method".into())
        );
    }

    #[test]
    fn call_identifier_with_syntax_sugar() {
        let source_file = File::from_string("ruby", "recognize_path.('some_path')");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Ruby::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("recognize_path".into()), "call".into())
        );
    }

    #[test]
    fn call_identifier_with_send() {
        let source_file = File::from_string("ruby", "object.send(:method_name)");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Ruby::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("object".into()), "send".into())
        );
    }

    #[test]
    fn call_identifier_with_safe_navigation_operator() {
        let source_file = File::from_string("ruby", "object&.method()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Ruby::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("object".into()), "method".into())
        );
    }

    #[test]
    fn call_with_splat() {
        let source_file = File::from_string("ruby", "foo(*args)");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Ruby::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("self".into()), "foo".into())
        );
    }

    #[test]
    fn tap_method() {
        let source_file = File::from_string("ruby", "some_object.tap { |x| puts x }");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Ruby::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("some_object".into()), "tap".into())
        );
    }

    #[test]
    fn call_member() {
        let source_file = File::from_string("ruby", "foo.bar()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Ruby::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".into()), "bar".into())
        );
    }

    fn call_node(tree: &Tree) -> Node {
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        return expression;
    }
}
