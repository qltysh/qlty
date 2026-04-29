use crate::code::node_source;
use crate::code::File;
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
[
    (class_definition
        name: (identifier) @name)
    (object_definition
        name: (identifier) @name)
    (trait_definition
        name: (identifier) @name)
    (enum_definition
        name: (identifier) @name)
] @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
[
    (function_definition
        name: (identifier) @name
        parameters: (parameters) @parameters)
    (function_definition
        name: (identifier) @name)
    (function_declaration
        name: (identifier) @name
        parameters: (parameters) @parameters)
    (function_declaration
        name: (identifier) @name)
] @definition.function
"#;

const FIELD_QUERY: &str = r#"
[
    (class_parameter
        name: (identifier) @name) @field
    (val_definition
        pattern: (identifier) @name) @field
    (var_definition
        pattern: (identifier) @name) @field
    (val_declaration
        name: (identifier) @name) @field
    (var_declaration
        name: (identifier) @name) @field
]
"#;

pub struct Scala {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl Scala {
    pub const NAME: &'static str = "scala";
    pub const THIS: &'static str = "this";

    pub const COMPILATION_UNIT: &'static str = "compilation_unit";

    pub const IF: &'static str = "if_expression";
    pub const MATCH: &'static str = "match_expression";
    pub const CASE_CLAUSE: &'static str = "case_clause";
    pub const FOR: &'static str = "for_expression";
    pub const WHILE: &'static str = "while_expression";
    pub const DO_WHILE: &'static str = "do_while_expression";
    pub const TRY: &'static str = "try_expression";
    pub const CATCH: &'static str = "catch_clause";
    pub const RETURN: &'static str = "return_expression";
    pub const THROW: &'static str = "throw_expression";

    pub const INFIX: &'static str = "infix_expression";
    pub const FIELD_EXPRESSION: &'static str = "field_expression";
    pub const CALL: &'static str = "call_expression";
    pub const FUNCTION_DEFINITION: &'static str = "function_definition";
    pub const LAMBDA: &'static str = "lambda_expression";

    pub const COMMENT: &'static str = "comment";
    pub const BLOCK_COMMENT: &'static str = "block_comment";
    pub const STRING: &'static str = "string";
    pub const INTERPOLATED_STRING: &'static str = "interpolated_string_expression";
    pub const BLOCK: &'static str = "block";

    pub const IDENTIFIER: &'static str = "identifier";

    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";
}

impl Default for Scala {
    fn default() -> Self {
        let language = tree_sitter_scala::language();

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

impl Language for Scala {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn self_keyword(&self) -> Option<&str> {
        Some(Self::THIS)
    }

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::COMPILATION_UNIT]
    }

    fn if_nodes(&self) -> Vec<&str> {
        vec![Self::IF]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::MATCH]
    }

    fn case_nodes(&self) -> Vec<&str> {
        vec![Self::CASE_CLAUSE]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::FOR, Self::WHILE, Self::DO_WHILE]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![Self::CATCH]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![Self::TRY]
    }

    fn jump_nodes(&self) -> Vec<&str> {
        vec![Self::RETURN, Self::THROW]
    }

    fn return_nodes(&self) -> Vec<&str> {
        vec![Self::RETURN]
    }

    fn binary_nodes(&self) -> Vec<&str> {
        vec![Self::INFIX]
    }

    fn boolean_operator_nodes(&self) -> Vec<&str> {
        vec![Self::AND, Self::OR]
    }

    fn field_nodes(&self) -> Vec<&str> {
        vec![Self::FIELD_EXPRESSION]
    }

    fn call_nodes(&self) -> Vec<&str> {
        vec![Self::CALL]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::FUNCTION_DEFINITION]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![Self::LAMBDA]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::COMMENT, Self::BLOCK_COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING, Self::INTERPOLATED_STRING]
    }

    fn block_nodes(&self) -> Vec<&str> {
        vec![Self::BLOCK]
    }

    fn constructor_names(&self) -> Vec<&str> {
        vec![Self::THIS]
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

    fn iterator_method_identifiers(&self) -> Vec<&str> {
        vec![
            "collect",
            "count",
            "exists",
            "filter",
            "filterNot",
            "find",
            "flatMap",
            "fold",
            "foldLeft",
            "foldRight",
            "forall",
            "foreach",
            "groupBy",
            "map",
            "partition",
            "reduce",
            "reduceLeft",
            "reduceRight",
            "scan",
            "sortBy",
            "sortWith",
            "zip",
        ]
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        if let Some(function_node) = node.child_by_field_name("function") {
            match function_node.kind() {
                Self::FIELD_EXPRESSION => {
                    let (receiver, object) = self.field_identifiers(source_file, &function_node);
                    (Some(receiver), object)
                }
                Self::IDENTIFIER => (
                    Some("".to_string()),
                    node_source(&function_node, source_file),
                ),
                _ => (
                    Some("<UNKNOWN>".to_string()),
                    node_source(&function_node, source_file),
                ),
            }
        } else {
            (Some("<UNKNOWN>".to_string()), "<UNKNOWN>".to_string())
        }
    }

    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String) {
        if node.kind() == Self::FIELD_EXPRESSION {
            let value_node = node.child_by_field_name("value");
            let field_node = node.child_by_field_name("field");

            let receiver = value_node
                .map(|n| node_source(&n, source_file))
                .unwrap_or_else(|| "<UNKNOWN>".to_string());

            let field = field_node
                .map(|n| node_source(&n, source_file))
                .unwrap_or_else(|| "<UNKNOWN>".to_string());

            (receiver, field)
        } else {
            ("<UNKNOWN>".to_string(), "<UNKNOWN>".to_string())
        }
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_scala::language()
    }

    fn has_field_names(&self) -> bool {
        true
    }

    fn function_name_node<'a>(&'a self, node: &'a Node) -> Node<'a> {
        node.child_by_field_name("name")
            .unwrap_or_else(|| node.child(0).unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::code::File;
    use std::collections::HashSet;
    use std::sync::Arc;

    #[test]
    fn registered_in_all_langs() {
        let language = crate::lang::from_str("scala").unwrap();
        assert_eq!(language.name(), "scala");
    }

    #[test]
    fn mutually_exclusive() {
        let lang = Scala::default();
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
    fn parses_class() {
        let source = "class Foo { def bar(x: Int): Int = x + 1 }";
        let file = Arc::new(File::from_string(Scala::NAME, source));
        let tree = file.parse();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn parses_object() {
        let source = "object Foo { def bar: Int = 1 }";
        let file = Arc::new(File::from_string(Scala::NAME, source));
        let tree = file.parse();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn parses_trait() {
        let source = "trait Foo { def bar(): Int }";
        let file = Arc::new(File::from_string(Scala::NAME, source));
        let tree = file.parse();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn parses_case_class() {
        let source = "case class Point(x: Int, y: Int)";
        let file = Arc::new(File::from_string(Scala::NAME, source));
        let tree = file.parse();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn parses_match_with_guard() {
        let source = r#"
class Demo {
  def classify(n: Int): String = n match {
    case x if x < 0 => "neg"
    case 0 => "zero"
    case _ => "pos"
  }
}
"#;
        let file = Arc::new(File::from_string(Scala::NAME, source));
        let tree = file.parse();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn parses_for_comprehension() {
        let source = r#"
object Demo {
  def main(): Seq[Int] = for { x <- Seq(1, 2); y <- Seq(3, 4) } yield x + y
}
"#;
        let file = Arc::new(File::from_string(Scala::NAME, source));
        let tree = file.parse();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn parses_aux_constructor_named_this() {
        let source = r#"
class Foo(val x: Int) {
  def this() = this(0)
}
"#;
        let file = Arc::new(File::from_string(Scala::NAME, source));
        let tree = file.parse();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn malformed_input_does_not_panic() {
        let file = Arc::new(File::from_string(Scala::NAME, "class { ::: !!! "));
        let _tree = file.parse();
    }
}
