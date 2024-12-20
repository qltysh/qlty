use self::sanitize::sanitize_command;
use crate::arguments::is_subcommand;
use crate::telemetry::analytics::{event_context, event_user, Track};
use crate::telemetry::git::repository_identifier;
use crate::{errors::CommandError, success::CommandSuccess};
use anyhow::Result;
use mac_address::get_mac_address;
use sentry::integrations::contexts::utils::os_context;
use sentry::integrations::panic::message_from_panic_info;
use sentry::protocol::map::Entry;
use sentry_backtrace::current_stacktrace;
use serde_json::json;
use std::path::PathBuf;
use std::time::Instant;
use time::OffsetDateTime;
use tracing::{debug, trace, warn};
use uuid::Uuid;

pub mod analytics;
mod git;
mod locale;
mod sanitize;

pub use analytics::AnalyticsClient;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const DETACHED_PROCESS: u32 = 0x00000008;

#[derive(Clone)]
pub struct Telemetry {
    command: String,
    pub start_time: Instant,
    repository_path: Option<PathBuf>,
    level: TelemetryLevel,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TelemetryLevel {
    Off,
    Errors,
    All,
}

impl Telemetry {
    pub fn new(command: &str, start_time: Instant, repository_path: Option<PathBuf>) -> Self {
        Telemetry {
            command: command.to_owned(),
            start_time,
            repository_path: repository_path.clone(),
            level: Telemetry::current_level(),
        }
    }

    pub fn track_command_success(&self, command_success: &CommandSuccess) -> Result<()> {
        if self.level == TelemetryLevel::Off || self.level == TelemetryLevel::Errors {
            return Ok(());
        }

        let mut properties = self.basic_properties();

        properties["Success"] = json!("true");
        properties["Exit Code"] = json!(command_success.exit_code());
        properties["Failed"] = json!(format!("{:?}", command_success.fail));

        if let Some(trigger) = command_success.trigger {
            properties["Trigger"] = json!(format!("{:?}", trigger));
        }

        if let Some(issues_count) = command_success.issues_count {
            properties["Issues Count"] = json!(issues_count);
        }

        self.track("Command Run", properties)
    }

    pub fn track_command_error(&self, command_error: &CommandError) -> Result<()> {
        if self.level == TelemetryLevel::Off {
            return Ok(());
        }

        let mut properties = self.basic_properties();

        properties["Success"] = json!("false");
        properties["Exit Code"] = json!(command_error.exit_code());
        properties["Error"] = json!(format!("{}", command_error));

        self.track("Command Error", properties)
    }

    pub fn panic(&self, panic_info: &std::panic::PanicHookInfo<'_>) -> Result<()> {
        if self.level == TelemetryLevel::Off {
            return Ok(());
        }

        let message = message_from_panic_info(panic_info);
        let mut event = ::sentry::protocol::Event {
            exception: vec![::sentry::protocol::Exception {
                ty: "panic".into(),
                mechanism: Some(::sentry::protocol::Mechanism {
                    ty: "panic".into(),
                    handled: Some(false),
                    ..Default::default()
                }),
                value: Some(message.to_string()),
                stacktrace: current_stacktrace(),
                ..Default::default()
            }]
            .into(),
            level: ::sentry::Level::Fatal,
            ..Default::default()
        };

        if let Entry::Vacant(entry) = event.contexts.entry("os".to_string()) {
            if let Some(os) = os_context() {
                entry.insert(os);
            }
        }

        self.background_panic(event)
    }

    fn track(&self, event: &str, properties: serde_json::Value) -> Result<()> {
        trace!("Tracking event (foreground): {}: {:?}", event, properties);
        let message_id = Uuid::new_v4().to_string();

        let track = Track {
            user: event_user(None, anonymous_id()?),
            event: event.to_owned(),
            properties,
            context: Some(event_context()),
            timestamp: Some(OffsetDateTime::now_utc()),
            extra: [("messageId".to_owned(), json!(message_id))]
                .iter()
                .cloned()
                .collect(),
            integrations: None,
        };

        self.background_track(&message_id, track)
    }

    fn background_track(&self, message_id: &str, event: Track) -> Result<()> {
        const COMMAND: &str = "telemetry";
        const COMMAND_ARG: &str = "--track";

        let payload = serde_json::to_string(&event)?;
        let filename = format!("qlty-event-{}.json", message_id);
        let tempfile_path = std::env::temp_dir().join(filename);

        std::fs::write(&tempfile_path, payload)?;
        debug!(
            "Tracking event: {} {} {} {}",
            std::env::current_exe()
                .expect("Could not determine current executable path")
                .display(),
            COMMAND,
            COMMAND_ARG,
            tempfile_path.display()
        );

        #[cfg(unix)]
        {
            if let Ok(fork::Fork::Child) = fork::fork() {
                fork::setsid()
                    .and_then(|_| {
                        fork::close_fd()?;
                        if let Ok(fork::Fork::Child) = fork::fork() {
                            let _ = exec::Command::new(std::env::current_exe().unwrap())
                                .arg(COMMAND)
                                .arg(COMMAND_ARG)
                                .arg(tempfile_path)
                                .exec();
                        }
                        Ok(())
                    })
                    .ok();
            }
        }

        #[cfg(windows)]
        {
            std::process::Command::new(std::env::current_exe().unwrap())
                .arg(COMMAND)
                .arg(COMMAND_ARG)
                .arg(tempfile_path)
                .creation_flags(DETACHED_PROCESS)
                .spawn()?;
        }

        Ok(())
    }

    fn background_panic(&self, event: ::sentry::protocol::Event) -> Result<()> {
        const COMMAND: &str = "telemetry";
        const COMMAND_ARG: &str = "--panic";

        let payload = serde_json::to_string(&event)?;
        let filename = format!("qlty-sentry-event-{}.json", event.event_id);
        let tempfile_path = std::env::temp_dir().join(filename);

        std::fs::write(&tempfile_path, payload)?;
        debug!(
            "Tracking panic: {} {} {} {}",
            std::env::current_exe()
                .expect("Could not determine current executable path")
                .display(),
            COMMAND,
            COMMAND_ARG,
            tempfile_path.display()
        );

        #[cfg(unix)]
        {
            if let Ok(fork::Fork::Child) = fork::fork() {
                fork::setsid()
                    .and_then(|_| {
                        fork::close_fd()?;
                        if let Ok(fork::Fork::Child) = fork::fork() {
                            let _ = exec::Command::new(std::env::current_exe().unwrap())
                                .arg(COMMAND)
                                .arg(COMMAND_ARG)
                                .arg(tempfile_path)
                                .exec();
                        }
                        Ok(())
                    })
                    .ok();
            }
        }

        #[cfg(windows)]
        {
            std::process::Command::new(std::env::current_exe().unwrap())
                .arg(COMMAND)
                .arg(COMMAND_ARG)
                .arg(tempfile_path)
                .creation_flags(DETACHED_PROCESS)
                .spawn()?;
        }

        Ok(())
    }

    fn basic_properties(&self) -> serde_json::Value {
        let (program, subcommand, sanitized_args) = sanitize_command(&self.command);

        let mut properties = json!({
            "Subcommand": subcommand,
            "Command": format!("{} {} {}", program, subcommand, sanitized_args).trim(),
            "Duration MS": self.start_time.elapsed().as_millis(),
            "Repository": repository_identifier(self.repository_path.clone()),
            "Environment": Self::environment(),
            "CI": std::env::var("CI").is_ok().to_string(),
        });

        if cfg!(debug_assertions) {
            properties["Username"] = json!(whoami::username());
        }

        properties
    }

    fn environment() -> String {
        let aws_execution_env = std::env::var("AWS_EXECUTION_ENV");

        match aws_execution_env {
            Ok(value) => format!("AWS_EXECUTION_ENV={}", value),
            Err(_) => "UNKNOWN".to_string(),
        }
    }

    pub fn current_level() -> TelemetryLevel {
        if cfg!(debug_assertions) {
            debug!("Telemetry disabled on debug builds");
            TelemetryLevel::Off
        } else {
            if is_subcommand("telemetry") {
                debug!("Telemetry disabled for telemetry subcommand to avoid infinite loop");
                return TelemetryLevel::Off;
            }

            match std::env::var("QLTY_TELEMETRY") {
                Ok(value) => match value.as_str() {
                    "off" => {
                        debug!("Telemetry disabled by QLTY_TELEMETRY=off");
                        TelemetryLevel::Off
                    }
                    "errors" => {
                        debug!("Telemetry enabled for errors");
                        TelemetryLevel::Errors
                    }
                    "all" => {
                        debug!("Telemetry enabled for all");
                        TelemetryLevel::All
                    }
                    _ => {
                        warn!("Invalid value for QLTY_TELEMETRY: {}", value);
                        TelemetryLevel::All
                    }
                },
                Err(_) => TelemetryLevel::All,
            }
        }
    }
}

fn anonymous_id() -> Result<String> {
    Ok(format!(
        "{:x}",
        md5::compute(get_mac_address()?.unwrap().bytes())
    ))
}
