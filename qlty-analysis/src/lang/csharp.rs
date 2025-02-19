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
    (local_function_statement
        name: (identifier) @name
        parameters: (_) @parameters)
] @definition.function
"#;

const FIELD_QUERY: &str = r#"
    [(field_declaration
        (variable_declaration 
        (variable_declarator name: (identifier) @name)
        )
    ) @field
    (property_declaration name: (identifier) @name) @field]
"#;

pub struct CSharp {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl CSharp {
    pub const BINARY: &'static str = "binary_expression";
    pub const BLOCK: &'static str = "block";
    pub const BREAK: &'static str = "break_statement";
    pub const CATCH: &'static str = "catch_clause";
    pub const CASE: &'static str = "switch_section";
    pub const COMMENT: &'static str = "comment";
    pub const COMPILATION_UNIT: &'static str = "complication_unit";
    pub const CONTINUE: &'static str = "continue_statement";
    pub const DO: &'static str = "do_statement";
    pub const FIELD_DECLARATION: &'static str = "field_declaration";
    pub const FIELD_ACCESS: &'static str = "member_access_expression";
    pub const FOR: &'static str = "for_statement";
    pub const FOREACH: &'static str = "foreach_statement";
    pub const GOTO: &'static str = "goto_statement";
    pub const INTERPOLATED_STRING: &'static str = "interpolated_string_expression";
    pub const METHOD_DECLARATION: &'static str = "method_declaration";
    pub const METHOD_INVOCATION: &'static str = "invocation_expression";
    pub const PROPERTY_DECLARATION: &'static str = "property_declaration";
    pub const IDENTIFIER: &'static str = "identifier";
    pub const IF: &'static str = "if_statement";
    pub const LAMBDA: &'static str = "lambda_expression";
    pub const RETURN: &'static str = "return_statement";
    pub const SELF: &'static str = "this";
    pub const STRING: &'static str = "string_literal";
    pub const SWITCH: &'static str = "switch_expression";
    pub const TERNARY: &'static str = "conditional_expression";
    pub const TRY: &'static str = "try_statement";
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
        "csharp"
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
        vec![Self::COMPILATION_UNIT]
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
        vec![Self::FOR, Self::FOREACH, Self::WHILE, Self::DO]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![Self::CATCH]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![Self::TRY]
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
        vec![Self::FIELD_DECLARATION, Self::PROPERTY_DECLARATION]
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
        vec![Self::COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING, Self::INTERPOLATED_STRING]
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
                let function_node = node.child_by_field_name("function");
                match function_node {
                    Some(f) => {
                        if f.kind() == Self::FIELD_ACCESS {
                            let (obj, property) = self.field_identifiers(source_file, &f);
                            (Some(obj), property)
                        } else {
                            (
                                Some(Self::SELF.to_owned()),
                                get_node_source_or_default(Some(f), source_file),
                            )
                        }
                    }
                    None => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
                }
            }
            _ => (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string()),
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        let object_node = node.child_by_field_name("expression");
        let property_node = node.child_by_field_name("name");

        match (&object_node, &property_node) {
            (Some(obj), Some(prop)) if obj.kind() == Self::FIELD_ACCESS => {
                let object_source =
                    get_node_source_or_default(obj.child_by_field_name("name"), source_file);
                let property_source = get_node_source_or_default(Some(*prop), source_file);
                (object_source, property_source)
            }
            (Some(o), Some(p)) => (
                get_node_source_or_default(Some(*o), source_file),
                get_node_source_or_default(Some(*p), source_file),
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

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use tree_sitter::Tree;

    #[test]
    fn mutually_exclusive() {
        let lang = CSharp::default();
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
        let source_file = File::from_string("csharp", "this.foo");
        let tree = source_file.parse();
        let root_node = root_node(&tree);
        let language = CSharp::default();

        assert_eq!(
            language.field_identifiers(&source_file, &root_node),
            ("this".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_write() {
        let source_file = File::from_string("csharp", "this.foo = 1");
        let tree = source_file.parse();
        let root_node = root_node(&tree);
        let assignment = root_node.named_child(0).unwrap();
        let field = assignment.named_child(0).unwrap();
        let language = CSharp::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("this".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_collaborator() {
        let source_file = File::from_string("csharp", "other.foo");
        let tree = source_file.parse();
        let root_node = root_node(&tree);
        let language = CSharp::default();

        assert_eq!(
            language.field_identifiers(&source_file, &root_node),
            ("other".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier() {
        let source_file = File::from_string("csharp", "foo()");
        let tree = source_file.parse();
        let call = root_node(&tree);
        let language = CSharp::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("this".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_member() {
        let source_file = File::from_string("csharp", "foo.bar()");
        let tree = source_file.parse();
        let call = root_node(&tree);
        let language = CSharp::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".into()), "bar".into())
        );
    }

    #[test]
    fn call_with_custom_context() {
        let source_file = File::from_string("csharp", "foo.bar(context)");
        let tree = source_file.parse();
        let root = root_node(&tree);
        let call = root.child(0).unwrap();
        let language = CSharp::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".into()), "bar".into())
        );
    }

    #[test]
    fn method_call_on_nested_object() {
        let source_file = File::from_string("csharp", "obj.nestedObj.foo()");
        let tree = source_file.parse();
        let root = root_node(&tree);
        let call = root.child(0).unwrap();
        let language = CSharp::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("nestedObj".into()), "foo".into())
        );
    }

    #[test]
    fn nested_field_access() {
        let source_file = File::from_string("csharp", "obj.nestedObj.oneMoreObj");
        let tree = source_file.parse();
        let root = root_node(&tree);
        let language = CSharp::default();

        assert_eq!(
            language.field_identifiers(&source_file, &root),
            ("nestedObj".to_string(), "oneMoreObj".to_string())
        );
    }

    // navigates down from "(compilation_unit (global statement ...))"
    fn root_node(tree: &Tree) -> Node {
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        expression.named_child(0).unwrap()
    }
}
