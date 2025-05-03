use crate::code::{node_source, File};
use core::fmt;
use std::sync::Arc;
use tree_sitter::{Node, Parser, Query};

mod csharp;
mod go;
mod java;
mod javascript;
mod kotlin;
mod php;
mod python;
mod ruby;
mod rust;
mod swift;
mod tsx;
mod typescript;
mod typescript_common;

pub use {
    csharp::*, go::*, java::*, javascript::*, kotlin::*, php::*, python::*, ruby::*, rust::*,
    swift::*, tsx::*, typescript::*,
};

#[allow(clippy::borrowed_box)]
pub fn from_str(name: &str) -> Option<&Box<dyn Language + Sync>> {
    ALL_LANGS.iter().find(|language| language.name() == name)
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref ALL_LANGS: Vec<Box<dyn Language + Sync>> = {
        vec![
            Box::<csharp::CSharp>::default(),
            Box::<php::Php>::default(),
            Box::<kotlin::Kotlin>::default(),
            Box::<go::Go>::default(),
            Box::<java::Java>::default(),
            Box::<javascript::JavaScript>::default(),
            Box::<python::Python>::default(),
            Box::<ruby::Ruby>::default(),
            Box::<rust::Rust>::default(),
            Box::<swift::Swift>::default(),
            Box::<typescript::TypeScript>::default(),
            Box::<tsx::TSX>::default(),
        ]
    };
}

pub trait Language {
    fn name(&self) -> &str;

    fn self_keyword(&self) -> Option<&str>;

    fn invisible_container_nodes(&self) -> Vec<&str> {
        vec![]
    }
    fn if_nodes(&self) -> Vec<&str>;
    fn elsif_nodes(&self) -> Vec<&str> {
        vec![]
    }
    fn else_nodes(&self) -> Vec<&str> {
        vec![]
    }
    fn ternary_nodes(&self) -> Vec<&str> {
        vec![]
    }
    fn switch_nodes(&self) -> Vec<&str>;
    fn case_nodes(&self) -> Vec<&str>;
    fn loop_nodes(&self) -> Vec<&str>;
    fn except_nodes(&self) -> Vec<&str> {
        vec![]
    }
    fn try_expression_nodes(&self) -> Vec<&str> {
        vec![]
    }
    fn conditional_assignment_nodes(&self) -> Vec<&str> {
        vec![]
    }
    fn jump_nodes(&self) -> Vec<&str>;
    fn return_nodes(&self) -> Vec<&str>;
    fn binary_nodes(&self) -> Vec<&str>;
    fn field_nodes(&self) -> Vec<&str>;
    fn call_nodes(&self) -> Vec<&str>;
    fn function_nodes(&self) -> Vec<&str>;
    fn closure_nodes(&self) -> Vec<&str>;
    fn comment_nodes(&self) -> Vec<&str>;
    fn string_nodes(&self) -> Vec<&str>;
    fn block_nodes(&self) -> Vec<&str> {
        vec![]
    }

    fn boolean_operator_nodes(&self) -> Vec<&str>;

    fn constructor_names(&self) -> Vec<&str> {
        vec![]
    }

    fn destructor_names(&self) -> Vec<&str> {
        vec![]
    }

    fn is_decorator_function(&self, _node: &Node) -> bool {
        false
    }

    fn is_instance_method(&self, _file: &File, _node: &Node) -> bool {
        true
    }

    fn is_jump_label(&self, _node: &Node) -> bool {
        false
    }

    fn all_operators(&self) -> Vec<&str> {
        vec![]
    }

    fn all_operands(&self) -> Vec<&str> {
        vec![]
    }

    fn iterator_method_identifiers(&self) -> Vec<&str> {
        vec![]
    }

    fn call_identifiers(&self, source_file: &File, node: &Node) -> (Option<String>, String);
    fn field_identifiers(&self, source_file: &File, node: &Node) -> (String, String);

    fn tree_sitter_language(&self) -> tree_sitter::Language;

    fn class_query(&self) -> &Query;
    fn function_declaration_query(&self) -> &Query;
    fn field_query(&self) -> &Query;
    fn implementation_query(&self) -> Option<&Query> {
        None
    }

    fn has_labeled_jumps(&self) -> bool {
        false
    }

    fn query(&self, query_source: &str) -> Query {
        Query::new(&self.tree_sitter_language(), query_source).unwrap()
    }

    fn parser(&self) -> Parser {
        let mut parser = Parser::new();
        parser
            .set_language(&self.tree_sitter_language())
            .expect("Error loading grammar");
        parser
    }

    fn has_field_names(&self) -> bool {
        false
    }

    fn sanitize_parameter_name(&self, parameter_name: String) -> Option<String> {
        if let Some(self_keyword) = self.self_keyword() {
            if parameter_name != self_keyword
                && parameter_name != format!("&{}", self_keyword)
                && parameter_name != format!("&mut {}", self_keyword)
            {
                Some(parameter_name)
            } else {
                None
            }
        } else {
            Some(parameter_name)
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
            let parameter_name = node_source(&parameter_node, source_file);

            let sanitized_parameter_name = self.sanitize_parameter_name(parameter_name);
            match sanitized_parameter_name {
                Some(sanitized_parameter_name) => parameter_names.push(sanitized_parameter_name),
                _ => {}
            };
        }
        parameter_names
    }

    fn function_name_node<'a>(&'a self, node: &'a Node) -> Node<'a> {
        node.child_by_field_name("name").unwrap()
    }
}

impl fmt::Display for dyn Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Language[{}]", self.name())
    }
}

impl fmt::Debug for dyn Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Language[{}]", self.name())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn language_names() {
        let variant = crate::lang::from_str("rust").unwrap();
        assert_eq!("rust", variant.name());

        let rust = crate::lang::Rust::default();
        assert_eq!(rust.name(), "rust");
    }

    #[test]
    fn language_parser() {
        crate::lang::Rust::default().parser();
    }
}
