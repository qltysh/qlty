pub mod auth;
mod build;
pub mod cache;
mod check;
mod completions;
pub mod config;
pub mod coverage;
mod dashboard;
mod deinit;
mod discord;
mod docs;
mod fmt;
mod git_index;
pub mod githooks;
mod init;
mod install;
mod metrics;
mod panic;
mod parse;
mod patch;
pub mod plugins;
mod smells;
mod telemetry;
mod upgrade;
mod validate;
mod version;

pub use {
    build::Build, check::Check, completions::Completions, dashboard::Dashboard, deinit::Deinit,
    discord::Discord, docs::Docs, fmt::Fmt, git_index::GitIndex, init::Init, install::Install,
    metrics::Metrics, panic::Panic, parse::Parse, patch::Patch, smells::Smells,
    telemetry::Telemetry, upgrade::Upgrade, validate::Validate, version::Version,
};
