use ignore::overrides::Override;
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
pub use ignore::WalkParallel;
use std::path::PathBuf;

/// Maximum allowable file size (in bytes) for processing.
pub const MAX_FILE_SIZE: usize = 2_098_000;

/// A helper structure for configuring and building a file or directory walker.
#[derive(Debug, Clone, Default)]
pub struct WalkerBuilder {
    /// A list of patterns to ignore during traversal.
    pub ignores: Vec<String>,
}

impl WalkerBuilder {
    /// Creates a new `WalkerBuilder` instance with the given root directory and paths.
    pub fn new() -> Self {
        Self {
            ignores: vec![String::from(".git/")],
        }
    }

    /// Constructs a `Walk` object based on the builder's paths and ignores.
    pub fn build(&self, paths: &[PathBuf]) -> WalkParallel {
        let mut builder = WalkBuilder::new(&paths[0]);

        for path in &paths[1..] {
            builder.add(path);
        }

        builder
            .follow_links(false)
            .max_filesize(Some(MAX_FILE_SIZE as u64))
            .hidden(false)
            .overrides(self.overrides())
            .threads(num_cpus::get());
        builder.build_parallel()
    }

    fn overrides(&self) -> Override {
        // TODO: Is it safe to use std::env::current_dir here?
        let mut builder = OverrideBuilder::new(
            std::env::current_dir().expect("Unable to identify current directory"),
        );

        builder.case_insensitive(true).unwrap();

        for glob in &self.ignores {
            builder.add(&format!("!{}", glob)).unwrap();
        }

        builder.build().unwrap()
    }
}
