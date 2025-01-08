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

pub struct Java {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl Java {
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

impl Default for Java {
    fn default() -> Self {
        let language = tree_sitter_java::language();

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

impl Language for Java {
    fn name(&self) -> &str {
        "java"
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
        tree_sitter_java::language()
    }
}

fn get_node_source_or_default(node: Option<Node>, source_file: &File) -> String {
    node.as_ref()
        .map(|n| node_source(n, source_file))
        .unwrap_or("<UNKNOWN>".to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use tree_sitter::Tree;

    #[test]
    fn mutually_exclusive() {
        let lang = Java::default();
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
        let source_file = File::from_string("java", "this.foo;");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Java::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("this".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_write() {
        let source_file = File::from_string("java", "this.foo = 1;");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let assignment = expression.named_child(0).unwrap();
        let field = assignment.named_child(0).unwrap();
        let language = Java::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("this".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_collaborator() {
        let source_file = File::from_string("java", "other.foo;");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Java::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("other".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier() {
        let source_file = File::from_string("java", "foo()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Java::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("this".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_member() {
        let source_file = File::from_string("java", "foo.bar()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Java::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".into()), "bar".into())
        );
    }

    #[test]
    fn call_with_custom_context() {
        let source_file = File::from_string("java", "foo.call(context);");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Java::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".to_string()), "call".to_string())
        );
    }

    #[test]
    fn method_call_on_nested_object() {
        let source_file = File::from_string("java", "obj.nestedObj.foo();");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Java::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("nestedObj".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn nested_field_access() {
        let source_file = File::from_string("java", "obj.nestedObj.oneMoreObj;");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Java::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("nestedObj".to_string(), "oneMoreObj".to_string())
        );
    }

    #[test]
    fn call_function_property() {
        let source_file = File::from_string("java", "foo.bar()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Java::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".to_string()), "bar".to_string())
        );
    }

    fn call_node(tree: &Tree) -> Node {
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        expression.named_child(0).unwrap()
    }
}
