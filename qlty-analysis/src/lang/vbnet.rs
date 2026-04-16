use crate::code::node_source;
use crate::code::File;
use crate::lang::Language;
use std::sync::Arc;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
[
  (type_declaration
    (class_block
      name: (identifier) @name))
  (type_declaration
    (interface_block
      name: (identifier) @name))
] @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
[
    (method_declaration
        name: (identifier) @name
        parameters: (parameter_list) @parameters)
    (constructor_declaration
        parameters: (parameter_list) @parameters)
] @definition.function
"#;

const FIELD_QUERY: &str = r#"
(member_access
    object: (_) @name
    member: (identifier) @field_name) @field
"#;

pub struct VBNet {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl VBNet {
    pub const IF: &'static str = "if_statement";
    pub const ELSEIF: &'static str = "elseif_clause";
    pub const ELSE: &'static str = "else_clause";
    pub const SELECT: &'static str = "select_case_statement";
    pub const CASE: &'static str = "case_clause";
    pub const FOR: &'static str = "for_statement";
    pub const FOR_EACH: &'static str = "for_each_statement";
    pub const WHILE: &'static str = "while_statement";
    pub const DO: &'static str = "do_statement";
    pub const TRY: &'static str = "try_statement";
    pub const CATCH: &'static str = "catch_block";
    pub const RETURN: &'static str = "return_statement";
    pub const EXIT: &'static str = "exit_statement";
    pub const CONTINUE: &'static str = "continue_statement";
    pub const GOTO: &'static str = "goto_statement";
    pub const BINARY: &'static str = "binary_expression";
    pub const INVOCATION: &'static str = "invocation";
    pub const MEMBER_ACCESS: &'static str = "member_access";
    pub const LAMBDA: &'static str = "lambda_expression";
    pub const COMMENT: &'static str = "comment";
    pub const STRING: &'static str = "string_literal";
    pub const INTERPOLATED_STRING: &'static str = "interpolated_string_expression";
    pub const METHOD_DECLARATION: &'static str = "method_declaration";
    pub const CONSTRUCTOR_DECLARATION: &'static str = "constructor_declaration";
    pub const IDENTIFIER: &'static str = "identifier";

    pub const AND_ALSO: &'static str = "AndAlso";
    pub const OR_ELSE: &'static str = "OrElse";
    pub const AND: &'static str = "And";
    pub const OR: &'static str = "Or";
}

impl Default for VBNet {
    fn default() -> Self {
        let language = tree_sitter_vb_dotnet::language();

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

impl Language for VBNet {
    fn name(&self) -> &str {
        "vbnet"
    }

    fn self_keyword(&self) -> Option<&str> {
        Some("Me")
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
        vec![
            "source_file",
            "type_declaration",
            "class_block",
            "interface_block",
            "module_block",
            "structure_block",
            "statement",
            "call_statement",
            "dim_statement",
            "assignment_statement",
            "expression",
            "blank_line",
            "modifiers",
            "modifier",
            "imports_statement",
            "namespace_block",
            "case_block",
            "catch_block",
        ]
    }

    fn if_nodes(&self) -> Vec<&str> {
        vec![Self::IF]
    }

    fn elsif_nodes(&self) -> Vec<&str> {
        vec![Self::ELSEIF]
    }

    fn else_nodes(&self) -> Vec<&str> {
        vec![Self::ELSE]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::SELECT]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec![Self::CASE]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::FOR, Self::FOR_EACH, Self::WHILE, Self::DO]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![Self::CATCH]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![Self::TRY]
    }

    fn jump_nodes(&self) -> Vec<&str> {
        vec![Self::EXIT, Self::CONTINUE, Self::GOTO]
    }

    fn return_nodes(&self) -> Vec<&str> {
        vec![Self::RETURN]
    }

    fn binary_nodes(&self) -> Vec<&str> {
        vec![Self::BINARY]
    }

    fn boolean_operator_nodes(&self) -> Vec<&str> {
        vec![Self::AND_ALSO, Self::OR_ELSE, Self::AND, Self::OR]
    }

    fn field_nodes(&self) -> Vec<&str> {
        vec![Self::MEMBER_ACCESS]
    }

    fn call_nodes(&self) -> Vec<&str> {
        vec![Self::INVOCATION]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::METHOD_DECLARATION, Self::CONSTRUCTOR_DECLARATION]
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

    fn constructor_names(&self) -> Vec<&str> {
        vec!["New"]
    }

    fn has_field_names(&self) -> bool {
        true
    }

    fn function_name_node<'a>(&'a self, node: &'a Node) -> Node<'a> {
        if node.kind() == Self::CONSTRUCTOR_DECLARATION {
            node.child_by_field_name("parameters").unwrap()
        } else {
            node.child_by_field_name("name").unwrap()
        }
    }

    fn function_name_from_node(&self, _source_file: &File, node: &Node) -> String {
        if node.kind() == Self::CONSTRUCTOR_DECLARATION {
            "New".to_string()
        } else {
            let name_node = node.child_by_field_name("name").unwrap();
            name_node
                .utf8_text(_source_file.contents.as_bytes())
                .unwrap()
                .to_string()
        }
    }

    fn get_parameter_names(
        &self,
        parameters_node: tree_sitter::Node,
        source_file: &Arc<File>,
    ) -> Vec<String> {
        let mut parameter_names = vec![];
        let cursor = &mut parameters_node.walk();

        for parameter_node in parameters_node.named_children(cursor) {
            if parameter_node.kind() != "parameter" {
                continue;
            }
            if let Some(name_node) = parameter_node.child_by_field_name("name") {
                let parameter_name = node_source(&name_node, source_file);
                let sanitized = self.sanitize_parameter_name(parameter_name);
                if let Some(sanitized) = sanitized {
                    parameter_names.push(sanitized);
                }
            }
        }
        parameter_names
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        match node.kind() {
            Self::INVOCATION => {
                let target_node = node.child_by_field_name("target");
                match target_node {
                    Some(t) => {
                        if t.kind() == Self::MEMBER_ACCESS {
                            let (obj, property) = self.field_identifiers(source_file, &t);
                            (Some(obj), property)
                        } else {
                            (
                                Some("Me".to_owned()),
                                get_node_source_or_default(Some(t), source_file),
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
        let object_node = node.child_by_field_name("object");
        let member_node = node.child_by_field_name("member");

        match (&object_node, &member_node) {
            (Some(obj), Some(prop)) if obj.kind() == Self::MEMBER_ACCESS => {
                let object_source =
                    get_node_source_or_default(obj.child_by_field_name("member"), source_file);
                let property_source = get_node_source_or_default(Some(*prop), source_file);
                (object_source, property_source)
            }
            (Some(obj), Some(prop)) => {
                // The object node might be wrapped in an "expression" node
                let obj_text = get_inner_identifier_source(obj, source_file);
                let prop_text = get_node_source_or_default(Some(*prop), source_file);
                (obj_text, prop_text)
            }
            _ => ("<UNKNOWN>".to_string(), "<UNKNOWN>".to_string()),
        }
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_vb_dotnet::language()
    }

    fn normalize_identifier(&self, name: &str) -> String {
        name.to_lowercase()
    }
}

fn get_node_source_or_default(node: Option<Node>, source_file: &File) -> String {
    node.as_ref()
        .map(|n| node_source(n, source_file))
        .unwrap_or("<UNKNOWN>".to_string())
}

fn get_inner_identifier_source(node: &Node, source_file: &File) -> String {
    // In the VB.NET grammar, object fields are often wrapped in "expression" nodes.
    // Drill down to find the actual identifier.
    if node.kind() == "expression" {
        if let Some(child) = node.named_child(0) {
            return get_inner_identifier_source(&child, source_file);
        }
    }
    node_source(node, source_file)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn language_parser() {
        VBNet::default().parser();
    }

    #[test]
    fn mutually_exclusive() {
        let lang = VBNet::default();
        let mut kinds: Vec<&str> = vec![];

        kinds.extend(lang.if_nodes());
        kinds.extend(lang.elsif_nodes());
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
    fn constructor_names_includes_new() {
        let lang = VBNet::default();
        assert!(lang.constructor_names().contains(&"New"));
    }

    #[test]
    fn class_query_captures_class() {
        let source_file = File::from_string(
            "vbnet",
            r#"
Public Class Foo
End Class
"#,
        );
        let tree = source_file.parse();
        let lang = VBNet::default();
        let query = lang.class_query();
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches: Vec<_> = cursor
            .matches(query, tree.root_node(), source_file.contents.as_bytes())
            .collect();
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn function_query_captures_sub() {
        let source_file = File::from_string(
            "vbnet",
            r#"
Public Class Foo
    Public Sub DoWork()
    End Sub
End Class
"#,
        );
        let tree = source_file.parse();
        let lang = VBNet::default();
        let query = lang.function_declaration_query();
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches: Vec<_> = cursor
            .matches(query, tree.root_node(), source_file.contents.as_bytes())
            .collect();
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn function_query_captures_function() {
        let source_file = File::from_string(
            "vbnet",
            r#"
Public Class Foo
    Public Function GetValue() As Integer
        Return 42
    End Function
End Class
"#,
        );
        let tree = source_file.parse();
        let lang = VBNet::default();
        let query = lang.function_declaration_query();
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches: Vec<_> = cursor
            .matches(query, tree.root_node(), source_file.contents.as_bytes())
            .collect();
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn function_query_captures_constructor() {
        let source_file = File::from_string(
            "vbnet",
            r#"
Public Class Foo
    Public Sub New()
    End Sub
End Class
"#,
        );
        let tree = source_file.parse();
        let lang = VBNet::default();
        let query = lang.function_declaration_query();
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches: Vec<_> = cursor
            .matches(query, tree.root_node(), source_file.contents.as_bytes())
            .collect();
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn call_identifier_me_member() {
        let source_file = File::from_string(
            "vbnet",
            r#"
Public Class Foo
    Public Sub DoWork()
        Me.DoSomething()
    End Sub
End Class
"#,
        );
        let tree = source_file.parse();
        let lang = VBNet::default();
        let invocation = find_first_node_of_kind(&tree.root_node(), "invocation").unwrap();

        assert_eq!(
            lang.call_identifiers(&source_file, &invocation),
            (Some("Me".to_string()), "DoSomething".to_string())
        );
    }

    #[test]
    fn field_identifier_me_member() {
        let source_file = File::from_string(
            "vbnet",
            r#"
Public Class Foo
    Public Sub DoWork()
        Dim x = Me.Value
    End Sub
End Class
"#,
        );
        let tree = source_file.parse();
        let lang = VBNet::default();
        let member_access = find_first_node_of_kind(&tree.root_node(), "member_access").unwrap();

        assert_eq!(
            lang.field_identifiers(&source_file, &member_access),
            ("Me".to_string(), "Value".to_string())
        );
    }

    #[test]
    fn get_parameter_names_extracts_correctly() {
        let source_file = File::from_string(
            "vbnet",
            r#"
Public Class Foo
    Public Sub DoWork(x As Integer, y As String)
    End Sub
End Class
"#,
        );
        let tree = source_file.parse();
        let lang = VBNet::default();
        let method = find_first_node_of_kind(&tree.root_node(), "method_declaration").unwrap();
        let params = method.child_by_field_name("parameters").unwrap();
        let arc_file = Arc::new(source_file);
        let names = lang.get_parameter_names(params, &arc_file);

        assert_eq!(names, vec!["x".to_string(), "y".to_string()]);
    }

    #[test]
    fn function_name_node_returns_method_name() {
        let source_file = File::from_string(
            "vbnet",
            r#"
Public Class Foo
    Public Sub DoWork()
    End Sub
End Class
"#,
        );
        let tree = source_file.parse();
        let lang = VBNet::default();
        let method = find_first_node_of_kind(&tree.root_node(), "method_declaration").unwrap();
        let name_node = lang.function_name_node(&method);
        let name = name_node
            .utf8_text(source_file.contents.as_bytes())
            .unwrap();

        assert_eq!(name, "DoWork");
    }

    #[test]
    fn normalize_identifier_lowercases() {
        let lang = VBNet::default();
        assert_eq!(lang.normalize_identifier("DoWork"), "dowork");
        assert_eq!(lang.normalize_identifier("Me"), "me");
        assert_eq!(lang.normalize_identifier("value"), "value");
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
