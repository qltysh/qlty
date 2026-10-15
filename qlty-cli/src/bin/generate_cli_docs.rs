use clap::CommandFactory;
use qlty::docs_export::export_command_tree;
use qlty::Arguments;

fn main() {
    let export = export_command_tree(Arguments::command());
    println!("{}", serde_json::to_string_pretty(&export).unwrap());
}
