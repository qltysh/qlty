use crate::code::node_source;
use crate::code::File;
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
(type_declaration
    (_
      name: (type_identifier) @name)) @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
[
    (method_declaration
        name: (field_identifier) @name
        parameters: (_) @parameters)
    (function_declaration
        name: (identifier) @name
        parameters: (_) @parameters)
] @definition.function
"#;

const FIELD_QUERY: &str = r#"
(struct_type
    (field_declaration_list
        (field_declaration
            name: (field_identifier) @name))) @field
"#;

pub struct Go {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl Go {
    pub const BINARY: &'static str = "binary_expression";
    pub const BLOCK: &'static str = "block";
    pub const BREAK: &'static str = "break_statement";
    pub const CALL_EXPRESSION: &'static str = "call_expression";
    pub const COMMENT: &'static str = "comment";
    pub const COMMUNICATION_CASE: &'static str = "communication_case";
    pub const CONTINUE: &'static str = "continue_statement";
    pub const EXPRESSION_CASE: &'static str = "expression_case";
    pub const EXPRESSION_SWITCH: &'static str = "expression_switch_statement";
    pub const FALLTHROUGH: &'static str = "fallthrough_statement";
    pub const FIELD_DECLARATION: &'static str = "field_declaration";
    pub const FOR: &'static str = "for_statement";
    pub const FUNCTION_DECLARATION: &'static str = "function_declaration";
    pub const GOTO: &'static str = "goto_statement";
    pub const IDENTIFIER: &'static str = "identifier";
    pub const IF: &'static str = "if_statement";
    pub const INTERPRETED_STRING: &'static str = "interpreted_string_literal";
    pub const LABEL_NAME: &'static str = "label_name";
    pub const LAMBDA: &'static str = "func_literal";
    pub const METHOD_DECLARATION: &'static str = "method_declaration";
    pub const RAW_STRING: &'static str = "raw_string_literal";
    pub const RETURN: &'static str = "return_statement";
    pub const SELECT: &'static str = "select_statement";
    pub const SELECTOR_EXPRESSION: &'static str = "selector_expression";
    pub const SELF: &'static str = "this";
    pub const SOURCE_FILE: &'static str = "source_file";
    pub const TYPE_CASE: &'static str = "type_case";
    pub const TYPE_SWITCH: &'static str = "type_switch_statement";

    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";
}

impl Default for Go {
    fn default() -> Self {
        let language = tree_sitter_go::language();

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

impl Language for Go {
    fn name(&self) -> &str {
        "go"
    }

    fn self_keyword(&self) -> Option<&str> {
        None
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

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::SOURCE_FILE]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::EXPRESSION_SWITCH, Self::TYPE_SWITCH, Self::SELECT]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec![
            Self::EXPRESSION_CASE,
            Self::TYPE_CASE,
            Self::COMMUNICATION_CASE,
        ]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::FOR]
    }

    fn jump_nodes(&self) -> Vec<&str> {
        vec![Self::BREAK, Self::CONTINUE, Self::GOTO, Self::FALLTHROUGH]
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
        vec![Self::FUNCTION_DECLARATION, Self::METHOD_DECLARATION]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![Self::LAMBDA]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::RAW_STRING, Self::INTERPRETED_STRING]
    }

    fn is_jump_label(&self, node: &Node) -> bool {
        node.kind() == Self::LABEL_NAME
    }

    fn has_labeled_jumps(&self) -> bool {
        true
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        let function_node = node.child_by_field_name("function");
        match function_node.as_ref().map(|n| n.kind()) {
            Some(Self::IDENTIFIER) => {
                (None, get_node_source_or_default(function_node, source_file))
            }
            Some(Self::SELECTOR_EXPRESSION) => {
                let (receiver, object) =
                    self.field_identifiers(source_file, &function_node.unwrap());

                (Some(receiver), object)
            }
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        let object_node = node.child_by_field_name("operand");
        let property_node = node.child_by_field_name("field");

        match (&object_node, &property_node) {
            (Some(obj), Some(prop)) if obj.kind() == Self::SELECTOR_EXPRESSION => {
                let object_source =
                    get_node_source_or_default(obj.child_by_field_name("field"), source_file);
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

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_go::language()
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
        let lang = Go::default();
        let mut kinds: Vec<&str> = vec![];

        kinds.extend(lang.if_nodes());
        kinds.extend(lang.switch_nodes());
        kinds.extend(lang.case_nodes());
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
        let source_file = File::from_string("go", "object.foo\n");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Go::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("object".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_write() {
        let source_file = File::from_string("go", "object.foo = 1");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let assignment = expression.named_child(0).unwrap();
        let field = assignment.named_child(0).unwrap();
        let language = Go::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("object".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier() {
        let source_file = File::from_string("go", "foo()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Go::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (None, "foo".to_string())
        );
    }

    #[test]
    fn call_member() {
        let source_file = File::from_string("go", "foo.bar()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Go::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".into()), "bar".into())
        );
    }

    #[test]
    fn call_with_custom_context() {
        let source_file = File::from_string("go", "foo.call(context)\n");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Go::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".to_string()), "call".to_string())
        );
    }

    #[test]
    fn method_call_on_nested_object() {
        let source_file = File::from_string("go", "obj.nestedObj.foo()\n");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Go::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("nestedObj".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn nested_field_access() {
        let source_file = File::from_string("go", "obj.nestedObj.oneMoreObj\n");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = Go::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("nestedObj".to_string(), "oneMoreObj".to_string())
        );
    }

    #[test]
    fn call_function_property() {
        let source_file = File::from_string("go", "foo.bar()\n");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = Go::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".to_string()), "bar".to_string())
        );
    }

    fn call_node(tree: &Tree) -> Node<'_> {
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        expression.named_child(0).unwrap()
    }
}
