use crate::code::File;
use crate::lang::{
    typescript_common::{TypeScriptCommon, COMMON_FUNCTION_DECLARATION_QUERY},
    Language,
};
use tree_sitter::Node;

pub const FUNCTION_DECLARATION_QUERY_EXTENSION: &str = r#"
(jsx_attribute
  (property_identifier) @name
  (jsx_expression
    (arrow_function
      (formal_parameters) @parameters)))
"#;

pub struct TSX {
    pub common: TypeScriptCommon,
    class_query: tree_sitter::Query,
    function_declaration_query: tree_sitter::Query,
    field_query: tree_sitter::Query,
}

impl Default for TSX {
    fn default() -> Self {
        let language = tree_sitter_typescript::language_tsx();
        let common = TypeScriptCommon::new(&language);
        let class_query = common.class_query();
        let field_query = common.field_query();

        let function_declaration_query = tree_sitter::Query::new(
            &common.language(),
            &format!(
                "[{}{}] @definition.function",
                COMMON_FUNCTION_DECLARATION_QUERY, FUNCTION_DECLARATION_QUERY_EXTENSION
            ),
        )
        .unwrap();

        Self {
            common,
            field_query,
            class_query,
            function_declaration_query,
        }
    }
}

impl Language for TSX {
    fn name(&self) -> &str {
        "tsx"
    }

    fn self_keyword(&self) -> Option<&str> {
        self.common.self_keyword()
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
        self.common.constructor_names()
    }

    fn if_nodes(&self) -> Vec<&str> {
        self.common.if_nodes()
    }

    fn else_nodes(&self) -> Vec<&str> {
        self.common.else_nodes()
    }

    fn conditional_assignment_nodes(&self) -> Vec<&str> {
        self.common.conditional_assignment_nodes()
    }

    fn invisible_container_nodes(&self) -> Vec<&str> {
        self.common.invisible_container_nodes()
    }

    fn switch_nodes(&self) -> Vec<&str> {
        self.common.switch_nodes()
    }

    fn case_nodes(&self) -> Vec<&str> {
        self.common.case_nodes()
    }

    fn ternary_nodes(&self) -> Vec<&str> {
        self.common.ternary_nodes()
    }

    fn loop_nodes(&self) -> Vec<&str> {
        self.common.loop_nodes()
    }

    fn except_nodes(&self) -> Vec<&str> {
        self.common.except_nodes()
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        self.common.try_expression_nodes()
    }

    fn jump_nodes(&self) -> Vec<&str> {
        self.common.jump_nodes()
    }

    fn return_nodes(&self) -> Vec<&str> {
        self.common.return_nodes()
    }

    fn binary_nodes(&self) -> Vec<&str> {
        self.common.binary_nodes()
    }

    fn boolean_operator_nodes(&self) -> Vec<&str> {
        self.common.boolean_operator_nodes()
    }

    fn field_nodes(&self) -> Vec<&str> {
        self.common.field_nodes()
    }

    fn call_nodes(&self) -> Vec<&str> {
        self.common.call_nodes()
    }

    fn function_nodes(&self) -> Vec<&str> {
        self.common.function_nodes()
    }

    fn closure_nodes(&self) -> Vec<&str> {
        self.common.closure_nodes()
    }

    fn comment_nodes(&self) -> Vec<&str> {
        self.common.comment_nodes()
    }

    fn string_nodes(&self) -> Vec<&str> {
        self.common.string_nodes()
    }

    fn iterator_method_identifiers(&self) -> Vec<&str> {
        self.common.iterator_method_identifiers()
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        self.common.call_identifiers(source_file, node)
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        self.common.field_identifiers(source_file, node)
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_typescript::language_tsx()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use tree_sitter::Tree;

    #[test]
    fn mutually_exclusive() {
        let lang = TSX::default();
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
        let source_file = File::from_string("tsx", "self.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = TSX::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_write() {
        let source_file = File::from_string("tsx", "self.foo = 1");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let assignment = expression.named_child(0).unwrap();
        let field = assignment.named_child(0).unwrap();
        let language = TSX::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("self".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn field_identifier_collaborator() {
        let source_file = File::from_string("tsx", "other.foo");
        let tree = source_file.parse();
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        let field = expression.named_child(0).unwrap();
        let language = TSX::default();

        assert_eq!(
            language.field_identifiers(&source_file, &field),
            ("other".to_string(), "foo".to_string())
        );
    }

    #[test]
    fn call_identifier() {
        let source_file = File::from_string("tsx", "foo()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = TSX::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("this".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_member() {
        let source_file = File::from_string("tsx", "foo.bar()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = TSX::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".into()), "bar".into())
        );
    }

    #[test]
    fn call_with_custom_context() {
        let source_file = File::from_string("tsx", "foo.call(context);");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = TSX::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".to_string()), "call".to_string())
        );
    }

    #[test]
    fn call_with_optional_chaining() {
        let source_file = File::from_string("tsx", "obj?.foo();");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = TSX::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("obj".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn method_call_on_nested_object() {
        let source_file = File::from_string("tsx", "obj.nestedObj.foo();");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = TSX::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("nestedObj".to_string()), "foo".to_string())
        );
    }

    #[test]
    fn call_returned_function() {
        let source_file = File::from_string("tsx", "getFunction()()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = TSX::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("getFunction()".to_string()), "<UNKNOWN>".to_string())
        );
    }

    #[test]
    fn call_function_property() {
        let source_file = File::from_string("tsx", "foo.bar()");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = TSX::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("foo".to_string()), "bar".to_string())
        );
    }

    #[test]
    fn call_anonymous_function() {
        let source_file = File::from_string("tsx", "(function() { return 'Hello'; })();");
        let tree = source_file.parse();
        let call = call_node(&tree);
        let language = TSX::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string())
        );
    }

    #[test]
    fn call_arrow_function() {
        let source_file = File::from_string("tsx", "const greeting = () => 'Hello'; greeting();");
        let tree = source_file.parse();
        let call = call_deeper_node(&tree);
        let language = TSX::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("this".to_string()), "greeting".to_string())
        );
    }

    #[test]
    fn call_function_returned_by_getter() {
        let source_file = File::from_string(
            "typescript",
            "let obj = { get myFn() { return function() {}; } }; obj.myFn();",
        );
        let tree = source_file.parse();
        let call = call_deeper_node(&tree);
        let language = TSX::default();

        assert_eq!(
            language.call_identifiers(&source_file, &call),
            (Some("obj".to_string()), "myFn".to_string())
        );
    }

    #[test]
    fn tsx_function_with_argument_test() {
        let source_file = File::from_string(
            "tsx",
            r#"
            function MyComponent(name: string) {
                return (
                    <div className="flex flex-col">
                        <div>Hello, {name} from TSX!</div>
                    </div>
                );
            }
        "#,
        );

        let tree = source_file.parse();
        let root_node = tree.root_node();
        assert!(!root_node.has_error());
    }

    fn call_node(tree: &Tree) -> Node {
        let root_node = tree.root_node();
        let expression = root_node.named_child(0).unwrap();
        expression.named_child(0).unwrap()
    }

    fn call_deeper_node(tree: &Tree) -> Node {
        let root_node = tree.root_node();
        let expression_statement = root_node.named_child(1).unwrap();
        expression_statement.named_child(0).unwrap()
    }
}
