pub trait EnvSource: std::fmt::Debug + Send + Sync {
    fn var(&self, name: &str) -> Option<String>;
}

#[derive(Debug, Clone, Default)]
pub struct SystemEnv;

impl EnvSource for SystemEnv {
    fn var(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}
