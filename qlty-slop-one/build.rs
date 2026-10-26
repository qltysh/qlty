use std::path::Path;

fn main() {
    let src = Path::new("grammar/tree-sitter-rust-0.24.2-abi14/src");
    let mut build = cc::Build::new();
    build.std("c11").include(src).warnings(false);
    #[cfg(target_env = "msvc")]
    build.flag("-utf-8");
    for name in ["parser.c", "scanner.c"] {
        let path = src.join(name);
        build.file(&path);
        println!("cargo:rerun-if-changed={}", path.display());
    }
    build.compile("tree-sitter-rust-slop-one");
}
