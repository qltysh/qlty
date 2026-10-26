# tree-sitter-rust 0.24.2, ABI 14

The Rust grammar that slopdetect pins (`tree-sitter-rust` 0.24.2, MIT licensed,
see LICENSE), regenerated from its `grammar.json` with the tree-sitter CLI
(`tree-sitter generate --abi 14`) so the workspace's tree-sitter 0.22 runtime
can load it. The parse tables are identical to the published 0.24.2 crate
(same state, symbol, and large-state counts); only the ABI metadata differs.

The exported symbol is renamed to `tree_sitter_rust_slop_one` so it does not
collide with the `tree_sitter_rust` symbol from the tree-sitter-rust 0.21.2
crate that Qlty's measurement uses.

SlopOne uses this grammar only to select source (removing inline `mod test`
modules). Measurement keeps Qlty's grammar.
