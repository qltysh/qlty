use crate::tool::command_error::ToolCommandError;
use anyhow::Error;
use chrono::Utc;
use qlty_types::analysis::v1::{Message, MessageLevel};

pub fn install_error_message(name: &str, error: &Error) -> Message {
    let command_error = error
        .chain()
        .find_map(|cause| cause.downcast_ref::<ToolCommandError>());

    let mut message = format!("Error installing tool {name}");
    let details = install_error_details(error, command_error);
    let mut ty = "executor.install.error";

    if let Some(failure) = command_error.and_then(|command_error| command_error.failure.as_ref()) {
        message = format!("{message}: {}", failure.summary);
        ty = failure.kind.message_ty();
    }

    Message {
        timestamp: Some(Utc::now().into()),
        module: "qlty_check::executor".to_string(),
        ty: ty.to_string(),
        level: MessageLevel::Error.into(),
        message,
        details,
        ..Default::default()
    }
}

fn install_error_details(error: &Error, command_error: Option<&ToolCommandError>) -> String {
    if let Some(command_error) = command_error {
        let output_tail = command_error.output_tail();

        if !output_tail.is_empty() {
            return format!("{error}\n\n{output_tail}");
        }
    }

    format!("{error:#}")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tool::install_failure::{InstallFailure, InstallFailureKind, BUILD_SECRETS_URL};
    use anyhow::anyhow;

    const AUTH_FAILED_STDERR: &str = "npm ERR! code E401\nnpm ERR! 401 Unauthorized - GET https://npm.pkg.github.com/@marschattha%2feslint-config-qlty-demo - authentication token not provided";

    fn command_error(stderr: &str, failure: Option<InstallFailure>) -> ToolCommandError {
        ToolCommandError {
            command: vec!["npm".to_string(), "install".to_string()],
            exit_code: 1,
            stdout: String::new(),
            stderr: stderr.to_string(),
            failure,
        }
    }

    fn wrapped(stderr: &str, failure: Option<InstallFailure>) -> Error {
        Error::from(command_error(stderr, failure)).context("Error installing eslint@9.7.0.")
    }

    #[test]
    fn auth_failure_upgrades_message_and_ty() {
        let failure = InstallFailure {
            kind: InstallFailureKind::AuthenticationFailed,
            summary: format!("npm registry authentication failed (see {BUILD_SECRETS_URL})"),
        };

        let message = install_error_message("eslint", &wrapped(AUTH_FAILED_STDERR, Some(failure)));

        assert_eq!(
            message.message,
            format!(
                "Error installing tool eslint: npm registry authentication failed (see {BUILD_SECRETS_URL})"
            )
        );
        assert_eq!(message.ty, "executor.install.auth_error");
        assert!(message
            .details
            .contains("authentication token not provided"));
    }

    #[test]
    fn maybe_private_failure_gets_not_found_ty() {
        let failure = InstallFailure {
            kind: InstallFailureKind::PackageMaybePrivate,
            summary: "npm package not found (it may be private)".to_string(),
        };

        let message =
            install_error_message("eslint", &wrapped("npm error code E404", Some(failure)));

        assert_eq!(
            message.message,
            "Error installing tool eslint: npm package not found (it may be private)"
        );
        assert_eq!(message.ty, "executor.install.package_not_found");
    }

    #[test]
    fn unclassified_failure_stays_generic() {
        let message = install_error_message("clippy", &wrapped("error: linking failed", None));

        assert_eq!(message.message, "Error installing tool clippy");
        assert_eq!(message.ty, "executor.install.error");
    }

    #[test]
    fn classified_ty_values_match_the_cli_filter_prefix() {
        for kind in [
            InstallFailureKind::AuthenticationFailed,
            InstallFailureKind::AccessDenied,
            InstallFailureKind::PackageMaybePrivate,
            InstallFailureKind::UnsupportedDependencyProtocol,
        ] {
            assert!(kind.message_ty().starts_with("executor.install."));
        }
    }

    #[test]
    fn details_append_command_output() {
        let message = install_error_message("eslint", &wrapped(AUTH_FAILED_STDERR, None));

        assert!(message
            .details
            .starts_with("Error installing eslint@9.7.0.\n\nnpm ERR! code E401"));
    }

    #[test]
    fn details_without_command_output() {
        let message = install_error_message("eslint", &Error::from(command_error("", None)));

        assert_eq!(
            message.details,
            r#"Command ["npm", "install"] exited with code 1"#
        );
    }

    #[test]
    fn details_without_command_error_include_the_cause_chain() {
        let error =
            anyhow!("No package file provided").context("Error installing rubocop@bundled.");

        let message = install_error_message("rubocop", &error);

        assert_eq!(
            message.details,
            "Error installing rubocop@bundled.: No package file provided"
        );
    }

    #[test]
    fn message_fields_without_command_error() {
        let message = install_error_message("eslint", &anyhow!("boom"));

        assert_eq!(message.ty, "executor.install.error");
        assert_eq!(message.level, MessageLevel::Error as i32);
        assert_eq!(message.message, "Error installing tool eslint");
        assert_eq!(message.details, "boom");
    }
}
