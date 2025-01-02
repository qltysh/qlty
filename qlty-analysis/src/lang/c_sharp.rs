use crate::code::File;
use crate::code::{child_source, node_source};
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
