fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = std::path::PathBuf::from(&manifest_dir);
    let src_dir = manifest_path.join("src");
    let parser_path = src_dir.join("parser.c");

    if needs_regeneration(&parser_path) {
        let grammar_js = manifest_path.join("grammar.js");
        if grammar_js.exists() {
            let status = std::process::Command::new("tree-sitter")
                .args(["generate"])
                .current_dir(&manifest_dir)
                .status();
            match status {
                Ok(s) if s.success() => {
                    println!("cargo:warning=Regenerated parser.c for ABI v14 compatibility");
                }
                _ => {
                    panic!(
                        "parser.c has ABI version > 14 (incompatible with tree-sitter 0.22.x). \
                         Install tree-sitter-cli 0.22.6 (`cargo install tree-sitter-cli@0.22.6`) \
                         and run `tree-sitter generate` in vendor/tree-sitter-vbnet/"
                    );
                }
            }
        } else {
            panic!(
                "parser.c has ABI version > 14 and grammar.js not found for regeneration. \
                 Please regenerate the parser with tree-sitter-cli 0.22.6."
            );
        }
    }

    let mut c_config = cc::Build::new();
    c_config.std("c11").include(&src_dir);

    #[cfg(target_env = "msvc")]
    c_config.flag("-utf-8");

    c_config.file(&parser_path);
    println!("cargo:rerun-if-changed={}", parser_path.to_str().unwrap());
    println!("cargo:rerun-if-changed=grammar.js");

    c_config.compile("tree-sitter-vb-dotnet");
}

fn needs_regeneration(parser_path: &std::path::Path) -> bool {
    if let Ok(contents) = std::fs::read_to_string(parser_path) {
        for line in contents.lines().take(30) {
            if line.contains("#define LANGUAGE_VERSION") {
                if let Some(version_str) = line.split_whitespace().last() {
                    if let Ok(version) = version_str.parse::<u32>() {
                        return version > 14;
                    }
                }
            }
        }
    }
    false
}
