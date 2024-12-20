use lazy_static::lazy_static;

pub const QLTY_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_COMMIT_QLTY: &str = env!("GIT_COMMIT_QLTY");
pub const BUILD_PROFILE: &str = env!("BUILD_PROFILE");
pub const BUILD_DATE: &str = env!("BUILD_DATE");

lazy_static! {
    pub static ref SHORT_ARCH: String = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => std::env::consts::ARCH,
    }
    .to_string();
    pub static ref OS_IDENTIFIER: String =
        format!("{}-{}", std::env::consts::OS, SHORT_ARCH.as_str());
    pub static ref BUILD_IDENTIFIER: String = match BUILD_PROFILE {
        "release" => format!("({} {})", GIT_COMMIT_QLTY, BUILD_DATE),
        _ => format!("({} {} {})", GIT_COMMIT_QLTY, BUILD_PROFILE, BUILD_DATE),
    };
    pub static ref LONG_VERSION: String = format!(
        "{} {} {}",
        QLTY_VERSION,
        OS_IDENTIFIER.as_str(),
        BUILD_IDENTIFIER.as_str()
    );
}

pub fn qlty_semver() -> semver::Version {
    semver::Version::parse(QLTY_VERSION).expect("QLTY_VERSION should be a valid semver")
}
