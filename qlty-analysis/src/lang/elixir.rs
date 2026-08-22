use crate::code::node_source;
use crate::code::File;
use crate::lang::Language;
use tree_sitter::Node;

const CLASS_QUERY: &str = r#"
(call
  target: (identifier) @_definition
  (arguments (alias) @name)
  (#match? @_definition "^(defmodule|defprotocol|defimpl)$")) @definition.class
"#;

const FUNCTION_DECLARATION_QUERY: &str = r#"
[
  (call
    target: (identifier) @_definition
    (arguments
      (call target: (identifier) @name (arguments) @parameters))
    (#match? @_definition "^(def|defp|defmacro|defmacrop|defguard|defguardp)$"))
  (call
    target: (identifier) @_definition
    (arguments (identifier) @name)
    (#match? @_definition "^(def|defp|defmacro|defmacrop|defguard|defguardp)$"))
  (call
    target: (identifier) @_definition
    (arguments
      (binary_operator
        left: (call target: (identifier) @name (arguments) @parameters)))
    (#match? @_definition "^(def|defp|defmacro|defmacrop|defguard|defguardp)$"))
  (call
    target: (identifier) @_definition
    (arguments
      (binary_operator left: (identifier) @name))
    (#match? @_definition "^(def|defp|defmacro|defmacrop|defguard|defguardp)$"))
] @definition.function
"#;

const FIELD_QUERY: &str = r#"
[
  (unary_operator
    operator: "@"
    operand: (call target: (identifier) @name)
    (#not-match? @name "^(after_compile|before_compile|behaviour|behavior|callback|compile|deprecated|derive|describetag|dialyzer|doc|doctest|enforce_keys|external_resource|impl|macrocallback|moduledoc|moduletag|on_definition|on_load|opaque|optional_callbacks|since|spec|tag|type|typedoc|typep)$")) @field
  (call
    target: (identifier) @_defstruct
    (arguments
      (list [(atom) @name (keywords (pair key: (keyword) @name))]))
    (#eq? @_defstruct "defstruct")) @field
  (call
    target: (identifier) @_defstruct
    (arguments (keywords (pair key: (keyword) @name)))
    (#eq? @_defstruct "defstruct")) @field
]
"#;

pub struct Elixir {
    pub class_query: tree_sitter::Query,
    pub function_declaration_query: tree_sitter::Query,
    pub field_query: tree_sitter::Query,
}

impl Elixir {
    pub const NAME: &'static str = "elixir";

    // `def`/`if`/`case` share the `call` kind and `=`/`|>`/`&&` share `binary_operator`, so
    // dispatch resolves them from source. `__elixir_` cannot collide with a grammar kind.
    pub const DEFINITION: &'static str = "__elixir_definition";
    pub const DEFINITION_HEAD: &'static str = "__elixir_definition_head";
    pub const CONDITIONAL: &'static str = "__elixir_conditional";
    pub const BRANCH: &'static str = "__elixir_branch";
    pub const COMPREHENSION: &'static str = "__elixir_comprehension";
    pub const TRY: &'static str = "__elixir_try";
    pub const BOOLEAN_BINARY: &'static str = "__elixir_boolean_binary";
    pub const CASE_CLAUSE: &'static str = "__elixir_case_clause";
    pub const STAB_CLAUSE: &'static str = "__elixir_stab_clause";
    pub const ATTRIBUTE_NAME: &'static str = "__elixir_attribute_name";
    pub const TYPE_DECLARATION: &'static str = "__elixir_type_declaration";
    pub const KEYWORD_ELSE: &'static str = "__elixir_keyword_else";
    pub const SECOND_CLAUSE_HEAD: &'static str = "__elixir_second_clause_head";
    pub const LATER_CLAUSE_HEAD: &'static str = "__elixir_later_clause_head";

    pub const SOURCE: &'static str = "source";
    pub const CALL: &'static str = "call";
    pub const BINARY_OPERATOR: &'static str = "binary_operator";
    pub const UNARY_OPERATOR: &'static str = "unary_operator";
    pub const STAB_CLAUSE_KIND: &'static str = "stab_clause";
    pub const ANONYMOUS_FUNCTION: &'static str = "anonymous_function";
    pub const RESCUE_BLOCK: &'static str = "rescue_block";
    pub const CATCH_BLOCK: &'static str = "catch_block";
    pub const DO_BLOCK: &'static str = "do_block";
    pub const ELSE_BLOCK: &'static str = "else_block";
    pub const AFTER_BLOCK: &'static str = "after_block";
    pub const COMMENT: &'static str = "comment";
    pub const STRING: &'static str = "string";
    pub const CHARLIST: &'static str = "charlist";
    pub const SIGIL: &'static str = "sigil";
    pub const IDENTIFIER: &'static str = "identifier";
    pub const DOT: &'static str = "dot";
    pub const ARGUMENTS: &'static str = "arguments";
    pub const KEYWORDS: &'static str = "keywords";
    pub const PAIR: &'static str = "pair";

    pub const AND: &'static str = "and";
    pub const OR: &'static str = "or";
    pub const AMPERSAND_AMPERSAND: &'static str = "&&";
    pub const PIPE_PIPE: &'static str = "||";

    const UNKNOWN: &'static str = "<UNKNOWN>";

    const ATTRIBUTE_OPERATOR: &'static str = "@";

    const ELSE_KEYWORD: &'static str = "else:";

    /// Macros whose keyword-form `else:` is a branch rather than an ordinary option.
    const ELSE_KEYWORD_CALLERS: [&'static str; 5] = ["if", "unless", "with", "try", "receive"];

    const DEFINITION_KEYWORDS: [&'static str; 6] = [
        "def",
        "defp",
        "defmacro",
        "defmacrop",
        "defguard",
        "defguardp",
    ];

    /// Macros whose first argument is a `name(params)` head that declares rather than calls.
    /// `defdelegate` is here but not in `DEFINITION_KEYWORDS`: only its head is a declaration.
    const DECLARATION_HEAD_KEYWORDS: [&'static str; 7] = [
        "def",
        "defp",
        "defmacro",
        "defmacrop",
        "defguard",
        "defguardp",
        "defdelegate",
    ];

    /// Attributes whose body is a pure type declaration and never executes. Value attributes
    /// are absent on purpose: the calls in their bodies do run.
    const TYPE_DECLARATION_ATTRIBUTES: [&'static str; 6] = [
        "callback",
        "macrocallback",
        "opaque",
        "spec",
        "type",
        "typep",
    ];

    fn dispatch_call(node: &Node, source_file: &File) -> &'static str {
        if let Some(declaration) = Self::declaration_for_head(node, source_file) {
            return match Self::preceding_clause_count(&declaration, source_file) {
                0 => Self::DEFINITION_HEAD,
                1 => Self::SECOND_CLAUSE_HEAD,
                _ => Self::LATER_CLAUSE_HEAD,
            };
        }

        if Self::is_attribute_name(node, source_file) {
            return Self::ATTRIBUTE_NAME;
        }

        if Self::is_in_type_declaration(node, source_file) {
            return Self::TYPE_DECLARATION;
        }

        let Some(target) = node.child_by_field_name("target") else {
            return Self::CALL;
        };

        if target.kind() != Self::IDENTIFIER {
            return Self::CALL; // qualified or dynamic call, e.g. Mod.fun()
        }

        let target_source = node_source(&target, source_file);

        if Self::DEFINITION_KEYWORDS.contains(&target_source.as_str()) {
            return Self::DEFINITION;
        }

        match target_source.as_str() {
            "if" | "unless" => Self::CONDITIONAL,
            "case" | "cond" | "with" | "receive" => Self::BRANCH,
            "for" => Self::COMPREHENSION,
            "try" => Self::TRY,
            _ => Self::CALL,
        }
    }

    /// `@timeout 5000` parses as a `call` named `timeout`; left as one, `@map` or `@reduce`
    /// would charge complexity.
    fn is_attribute_name(node: &Node, source_file: &File) -> bool {
        node.parent().is_some_and(|parent| {
            Self::is_module_attribute(&parent, source_file)
                && parent.child_by_field_name("operand") == Some(*node)
        })
    }

    fn is_module_attribute(node: &Node, source_file: &File) -> bool {
        node.kind() == Self::UNARY_OPERATOR
            && node
                .child_by_field_name("operator")
                .is_some_and(|operator| {
                    node_source(&operator, source_file) == Self::ATTRIBUTE_OPERATOR
                })
    }

    /// A `@spec`/`@callback`/`@type` body never executes, so its calls must not count, or
    /// `@spec map(...)` charges complexity for the name it declares. Walks to the nearest
    /// enclosing attribute — nesting shape is irrelevant; calls in a *value* attribute count.
    fn is_in_type_declaration(node: &Node, source_file: &File) -> bool {
        let mut current = *node;

        while let Some(parent) = current.parent() {
            if Self::is_module_attribute(&parent, source_file) {
                return Self::is_type_declaration_name(&parent, source_file);
            }

            current = parent;
        }

        false
    }

    fn is_type_declaration_name(attribute: &Node, source_file: &File) -> bool {
        attribute
            .child_by_field_name("operand")
            .and_then(|operand| operand.child_by_field_name("target"))
            .is_some_and(|target| {
                target.kind() == Self::IDENTIFIER
                    && Self::TYPE_DECLARATION_ATTRIBUTES
                        .contains(&node_source(&target, source_file).as_str())
            })
    }

    /// The `name(params)` head inside a `def`, behind any `when` guards. Not a real call:
    /// dispatched as one, every function signature reads as self-recursion.
    fn declaration_for_head<'tree>(node: &Node<'tree>, source_file: &File) -> Option<Node<'tree>> {
        let mut current = *node;

        while let Some(parent) = current.parent() {
            match parent.kind() {
                Self::BINARY_OPERATOR if parent.child_by_field_name("left") == Some(current) => {
                    current = parent;
                }
                Self::ARGUMENTS => {
                    let declaration = parent.parent()?;

                    return Self::is_declaration_call(&declaration, source_file)
                        .then_some(declaration);
                }
                _ => return None,
            }
        }

        None
    }

    fn is_declaration_call(node: &Node, source_file: &File) -> bool {
        node.kind() == Self::CALL
            && node.child_by_field_name("target").is_some_and(|target| {
                target.kind() == Self::IDENTIFIER
                    && Self::DECLARATION_HEAD_KEYWORDS
                        .contains(&node_source(&target, source_file).as_str())
            })
    }

    /// `=`, `|>`, `::` and arithmetic are all `binary_operator`; mapping them all into
    /// `binary_nodes` would charge every binding and pipeline stage.
    fn dispatch_binary_operator(node: &Node, source_file: &File) -> &'static str {
        let Some(operator) = node.child_by_field_name("operator") else {
            return Self::BINARY_OPERATOR;
        };

        match node_source(&operator, source_file).as_str() {
            Self::AMPERSAND_AMPERSAND | Self::PIPE_PIPE | Self::AND | Self::OR => {
                Self::BOOLEAN_BINARY
            }
            _ => Self::BINARY_OPERATOR,
        }
    }

    /// `stab_clause` is a case arm, an `else`/`after` arm, a rescue handler and an `fn` clause
    /// alike. Only the first two branch: rescue counts via `except_nodes`, closures never do.
    fn dispatch_stab_clause(node: &Node) -> &'static str {
        match node.parent().map(|parent| parent.kind()) {
            Some(Self::DO_BLOCK) | Some(Self::ELSE_BLOCK) | Some(Self::AFTER_BLOCK) => {
                Self::CASE_CLAUSE
            }
            _ => Self::STAB_CLAUSE,
        }
    }

    /// `if x, do: 1, else: 2` spells `else` as a `pair`, not an `else_block`, so the two forms
    /// of one branch would score differently. Scoped to a conditional's own argument list.
    fn dispatch_pair(node: &Node, source_file: &File) -> &'static str {
        let Some(key) = node.child_by_field_name("key") else {
            return Self::PAIR;
        };

        // The `keyword` token spans its trailing whitespace: `else: ` and `else:\n` alike.
        if node_source(&key, source_file).trim_end() != Self::ELSE_KEYWORD {
            return Self::PAIR;
        }

        if Self::is_conditional_keyword(node, source_file) {
            Self::KEYWORD_ELSE
        } else {
            Self::PAIR
        }
    }

    fn is_conditional_keyword(node: &Node, source_file: &File) -> bool {
        node.parent()
            .filter(|keywords| keywords.kind() == Self::KEYWORDS)
            .and_then(|keywords| keywords.parent())
            .filter(|arguments| arguments.kind() == Self::ARGUMENTS)
            .and_then(|arguments| arguments.parent())
            .and_then(|call| call.child_by_field_name("target"))
            .is_some_and(|target| {
                target.kind() == Self::IDENTIFIER
                    && Self::ELSE_KEYWORD_CALLERS
                        .contains(&node_source(&target, source_file).as_str())
            })
    }

    /// Elixir branches through clause heads, not the body: an eight-clause `handle_call/3`
    /// would otherwise score as a one-liner. Scans all siblings — `@doc` sits between clauses.
    fn preceding_clause_count(node: &Node, source_file: &File) -> usize {
        let Some(signature) = Self::definition_signature(node, source_file) else {
            return 0;
        };

        let mut count = 0;
        let mut sibling = node.prev_named_sibling();

        while let Some(current) = sibling {
            if Self::definition_signature(&current, source_file) == Some(signature) {
                count += 1;
            }

            sibling = current.prev_named_sibling();
        }

        count
    }

    /// Two clauses belong to the same function when name and arity agree.
    fn definition_signature<'src>(
        node: &Node,
        source_file: &'src File,
    ) -> Option<(&'src str, usize)> {
        if node.kind() != Self::CALL {
            return None;
        }

        let target = node.child_by_field_name("target")?;

        if target.kind() != Self::IDENTIFIER
            || !Self::DEFINITION_KEYWORDS.contains(&Self::text(&target, source_file)?)
        {
            return None;
        }

        let head = Self::definition_head(node)?;

        match head.kind() {
            Self::IDENTIFIER => Some((Self::text(&head, source_file)?, 0)),
            Self::CALL => {
                let name = head.child_by_field_name("target")?;
                let arity = Self::arguments_child(&head)
                    .map(|arguments| arguments.named_child_count())
                    .unwrap_or(0);

                Some((Self::text(&name, source_file)?, arity))
            }
            _ => None,
        }
    }

    fn text<'src>(node: &Node, source_file: &'src File) -> Option<&'src str> {
        node.utf8_text(source_file.contents.as_bytes()).ok()
    }

    /// The `name(params)` head of a definition, reached through any `when` guards.
    fn definition_head<'tree>(node: &Node<'tree>) -> Option<Node<'tree>> {
        let first = Self::arguments_child(node)?.named_child(0)?;

        if first.kind() == Self::BINARY_OPERATOR {
            first.child_by_field_name("left")
        } else {
            Some(first)
        }
    }

    /// `call` nodes have no `arguments` *field*, so the argument list is found by kind.
    fn arguments_child<'tree>(node: &Node<'tree>) -> Option<Node<'tree>> {
        (0..node.named_child_count())
            .filter_map(|index| node.named_child(index))
            .find(|child| child.kind() == Self::ARGUMENTS)
    }
}

impl Default for Elixir {
    fn default() -> Self {
        let language = tree_sitter_elixir::language();

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

impl Language for Elixir {
    fn name(&self) -> &str {
        Self::NAME
    }

    /// Elixir has no `self`, so LCOM4 becomes call cohesion: bare `foo()` links functions,
    /// qualified `Mod.fun()` does not. Also keeps attributes out, since `visit_field` is gated.
    fn self_keyword(&self) -> Option<&str> {
        None
    }

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![Self::SOURCE]
    }

    fn if_nodes(&self) -> Vec<&str> {
        vec![Self::CONDITIONAL]
    }

    /// The clause that first proves a function is multi-clause: `visit_elsif` adds 1 to both
    /// counters, weighting the group like the single `case` it replaces.
    fn elsif_nodes(&self) -> Vec<&str> {
        vec![Self::SECOND_CLAUSE_HEAD]
    }

    fn else_nodes(&self) -> Vec<&str> {
        vec![Self::ELSE_BLOCK, Self::KEYWORD_ELSE]
    }

    fn switch_nodes(&self) -> Vec<&str> {
        vec![Self::BRANCH]
    }

    /// Later clause heads: `visit_case` adds 1 to cyclomatic only, exactly like a `case` arm.
    fn case_nodes(&self) -> Vec<&str> {
        vec![Self::CASE_CLAUSE, Self::LATER_CLAUSE_HEAD]
    }

    fn loop_nodes(&self) -> Vec<&str> {
        vec![Self::COMPREHENSION]
    }

    fn except_nodes(&self) -> Vec<&str> {
        vec![Self::RESCUE_BLOCK, Self::CATCH_BLOCK]
    }

    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![Self::TRY]
    }

    fn jump_nodes(&self) -> Vec<&str> {
        vec![]
    }

    /// Elixir has no `return`, so the `return-statements` smell correctly never fires.
    fn return_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn binary_nodes(&self) -> Vec<&str> {
        vec![Self::BOOLEAN_BINARY]
    }

    fn boolean_operator_nodes(&self) -> Vec<&str> {
        vec![
            Self::AMPERSAND_AMPERSAND,
            Self::PIPE_PIPE,
            Self::AND,
            Self::OR,
        ]
    }

    /// No field-access node in the grammar; field *counting* runs through `field_query`.
    fn field_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn call_nodes(&self) -> Vec<&str> {
        vec![Self::CALL]
    }

    fn function_nodes(&self) -> Vec<&str> {
        vec![Self::DEFINITION]
    }

    fn closure_nodes(&self) -> Vec<&str> {
        vec![Self::ANONYMOUS_FUNCTION]
    }

    fn comment_nodes(&self) -> Vec<&str> {
        vec![Self::COMMENT]
    }

    fn string_nodes(&self) -> Vec<&str> {
        vec![Self::STRING, Self::CHARLIST, Self::SIGIL]
    }

    /// Idiomatic Elixir iterates with higher-order `Enum`/`Stream` calls, not loop keywords.
    fn iterator_method_identifiers(&self) -> Vec<&str> {
        vec![
            "all?",
            "any?",
            "chunk_by",
            "chunk_while",
            "dedup_by",
            "drop_while",
            "each",
            "each_with_index",
            "filter",
            "find",
            "find_index",
            "find_value",
            "flat_map",
            "flat_map_reduce",
            "group_by",
            "map",
            "map_every",
            "map_intersperse",
            "map_join",
            "map_reduce",
            "max_by",
            "min_by",
            "min_max_by",
            "reduce",
            "reduce_while",
            "reject",
            "scan",
            "sort_by",
            "split_while",
            "split_with",
            "take_while",
            "uniq_by",
            "zip_reduce",
            "zip_with",
            "with_index",
        ]
    }

    fn dispatch_node_kind(&self, node: &Node, source_file: &File) -> &'static str {
        match node.kind() {
            Self::CALL => Self::dispatch_call(node, source_file),
            Self::BINARY_OPERATOR => Self::dispatch_binary_operator(node, source_file),
            Self::STAB_CLAUSE_KIND => Self::dispatch_stab_clause(node),
            Self::PAIR => Self::dispatch_pair(node, source_file),
            _ => node.kind(),
        }
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String) {
        let Some(target) = node.child_by_field_name("target") else {
            return (Some(Self::UNKNOWN.to_string()), Self::UNKNOWN.to_string());
        };

        match target.kind() {
            Self::IDENTIFIER => (None, node_source(&target, source_file)),
            Self::DOT => {
                let receiver = target
                    .child_by_field_name("left")
                    .map(|left| node_source(&left, source_file))
                    .unwrap_or_else(|| Self::UNKNOWN.to_string());
                let name = target
                    .child_by_field_name("right")
                    .map(|right| node_source(&right, source_file))
                    .unwrap_or_else(|| Self::UNKNOWN.to_string());
                (Some(receiver), name)
            }
            _ => (Some(Self::UNKNOWN.to_string()), Self::UNKNOWN.to_string()),
        }
    }

    /// Unreachable: `field_nodes()` is empty, so `visit_field` never fires.
    fn field_identifiers(&self, _source_file: &File, _node: &Node) -> (String, String) {
        (Self::UNKNOWN.to_string(), Self::UNKNOWN.to_string())
    }

    /// The trait default reads a `name` field, which `call` nodes lack — it would panic on
    /// every Elixir function. They have no `arguments` field either, so it is found by kind.
    fn function_name_from_node(&self, source_file: &File, node: &Node) -> String {
        let mut cursor = node.walk();
        let Some(arguments) = node
            .named_children(&mut cursor)
            .find(|child| child.kind() == Self::ARGUMENTS)
        else {
            return Self::UNKNOWN.to_string();
        };

        let mut cursor = arguments.walk();
        for child in arguments.named_children(&mut cursor) {
            let head = match child.kind() {
                Self::BINARY_OPERATOR => child.child_by_field_name("left"),
                _ => Some(child),
            };

            match head.map(|head| (head.kind(), head)) {
                Some((Self::IDENTIFIER, head)) => return node_source(&head, source_file),
                Some((Self::CALL, head)) => {
                    if let Some(target) = head.child_by_field_name("target") {
                        return node_source(&target, source_file);
                    }
                }
                _ => {}
            }
        }

        Self::UNKNOWN.to_string()
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_elixir::language()
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

    fn has_field_names(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::code::File;
    use std::collections::HashSet;
    use tree_sitter::Node;

    fn collect_dispatch_kinds(
        node: Node,
        language: &Elixir,
        file: &File,
        kinds: &mut Vec<&'static str>,
    ) {
        if node.is_named() {
            kinds.push(language.dispatch_node_kind(&node, file));
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            collect_dispatch_kinds(child, language, file, kinds);
        }
    }

    fn dispatch_kinds(source: &str) -> Vec<&'static str> {
        let file = File::from_string(Elixir::NAME, source);
        let tree = file.parse();
        let language = Elixir::default();
        let mut kinds = vec![];
        collect_dispatch_kinds(tree.root_node(), &language, &file, &mut kinds);
        kinds
    }

    fn definition_nodes<'tree>(node: Node<'tree>, file: &File, found: &mut Vec<Node<'tree>>) {
        let language = Elixir::default();

        if node.is_named() && language.dispatch_node_kind(&node, file) == Elixir::DEFINITION {
            found.push(node);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            definition_nodes(child, file, found);
        }
    }

    fn clause_index(source: &str, definition: usize) -> usize {
        let file = File::from_string(Elixir::NAME, source);
        let tree = file.parse();
        let mut definitions = vec![];
        definition_nodes(tree.root_node(), &file, &mut definitions);
        Elixir::preceding_clause_count(&definitions[definition], &file)
    }

    fn match_count(query: &tree_sitter::Query, source: &str) -> usize {
        let file = File::from_string(Elixir::NAME, source);
        let tree = file.parse();
        let mut cursor = tree_sitter::QueryCursor::new();
        cursor
            .matches(query, tree.root_node(), file.contents.as_bytes())
            .count()
    }

    #[test]
    fn registered_in_all_langs() {
        assert_eq!(crate::lang::from_str("elixir").unwrap().name(), "elixir");
    }

    #[test]
    fn parses_module() {
        let file = File::from_string(
            Elixir::NAME,
            "defmodule Foo do\n  def bar(x), do: x + 1\nend\n",
        );
        assert!(!file.parse().root_node().has_error());
    }

    #[test]
    fn mutually_exclusive() {
        let lang = Elixir::default();
        let mut kinds: Vec<&str> = vec![];

        kinds.extend(lang.if_nodes());
        kinds.extend(lang.elsif_nodes());
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
        kinds.extend(lang.block_nodes());

        let unique: HashSet<_> = kinds.iter().cloned().collect();
        assert_eq!(unique.len(), kinds.len());
    }

    #[test]
    fn synthetic_kinds_are_namespaced() {
        let synthetic = [
            Elixir::DEFINITION,
            Elixir::DEFINITION_HEAD,
            Elixir::CONDITIONAL,
            Elixir::BRANCH,
            Elixir::COMPREHENSION,
            Elixir::TRY,
            Elixir::BOOLEAN_BINARY,
            Elixir::CASE_CLAUSE,
            Elixir::STAB_CLAUSE,
            Elixir::ATTRIBUTE_NAME,
            Elixir::TYPE_DECLARATION,
        ];
        assert!(synthetic.iter().all(|kind| kind.starts_with("__elixir_")));
    }

    #[test]
    fn malformed_input_does_not_panic() {
        let file = File::from_string(Elixir::NAME, "defmodule do :::: end");
        let _tree = file.parse();
    }

    #[test]
    fn classifies_if_as_conditional() {
        assert!(dispatch_kinds("if x, do: 1, else: 2").contains(&Elixir::CONDITIONAL));
    }

    #[test]
    fn classifies_case_as_branch() {
        assert!(dispatch_kinds("case x do\n 1 -> :a\n _ -> :b\nend").contains(&Elixir::BRANCH));
    }

    #[test]
    fn classifies_def_as_definition() {
        assert!(dispatch_kinds("def f, do: 1").contains(&Elixir::DEFINITION));
    }

    #[test]
    fn classifies_defmodule_as_plain_call() {
        assert!(dispatch_kinds("defmodule M do\nend").contains(&"call"));
    }

    #[test]
    fn classifies_qualified_call_as_plain_call() {
        assert!(dispatch_kinds("Enum.map(list, fn x -> x end)").contains(&"call"));
    }

    #[test]
    fn definition_head_is_not_a_call() {
        let kinds = dispatch_kinds("def a(x), do: x");
        assert!(kinds.contains(&Elixir::DEFINITION_HEAD));
        assert!(!kinds.contains(&"call"));
    }

    #[test]
    fn guarded_definition_head_is_not_a_call_but_the_guard_is() {
        let kinds = dispatch_kinds("def a(x) when is_integer(x), do: x");
        assert!(kinds.contains(&Elixir::DEFINITION_HEAD));
        assert!(kinds.contains(&"call"));
    }

    #[test]
    fn boolean_binary_is_remapped_and_arithmetic_is_not() {
        assert!(dispatch_kinds("a and b").contains(&Elixir::BOOLEAN_BINARY));
        assert!(!dispatch_kinds("x = a + b").contains(&Elixir::BOOLEAN_BINARY));
    }

    #[test]
    fn case_arms_are_case_clauses() {
        let kinds = dispatch_kinds("case x do\n 1 -> :a\n _ -> :b\nend");
        assert_eq!(
            2,
            kinds
                .iter()
                .filter(|kind| **kind == Elixir::CASE_CLAUSE)
                .count()
        );
    }

    #[test]
    fn anonymous_function_clauses_are_not_case_clauses() {
        let kinds = dispatch_kinds("Enum.map(l, fn x -> x end)");
        assert!(kinds.contains(&Elixir::STAB_CLAUSE));
        assert!(!kinds.contains(&Elixir::CASE_CLAUSE));
    }

    #[test]
    fn rescue_and_catch_clauses_are_not_case_clauses() {
        let kinds = dispatch_kinds("try do\n a()\nrescue\n e -> e\ncatch\n :error, e -> e\nend");
        assert_eq!(
            2,
            kinds
                .iter()
                .filter(|kind| **kind == Elixir::STAB_CLAUSE)
                .count()
        );
        assert!(!kinds.contains(&Elixir::CASE_CLAUSE));
    }

    #[test]
    fn with_else_arms_are_case_clauses() {
        let kinds = dispatch_kinds("with {:ok, a} <- f() do\n a\nelse\n _ -> :err\nend");
        assert!(kinds.contains(&Elixir::CASE_CLAUSE));
    }

    #[test]
    fn receive_after_arms_are_case_clauses() {
        let kinds = dispatch_kinds("receive do\n x -> x\nafter\n 1000 -> :timeout\nend");
        assert_eq!(
            2,
            kinds
                .iter()
                .filter(|kind| **kind == Elixir::CASE_CLAUSE)
                .count()
        );
    }

    #[test]
    fn typespec_body_is_a_type_declaration_not_a_call() {
        let kinds = dispatch_kinds("@spec map(list) :: list");
        assert!(kinds.contains(&Elixir::TYPE_DECLARATION));
        assert!(!kinds.contains(&"call"));
    }

    #[test]
    fn callback_body_is_a_type_declaration_not_a_call() {
        let kinds = dispatch_kinds("@callback filter(list) :: list");
        assert!(kinds.contains(&Elixir::TYPE_DECLARATION));
        assert!(!kinds.contains(&"call"));
    }

    #[test]
    fn attribute_name_is_not_a_call() {
        let kinds = dispatch_kinds("@map %{a: 1}");
        assert!(kinds.contains(&Elixir::ATTRIBUTE_NAME));
        assert!(!kinds.contains(&"call"));
    }

    #[test]
    fn calls_in_an_attribute_value_are_still_calls() {
        let kinds = dispatch_kinds("@names Enum.map([1], fn n -> n end)");
        assert!(kinds.contains(&Elixir::ATTRIBUTE_NAME));
        assert_eq!(1, kinds.iter().filter(|kind| **kind == "call").count());
    }

    #[test]
    fn argument_position_types_are_type_declarations() {
        let kinds = dispatch_kinds("@spec fetch(map()) :: term()");
        assert_eq!(0, kinds.iter().filter(|kind| **kind == "call").count());
        assert_eq!(
            3,
            kinds
                .iter()
                .filter(|kind| **kind == Elixir::TYPE_DECLARATION)
                .count()
        );
    }

    #[test]
    fn nested_types_inside_a_type_declaration_are_type_declarations() {
        let kinds = dispatch_kinds("@type t :: %{a: map(), b: [map()]}");
        assert!(!kinds.contains(&"call"));
    }

    #[test]
    fn defdelegate_head_is_not_a_call() {
        let kinds = dispatch_kinds("defdelegate map(list, fun), to: Enum");
        assert!(kinds.contains(&Elixir::DEFINITION_HEAD));
        assert_eq!(1, kinds.iter().filter(|kind| **kind == "call").count());
    }

    #[test]
    fn calls_in_a_function_body_are_not_type_declarations() {
        let kinds = dispatch_kinds(
            "defmodule M do\n def go(l) do\n  Enum.map(l, fn n -> n end)\n end\nend",
        );
        assert!(kinds.contains(&"call"));
        assert!(!kinds.contains(&Elixir::TYPE_DECLARATION));
    }

    #[test]
    fn apply_with_computed_target_does_not_panic() {
        let file = File::from_string(Elixir::NAME, "apply(m, f, a)");
        let tree = file.parse();
        let call = tree.root_node().named_child(0).unwrap();
        assert_eq!(
            (None, "apply".to_string()),
            Elixir::default().call_identifiers(&file, &call)
        );
    }

    #[test]
    fn class_query_matches_module_protocol_and_impl() {
        let language = Elixir::default();
        assert_eq!(
            1,
            match_count(language.class_query(), "defmodule App.Foo do\nend")
        );
        assert_eq!(
            2,
            match_count(
                language.class_query(),
                "defmodule A do\n defmodule B do\n end\nend"
            )
        );
        assert_eq!(
            1,
            match_count(
                language.class_query(),
                "defimpl P, for: Foo do\n def f(x), do: x\nend"
            )
        );
    }

    #[test]
    fn function_query_matches_every_definition_form() {
        let language = Elixir::default();
        let query = language.function_declaration_query();
        assert_eq!(
            1,
            match_count(query, "defmodule M do\n def a(x, y), do: x\nend")
        );
        assert_eq!(1, match_count(query, "defmodule M do\n def a, do: 1\nend"));
        assert_eq!(
            1,
            match_count(
                query,
                "defmodule M do\n def a(x) when is_integer(x), do: x\nend"
            )
        );
        assert_eq!(
            1,
            match_count(query, "defmodule M do\n def a when true, do: 1\nend")
        );
        assert_eq!(
            3,
            match_count(
                query,
                "defmodule M do\n def a(x), do: x\n defp b, do: 1\n defmacro c(y), do: y\nend"
            )
        );
    }

    #[test]
    fn field_query_counts_attributes_and_struct_fields() {
        let language = Elixir::default();
        let query = language.field_query();
        assert_eq!(
            4,
            match_count(
                query,
                "defmodule M do\n @foo 1\n @bar 2\n defstruct [:x, :y]\n def a, do: @foo\nend"
            )
        );
        assert_eq!(
            2,
            match_count(query, "defmodule M do\n defstruct x: 1, y: 2\nend")
        );
        assert_eq!(
            2,
            match_count(query, "defmodule M do\n defstruct [:a, b: 1]\nend")
        );
    }

    #[test]
    fn field_query_ignores_reserved_attributes() {
        let language = Elixir::default();
        let source = "defmodule M do\n @moduledoc \"m\"\n @doc \"d\"\n @spec f(integer) :: integer\n @impl true\n @enforce_keys [:a]\n @behaviour GenServer\n defstruct [:a]\n def f(x), do: x\nend";
        assert_eq!(1, match_count(language.field_query(), source));
    }

    #[test]
    fn local_call_has_no_receiver() {
        let file = File::from_string(Elixir::NAME, "foo(1)");
        let tree = file.parse();
        let call = tree.root_node().named_child(0).unwrap();
        assert_eq!(
            (None, "foo".to_string()),
            Elixir::default().call_identifiers(&file, &call)
        );
    }

    #[test]
    fn qualified_call_has_receiver() {
        let file = File::from_string(Elixir::NAME, "Mod.fun(1)");
        let tree = file.parse();
        let call = tree.root_node().named_child(0).unwrap();
        assert_eq!(
            (Some("Mod".to_string()), "fun".to_string()),
            Elixir::default().call_identifiers(&file, &call)
        );
    }

    #[test]
    fn dynamic_call_target_does_not_panic() {
        let file = File::from_string(Elixir::NAME, "fun.(1)");
        let tree = file.parse();
        let call = tree.root_node().named_child(0).unwrap();
        let (receiver, name) = Elixir::default().call_identifiers(&file, &call);
        assert_eq!(
            (Some("fun".to_string()), "<UNKNOWN>".to_string()),
            (receiver, name)
        );
    }

    fn function_name(source: &str) -> String {
        let file = File::from_string(Elixir::NAME, source);
        let tree = file.parse();
        let definition = tree.root_node().named_child(0).unwrap();
        Elixir::default().function_name_from_node(&file, &definition)
    }

    #[test]
    fn function_name_from_parenthesized_definition() {
        assert_eq!("a", function_name("def a(x), do: x"));
    }

    #[test]
    fn function_name_from_bare_definition() {
        assert_eq!("a", function_name("def a, do: 1"));
    }

    #[test]
    fn function_name_from_guarded_definition() {
        assert_eq!("a", function_name("def a(x) when g(x), do: x"));
    }

    #[test]
    fn keyword_form_else_is_an_else() {
        assert!(dispatch_kinds("if x, do: 1, else: 2").contains(&Elixir::KEYWORD_ELSE));
    }

    #[test]
    fn keyword_form_else_is_recognized_for_every_conditional() {
        assert!(dispatch_kinds("unless x, do: 1, else: 2").contains(&Elixir::KEYWORD_ELSE));
        assert!(
            dispatch_kinds("with {:ok, a} <- f(), do: a, else: (_ -> :err)")
                .contains(&Elixir::KEYWORD_ELSE)
        );
    }

    #[test]
    fn an_ordinary_else_keyword_is_not_an_else() {
        assert!(!dispatch_kinds("config(else: 1)").contains(&Elixir::KEYWORD_ELSE));
        assert!(!dispatch_kinds("%{else: 1}").contains(&Elixir::KEYWORD_ELSE));
    }

    #[test]
    fn do_keyword_is_not_an_else() {
        assert!(!dispatch_kinds("if x, do: 1").contains(&Elixir::KEYWORD_ELSE));
    }

    #[test]
    fn a_single_clause_function_is_a_first_clause() {
        assert_eq!(0, clause_index("defmodule M do\n def f(x), do: x\nend", 0));
    }

    #[test]
    fn later_clauses_of_the_same_function_are_numbered() {
        let source = "defmodule M do\n def f(0), do: 1\n def f(n), do: n\n def f(_), do: 0\nend";
        assert_eq!(0, clause_index(source, 0));
        assert_eq!(1, clause_index(source, 1));
        assert_eq!(2, clause_index(source, 2));
    }

    #[test]
    fn clauses_are_grouped_across_interleaved_attributes() {
        let source = "defmodule M do\n @doc \"d\"\n def f(0), do: 1\n @spec f(integer) :: integer\n def f(n), do: n\nend";
        assert_eq!(1, clause_index(source, 1));
    }

    #[test]
    fn a_different_arity_is_a_different_function() {
        let source = "defmodule M do\n def f(a), do: a\n def f(a, b), do: {a, b}\nend";
        assert_eq!(0, clause_index(source, 1));
    }

    #[test]
    fn a_different_name_is_a_different_function() {
        let source = "defmodule M do\n def f(a), do: a\n def g(a), do: a\nend";
        assert_eq!(0, clause_index(source, 1));
    }

    #[test]
    fn guarded_clauses_of_one_function_are_grouped() {
        let source = "defmodule M do\n def f(x) when x < 0, do: :neg\n def f(x) when x > 0, do: :pos\n def f(_), do: :zero\nend";
        assert_eq!(2, clause_index(source, 2));
    }

    #[test]
    fn zero_arity_clauses_are_grouped() {
        let source = "defmodule M do\n def f, do: 1\n def f, do: 2\nend";
        assert_eq!(1, clause_index(source, 1));
    }

    #[test]
    fn clauses_in_different_modules_are_not_grouped() {
        let source =
            "defmodule A do\n def f(x), do: x\nend\n\ndefmodule B do\n def f(x), do: x\nend";
        assert_eq!(0, clause_index(source, 1));
    }
}
