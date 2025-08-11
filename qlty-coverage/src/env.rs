pub trait EnvSource: std::fmt::Debug {
    fn var(&self, name: &str) -> Option<String>;
}

#[derive(Debug, Clone, Default)]
pub struct SystemEnv {}

impl EnvSource for SystemEnv {
    fn var(&self, name: &str) -> Option<String> {
        // TODO: What's the Rust macro for Result<T> to Option<T> again?
        std::env::var(name).ok()
    }
}
