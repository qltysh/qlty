use crate::{
    telemetry::{analytics::Track, AnalyticsClient},
    Arguments, CommandError, CommandSuccess,
};
use anyhow::{anyhow, Result};
use clap::Args;
use qlty_config::version::BUILD_PROFILE;
use std::path::PathBuf;
use tracing::{debug, info};

const SENTRY_DSN: Option<&str> = option_env!("SENTRY_DSN");

#[derive(Args, Debug)]
pub struct Telemetry {
    #[arg(long)]
    pub track: Option<PathBuf>,

    #[arg(long)]
    pub panic: Option<PathBuf>,
}

impl Telemetry {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        if self.track.is_some() {
            return self.send_track();
        }

        if self.panic.is_some() {
            return self.send_panic();
        }

        CommandSuccess::ok()
    }

    fn send_track(&self) -> Result<CommandSuccess, CommandError> {
        let payload_path = self.track.clone().unwrap();
        let payload = std::fs::read_to_string(&payload_path).map_err(|err| {
            anyhow!(
                "Unable to read telemetry payload file: {} ({:?})",
                payload_path.display(),
                err
            )
        })?;

        let client = AnalyticsClient::new()?;
        let event: Track = serde_json::from_str(&payload).unwrap();
        client.send_track(event)?;

        CommandSuccess::ok()
    }

    fn send_panic(&self) -> Result<CommandSuccess, CommandError> {
        let dsn = SENTRY_DSN.unwrap_or_default();
        if dsn.is_empty() {
            // ignore telemetry if no DSN is set
            debug!("No Sentry DSN set, skipping telemetry");
            return CommandSuccess::ok();
        }

        let payload_path = self.panic.clone().unwrap();
        let payload = std::fs::read_to_string(&payload_path).map_err(|err| {
            anyhow!(
                "Unable to read telemetry payload file: {} ({:?})",
                payload_path.display(),
                err
            )
        })?;

        let event: ::sentry::protocol::Event = serde_json::from_str(&payload).unwrap();

        let _guard = sentry::init((
            dsn.to_string(),
            sentry::ClientOptions {
                debug: true,
                release: sentry::release_name!(),
                environment: match BUILD_PROFILE {
                    "release" => Some("production".into()),
                    _ => Some("development".into()),
                },
                default_integrations: false,
                ..Default::default()
            },
        ));

        sentry::capture_event(event);

        info!(
            "Telemetry payload sent to Sentry: {}",
            payload_path.display()
        );

        CommandSuccess::ok()
    }
}
