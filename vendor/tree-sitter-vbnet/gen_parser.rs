use std::process::Command;

fn main() {
    let status = Command::new("tree-sitter")
        .args(&["generate"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("Failed to run tree-sitter generate");

    if !status.success() {
        panic!("tree-sitter generate failed");
    }
}
