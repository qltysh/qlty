use crate::code::node_source;
use crate::code::File;
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
[
    (class_specifier
        name: (type_identifier) @name
        body: (_))
    (struct_specifier
        name: (type_identifier) @name
        body: (_))
] @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
(function_definition
    declarator: (function_declarator
        declarator: (_) @name
        parameters: (_) @parameters)) @definition.function
"#;

const FIELD_QUERY: &str = r#"
(field_declaration
    declarator: (_) @name) @field
"#;

pub struct Cpp {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

pub const IF: &str = "if_statement";
pub const FOR: &str = "for_statement";
pub const FOR_RANGE_LOOP: &str = "for_range_loop";
pub const WHILE: &str = "while_statement";
pub const DO: &str = "do_statement";
pub const SWITCH: &str = "switch_statement";
pub const CASE: &str = "case_statement";
pub const BREAK: &str = "break_statement";
pub const CONTINUE: &str = "continue_statement";
pub const RETURN: &str = "return_statement";
pub const GOTO: &str = "goto_statement";
pub const BINARY: &str = "binary_expression";
pub const CONDITIONAL: &str = "conditional_expression";
pub const CALL: &str = "call_expression";
pub const FUNCTION_DEFINITION: &str = "function_definition";
pub const FIELD_DECLARATION: &str = "field_declaration";
pub const COMPOUND_STATEMENT: &str = "compound_statement";
pub const TRANSLATION_UNIT: &str = "translation_unit";
pub const STRING_LITERAL: &str = "string_literal";
pub const RAW_STRING: &str = "raw_string_literal";
pub const COMMENT: &str = "comment";
pub const AND: &str = "&&";
pub const OR: &str = "||";
pub const FIELD_EXPRESSION: &str = "field_expression";
pub const CLASS_SPECIFIER: &str = "class_specifier";
pub const STRUCT_SPECIFIER: &str = "struct_specifier";
pub const LAMBDA: &str = "lambda_expression";
pub const TRY: &str = "try_statement";
pub const CATCH: &str = "catch_clause";
pub const THROW: &str = "throw_statement";
pub const NAMESPACE: &str = "namespace_definition";
pub const USING_DECLARATION: &str = "using_declaration";

impl Default for Cpp {
    fn default() -> Self {
        let language = tree_sitter_cpp::language();

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

impl Language for Cpp {
    fn name(&self) -> &str {
        "cpp"
    }

    fn self_keyword(&self) -> Option<&str> {
        Some("this")
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
        vec![IF]
    }

    fn block_nodes(&self) -> Vec<&str> {
        vec![COMPOUND_STATEMENT]
    }

    fn conditional_assignment_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![TRANSLATION_UNIT]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![SWITCH]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec![CASE]
    }

    fn ternary_nodes(&self) -> Vec<&str> {
        vec![CONDITIONAL]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![FOR, WHILE, DO, FOR_RANGE_LOOP]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![CATCH]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![TRY]
    }

    fn jump_nodes(&self) -> Vec<&str> {
        vec![BREAK, CONTINUE, GOTO]
    }

    fn return_nodes(&self) -> Vec<&str> {
        vec![RETURN]
    }

    fn binary_nodes(&self) -> Vec<&str> {
        vec![BINARY]
    }

    fn boolean_operator_nodes(&self) -> Vec<&str> {
        vec![AND, OR]
    }

    fn field_nodes(&self) -> Vec<&str> {
        vec![FIELD_DECLARATION]
    }

    fn call_nodes(&self) -> Vec<&str> {
        vec![CALL]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![FUNCTION_DEFINITION]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![LAMBDA]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![STRING_LITERAL, RAW_STRING]
    }

    fn has_labeled_jumps(&self) -> bool {
        true
    }

    fn function_name_node<'a>(&'a self, node: &'a Node) -> Node<'a> {
        let mut current = node.child_by_field_name("declarator").unwrap();
        while let Some(inner) = current.child_by_field_name("declarator") {
            current = inner;
        }
        current
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        match node.kind() {
            CALL => {
                let function_node = node.child_by_field_name("function").unwrap();

                if function_node.kind() == FIELD_EXPRESSION {
                    let object_node = function_node.child_by_field_name("argument").unwrap();
                    let object_source = node_source(&object_node, source_file);

                    let field_node = function_node.child_by_field_name("field").unwrap();
                    let method_name = node_source(&field_node, source_file);

                    (Some(object_source), method_name)
                } else {
                    let function_name = node_source(&function_node, source_file);
                    (Some("this".to_string()), function_name)
                }
            }
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        if node.kind() == FIELD_EXPRESSION {
            let object_node = node.child_by_field_name("argument").unwrap();
            let object_source = node_source(&object_node, source_file);

            let field_node = node.child_by_field_name("field").unwrap();
            let field_name = node_source(&field_node, source_file);

            (object_source, field_name)
        } else {
            ("<UNKNOWN>".to_string(), "<UNKNOWN>".to_string())
        }
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_cpp::language()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn mutually_exclusive() {
        let lang = Cpp::default();
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
    fn call_identifier_simple() {
        let source_file = File::from_string("cpp", "void f() { foo(); }");
        let tree = source_file.parse();
        let root = tree.root_node();
        let func = root.named_child(0).unwrap();
        let body = func.child_by_field_name("body").unwrap();
        let expr_stmt = body.named_child(0).unwrap();
        let call = expr_stmt.named_child(0).unwrap();
        let language = Cpp::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("this".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier_method() {
        let source_file = File::from_string("cpp", "void f() { obj.bar(); }");
        let tree = source_file.parse();
        let root = tree.root_node();
        let func = root.named_child(0).unwrap();
        let body = func.child_by_field_name("body").unwrap();
        let expr_stmt = body.named_child(0).unwrap();
        let call = expr_stmt.named_child(0).unwrap();
        let language = Cpp::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("obj".to_string()), "bar".to_string())
        );
    }

    #[test]
    fn call_identifier_arrow() {
        let source_file = File::from_string("cpp", "void f() { ptr->method(); }");
        let tree = source_file.parse();
        let root = tree.root_node();
        let func = root.named_child(0).unwrap();
        let body = func.child_by_field_name("body").unwrap();
        let expr_stmt = body.named_child(0).unwrap();
        let call = expr_stmt.named_child(0).unwrap();
        let language = Cpp::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("ptr".to_string()), "method".to_string())
        );
    }

    #[test]
    fn field_identifier_dot() {
        let source_file = File::from_string("cpp", "void f() { obj.field; }");
        let tree = source_file.parse();
        let root = tree.root_node();
        let func = root.named_child(0).unwrap();
        let body = func.child_by_field_name("body").unwrap();
        let expr_stmt = body.named_child(0).unwrap();
        let field_expr = expr_stmt.named_child(0).unwrap();
        let language = Cpp::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field_expr),
            ("obj".to_string(), "field".to_string())
        );
    }

    #[test]
    fn field_identifier_arrow() {
        let source_file = File::from_string("cpp", "void f() { ptr->field; }");
        let tree = source_file.parse();
        let root = tree.root_node();
        let func = root.named_child(0).unwrap();
        let body = func.child_by_field_name("body").unwrap();
        let expr_stmt = body.named_child(0).unwrap();
        let field_expr = expr_stmt.named_child(0).unwrap();
        let language = Cpp::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field_expr),
            ("ptr".to_string(), "field".to_string())
        );
    }
}
