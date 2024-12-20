use crate::allocator::ALLOCATOR;
use crate::arguments::is_subcommand;
use bytesize::ByteSize;
use chrono::{SecondsFormat, Utc};
use console::{style, StyledObject};
use qlty_config::Library;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use supports_color::Stream;
use tracing::{debug, Subscriber};
use tracing_appender::non_blocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::{FormatEvent, FormatFields, FormattedFields};
use tracing_subscriber::registry;
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt::{self},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    Layer,
};

#[derive(Debug)]
struct LogFormatter;

// Pulled and modified from tracing-subscriber documentation:
// https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/trait.FormatEvent.html#examples
impl<S, N> FormatEvent<S, N> for LogFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        fn ansi<D>(writer: &fmt::format::Writer<'_>, object: D) -> StyledObject<D> {
            style(object).force_styling(writer.has_ansi_escapes())
        }

        // Format values from the event's metadata:
        let metadata = event.metadata();
        let time = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        let styled_level = ansi(&writer, metadata.level());
        let styled_level = match *metadata.level() {
            tracing::Level::TRACE => styled_level.magenta(),
            tracing::Level::DEBUG => styled_level.blue(),
            tracing::Level::INFO => styled_level.green(),
            tracing::Level::WARN => styled_level.yellow(),
            tracing::Level::ERROR => styled_level.red(),
        };
        let styled_time = ansi(&writer, time).dim();

        // ThreadIdsare formatted as ThreadId(1) in current Rust, which is verbose
        let thread_id = format!("{:?}", thread::current().id());
        let tid = thread_id
            .trim_start_matches("ThreadId(")
            .trim_end_matches(')');

        let styled_info = ansi(
            &writer,
            format!(
                "[T{}] {} ({}):",
                tid,
                metadata.module_path().unwrap_or_default(),
                ByteSize(ALLOCATOR.allocated() as u64),
            ),
        )
        .dim();
        write!(
            &mut writer,
            "{} {} {} ",
            styled_time, styled_level, styled_info
        )?;

        // Format all the spans in the event's span context.
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                write!(writer, "{}", span.name())?;

                // `FormattedFields` is a formatted representation of the span's
                // fields, which is stored in its extensions by the `fmt` layer's
                // `new_span` method. The fields will have been formatted
                // by the same field formatter that's provided to the event
                // formatter in the `FmtContext`.
                let ext = span.extensions();
                let fields = &ext
                    .get::<FormattedFields<N>>()
                    .expect("will never be `None`");

                // Skip formatting the fields if the span had no fields.
                if !fields.is_empty() {
                    write!(writer, "{{{}}}", fields)?;
                }
                write!(writer, ": ")?;
            }
        }

        // Write fields on the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

#[derive(Debug)]
pub enum LogDestination {
    Stdout,
    Stderr,
    Directory(PathBuf),
}

pub fn init_logs(repository_path: Option<PathBuf>) -> Vec<WorkerGuard> {
    let mut guards = vec![];
    let mut layers = vec![];
    let stdout_ansi = supports_color::on(Stream::Stdout).is_some();
    let stderr_ansi = supports_color::on(Stream::Stderr).is_some();

    add_file_log_layer(repository_path.clone(), &mut layers, &mut guards);

    for (env, destination, ansi) in [
        ("QLTY_LOG_STDERR", &LogDestination::Stderr, stderr_ansi),
        ("QLTY_LOG_STDOUT", &LogDestination::Stdout, stdout_ansi),
    ] {
        if std::env::var(env).is_ok() || is_telemetry() {
            add_layer(destination, ansi, &mut layers, &mut guards);
            break;
        }
    }

    registry()
        .with(layers)
        .try_init()
        .expect("Could not set global default logger");
    guards
}

fn is_telemetry() -> bool {
    is_subcommand("telemetry")
}

fn add_file_log_layer<S>(
    mut repository_path: Option<PathBuf>,
    layers: &mut Vec<Box<dyn Layer<S> + Send + Sync + 'static>>,
    guards: &mut Vec<WorkerGuard>,
) where
    S: Subscriber,
    for<'a> S: LookupSpan<'a>,
{
    if is_telemetry() {
        return;
    }
    if repository_path.is_none() {
        repository_path = Some(PathBuf::from(std::env::var("HOME").unwrap_or_default()));
    }
    let log_dir = logs_dir(repository_path).unwrap_or_default();
    add_layer(&LogDestination::Directory(log_dir), false, layers, guards);
}

fn add_layer<S>(
    destination: &LogDestination,
    ansi: bool,
    layers: &mut Vec<Box<dyn Layer<S> + Send + Sync + 'static>>,
    guards: &mut Vec<WorkerGuard>,
) where
    S: Subscriber,
    for<'a> S: LookupSpan<'a>,
{
    let formatter = LogFormatter {};
    let (writer, guard) = non_blocking(log_writer(destination));
    guards.push(guard);
    layers.push(
        fmt::layer()
            .event_format(formatter)
            .with_writer(writer)
            .with_ansi(ansi)
            .with_filter(env_filter())
            .boxed(),
    );
}

fn log_writer(destination: &LogDestination) -> Box<dyn Write + Send + Sync> {
    match destination {
        LogDestination::Stdout => Box::new(std::io::stdout()),
        LogDestination::Stderr => Box::new(std::io::stderr()),
        LogDestination::Directory(log_dir) => {
            if log_dir.exists() {
                Box::new(tracing_appender::rolling::daily(log_dir, "qlty-cli"))
            } else {
                match Library::global_logs_root() {
                    Ok(global_logs_dir) => {
                        if std::fs::create_dir_all(&global_logs_dir).is_ok() {
                            Box::new(tracing_appender::rolling::daily(
                                global_logs_dir,
                                "qlty-cli",
                            ))
                        } else {
                            eprintln!("Could not create logs directory: {:?}", global_logs_dir);
                            Box::new(std::io::stderr())
                        }
                    }
                    Err(e) => {
                        eprintln!("Could not determine logs directory: {:?}", e);
                        Box::new(std::io::stderr())
                    }
                }
            }
        }
    }
}

pub fn logs_dir(repository_path: Option<PathBuf>) -> Option<PathBuf> {
    let mut directory = repository_path.clone();

    match directory {
        Some(ref mut path) => {
            path.push(".qlty");
            path.push("logs");
            Some(path.to_owned())
        }
        None => {
            debug!("No repository path, unknown logs dir");
            None
        }
    }
}

fn env_filter() -> EnvFilter {
    let args = std::env::args().collect::<Vec<String>>();

    if let Ok(log_env) = std::env::var("QLTY_LOG") {
        EnvFilter::builder()
            .with_default_directive(default_log_level().into())
            .parse_lossy(log_env)
    } else if args.contains(&"--debug".to_string()) {
        EnvFilter::builder()
            .with_default_directive(default_log_level().into())
            .parse_lossy("qlty=debug")
    } else {
        EnvFilter::builder()
            .with_default_directive(default_log_level().into())
            .from_env_lossy()
    }
}

fn default_log_level() -> LevelFilter {
    if is_subcommand("telemetry") {
        // Reduce noise when running telemetry
        LevelFilter::ERROR
    } else {
        LevelFilter::INFO
    }
}
