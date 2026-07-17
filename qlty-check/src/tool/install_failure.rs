pub const BUILD_SECRETS_URL: &str = "https://docs.qlty.sh/cloud/build-secrets";

#[derive(Debug)]
pub struct InstallFailure {
    pub kind: InstallFailureKind,
    pub summary: String,
}

#[derive(Debug)]
pub enum InstallFailureKind {
    AuthenticationFailed,
    AccessDenied,
    PackageMaybePrivate,
    UnsupportedDependencyProtocol,
}

impl InstallFailureKind {
    pub fn message_ty(&self) -> &'static str {
        match self {
            InstallFailureKind::AuthenticationFailed | InstallFailureKind::AccessDenied => {
                "executor.install.auth_error"
            }
            InstallFailureKind::PackageMaybePrivate => "executor.install.package_not_found",
            InstallFailureKind::UnsupportedDependencyProtocol => {
                "executor.install.unsupported_protocol"
            }
        }
    }
}
