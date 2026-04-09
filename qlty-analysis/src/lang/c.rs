use crate::code::node_source;
use crate::code::File;
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
(struct_specifier
    name: (type_identifier) @name
    body: (field_declaration_list)) @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
(function_definition
    declarator: (function_declarator
        declarator: (identifier) @name
        parameters: (parameter_list) @parameters)) @definition.function
"#;

const FIELD_QUERY: &str = r#"
(struct_specifier
    body: (field_declaration_list
        (field_declaration) @name)) @field
"#;

pub struct C {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl C {
    pub const IF: &'static str = "if_statement";
    pub const FOR: &'static str = "for_statement";
    pub const WHILE: &'static str = "while_statement";
    pub const DO: &'static str = "do_statement";
    pub const SWITCH: &'static str = "switch_statement";
    pub const CASE: &'static str = "case_statement";
    pub const BREAK: &'static str = "break_statement";
    pub const CONTINUE: &'static str = "continue_statement";
    pub const RETURN: &'static str = "return_statement";
    pub const GOTO: &'static str = "goto_statement";
    pub const BINARY: &'static str = "binary_expression";
    pub const CONDITIONAL: &'static str = "conditional_expression";
    pub const CALL: &'static str = "call_expression";
    pub const FUNCTION_DEFINITION: &'static str = "function_definition";
    pub const FIELD_DECLARATION: &'static str = "field_declaration";
    pub const COMPOUND_STATEMENT: &'static str = "compound_statement";
    pub const TRANSLATION_UNIT: &'static str = "translation_unit";
    pub const STRING_LITERAL: &'static str = "string_literal";
    pub const COMMENT: &'static str = "comment";
    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";
    pub const FIELD_EXPRESSION: &'static str = "field_expression";
    pub const IDENTIFIER: &'static str = "identifier";
}

impl Default for C {
    fn default() -> Self {
        let language = tree_sitter_c::language();

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

impl Language for C {
    fn name(&self) -> &str {
        "c"
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

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::SWITCH]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec![Self::CASE]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::FOR, Self::WHILE, Self::DO]
    }

    fn jump_nodes(&self) -> Vec<&str> {
        vec![Self::BREAK, Self::CONTINUE, Self::GOTO]
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
        vec![Self::CALL]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::FUNCTION_DEFINITION]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING_LITERAL]
    }

    fn block_nodes(&self) -> Vec<&str> {
        vec![Self::COMPOUND_STATEMENT]
    }

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::TRANSLATION_UNIT]
    }

    fn ternary_nodes(&self) -> Vec<&str> {
        vec![Self::CONDITIONAL]
    }

    fn has_labeled_jumps(&self) -> bool {
        true
    }

    fn function_name_node<'a>(&'a self, node: &'a Node) -> Node<'a> {
        let declarator = node.child_by_field_name("declarator").unwrap();
        let mut current = declarator;
        loop {
            if let Some(inner) = current.child_by_field_name("declarator") {
                current = inner;
            } else {
                return current;
            }
        }
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        let function_node = node.child_by_field_name("function");
        match function_node.as_ref().map(|n| n.kind()) {
            Some(Self::IDENTIFIER) => {
                let name = function_node
                    .as_ref()
                    .map(|n| node_source(n, source_file))
                    .unwrap_or_else(|| "<UNKNOWN>".to_string());
                (None, name)
            }
            Some(Self::FIELD_EXPRESSION) => {
                let fn_node = function_node.unwrap();
                let (receiver, field) = self.field_identifiers(source_file, &fn_node);
                (Some(receiver), field)
            }
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        let object_node = node.child_by_field_name("argument");
        let property_node = node.child_by_field_name("field");

        match (&object_node, &property_node) {
            (Some(obj), Some(prop)) if obj.kind() == Self::FIELD_EXPRESSION => {
                let inner_field = obj.child_by_field_name("field");
                let object_source = inner_field
                    .as_ref()
                    .map(|n| node_source(n, source_file))
                    .unwrap_or_else(|| "<UNKNOWN>".to_string());
                let property_source = node_source(prop, source_file);
                (object_source, property_source)
            }
            (Some(obj), Some(prop)) => (
                node_source(obj, source_file),
                node_source(prop, source_file),
            ),
            _ => ("<UNKNOWN>".to_string(), "<UNKNOWN>".to_string()),
        }
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_c::language()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use tree_sitter::Tree;

    #[test]
    fn mutually_exclusive() {
        let lang = C::default();
        let mut kinds: Vec<&str> = vec![];

        kinds.extend(lang.if_nodes());
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
    fn call_identifier() {
        let source_file = File::from_string("c", "void main() { foo(); }");
        let tree = source_file.parse();
        let call = find_call_node(&tree);
        let language = C::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (None, "foo".to_string())
        );
    }

    #[test]
    fn call_member() {
        let source_file = File::from_string("c", "void main() { obj.func(); }");
        let tree = source_file.parse();
        let call = find_call_node(&tree);
        let language = C::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("obj".to_string()), "func".to_string())
        );
    }

    #[test]
    fn field_identifier_read() {
        let source_file = File::from_string("c", "void main() { obj.foo; }");
        let tree = source_file.parse();
        let field = find_first_node_of_kind(&tree.root_node(), "field_expression").unwrap();
        let language = C::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("obj".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn nested_field_access() {
        let source_file = File::from_string("c", "void main() { obj.nested.field; }");
        let tree = source_file.parse();
        let root = tree.root_node();
        let outer_field = find_first_node_of_kind(&root, "field_expression").unwrap();
        let language = C::default();

        assert_eq!(
            language.field_identifiers(&source_file, &outer_field),
            ("nested".to_string(), "field".to_string())
        );
    }

    fn find_call_node<'a>(tree: &'a Tree) -> Node<'a> {
        find_first_node_of_kind(&tree.root_node(), "call_expression").unwrap()
    }

    fn find_first_node_of_kind<'a>(node: &Node<'a>, kind: &str) -> Option<Node<'a>> {
        if node.kind() == kind {
            return Some(*node);
        }
        let cursor = &mut node.walk();
        for child in node.children(cursor) {
            if let Some(found) = find_first_node_of_kind(&child, kind) {
                return Some(found);
            }
        }
        None
    }
}
