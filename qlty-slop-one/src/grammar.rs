//! The pinned Rust grammar used for source selection (see `grammar/`).

extern "C" {
    fn tree_sitter_rust_slop_one() -> tree_sitter::Language;
}

/// The tree-sitter-rust 0.24.2 grammar, the version slopdetect pins.
pub fn rust() -> tree_sitter::Language {
    unsafe { tree_sitter_rust_slop_one() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_error(source: &str) -> bool {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&rust()).unwrap();
        parser.parse(source, None).unwrap().root_node().has_error()
    }

    #[test]
    fn parses_raw_borrows() {
        assert!(!has_error("fn f(p: *const i32) { let r = &raw const *p; }"));
    }

    #[test]
    fn parses_unsafe_extern_blocks() {
        assert!(!has_error("unsafe extern \"C\" { fn f(); }"));
    }

    #[test]
    fn reports_invalid_syntax() {
        assert!(has_error("fn f( {"));
    }
}
