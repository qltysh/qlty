// Portions of this code are from https://github.com/meilisearch/segment
// MIT License: https://github.com/meilisearch/segment/blob/main/LICENSE
use crate::logging::logs_dir;
use crate::telemetry::locale::current_locale;
use crate::version::BUILD_IDENTIFIER;
use anyhow::{anyhow, Context, Result};
use base64::Engine as _;
use qlty_analysis::version::QLTY_VERSION;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::{Map, Value};
use std::io::Write;
use std::{fs::OpenOptions, path::PathBuf};
use time::OffsetDateTime;
use tracing::debug;

const WRITE_KEY: Option<&str> = option_env!("CIO_WRITE_KEY");
const TRACK_URL: &str = "https://cdp.customer.io/v1/track";

#[derive(Clone)]
pub struct SegmentClient {
    repository_path: Option<PathBuf>,
}

impl SegmentClient {
    pub fn new(repository_path: Option<PathBuf>) -> Result<Self> {
        Ok(Self { repository_path })
    }

    pub fn send_track(&self, track: Track) -> Result<()> {
        let write_key = WRITE_KEY.unwrap_or_default();
        if write_key.is_empty() {
            // ignore telemetry if no write key is set
            debug!("No write key set, skipping telemetry");
            return Ok(());
        }

        let message = Message::from(track);

        if let Err(error) = self.log(&message) {
            debug!("Could not log telemetry event: {}", error);
        }

        let http_basic_authorization = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD
                .encode(&format!("{}:", WRITE_KEY.unwrap_or_default()))
        );

        debug!(
            "POST {} with Authorization: {}: {:?}",
            TRACK_URL, http_basic_authorization, message
        );

        ureq::post(TRACK_URL)
            .set("Authorization", &http_basic_authorization)
            .send_json(serde_json::to_value(message)?)
            .map(|_| ())
            .with_context(|| "Failed to send telemetry event to Segment")
    }

    fn log(&self, message: &Message) -> Result<()> {
        let log_line = serde_json::to_string(&message)?;

        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_path()?)?;
        log_file.write_all((log_line + "\n").as_bytes())?;

        Ok(())
    }

    fn log_path(&self) -> Result<PathBuf> {
        if let Some(logs_dir) = logs_dir(self.repository_path.clone()) {
            let log_file_basename = "telemetry";
            let date_suffix = chrono::Utc::now().format("%Y-%m-%d");
            let log_filename = format!("{}-{}", log_file_basename, date_suffix);
            Ok(logs_dir.join(log_filename))
        } else {
            Err(anyhow!("Could not determine logs directory"))
        }
    }
}

pub fn segment_user(user_id: Option<String>, anonymous_id: String) -> User {
    match user_id {
        Some(user_id) => User::Both {
            anonymous_id: anonymous_id.clone(),
            user_id: user_id.clone(),
        },
        None => User::AnonymousId {
            anonymous_id: anonymous_id.clone(),
        },
    }
}

pub fn segment_context() -> serde_json::Value {
    json!({
        "locale": current_locale(),
        "os": {
            "name": std::env::consts::OS,
        },
        "device": {
            "type": std::env::consts::ARCH,
        },
        "app": {
            "name": "qlty",
            "version": QLTY_VERSION,
            "build": BUILD_IDENTIFIER.as_str()
        },
    })
}

/// An enum containing all values which may be sent to Segment's tracking API.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Track(Track),
    Batch(Batch),
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Default)]
pub struct Batch {
    /// The batch of messages to send.
    pub batch: Vec<BatchMessage>,

    /// Context associated with this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,

    /// Integrations to route this message to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrations: Option<Value>,

    /// Extra fields to put at the top level of this message.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// An enum containing all messages which may be placed inside a batch.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BatchMessage {
    #[serde(rename = "track")]
    Track(Track),
}

/// User ID information.
///
/// All Segment tracking API calls require a user ID, an anonymous ID, or both.
/// See [Segment's
/// documentation](https://segment.com/docs/spec/identify/#identities) for how
/// user IDs and anonymous IDs should be used.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum User {
    /// The user is identified only by a user ID.
    UserId {
        #[serde(rename = "userId")]
        user_id: String,
    },

    /// The user is identified only by an anonymous ID.
    AnonymousId {
        #[serde(rename = "anonymousId")]
        anonymous_id: String,
    },

    /// The user is identified by both a user ID and an anonymous ID.
    Both {
        #[serde(rename = "userId")]
        user_id: String,

        #[serde(rename = "anonymousId")]
        anonymous_id: String,
    },
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// The user associated with this message.
    #[serde(flatten)]
    pub user: User,

    /// The name of the event being tracked.
    pub event: String,

    /// The properties associated with the event.
    pub properties: Value,

    /// The timestamp associated with this message.
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub timestamp: Option<OffsetDateTime>,

    /// Context associated with this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,

    /// Integrations to route this message to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrations: Option<Value>,

    /// Extra fields to put at the top level of this message.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

macro_rules! into {
    (from $from:ident into $for:ident) => {
        impl From<$from> for $for {
            fn from(message: $from) -> Self {
                Self::$from(message)
            }
        }
    };
    ($(from $from:ident into $for:ident),+ $(,)?) => {
        $(
            into!{from $from into $for}
        )+
    };
}

into! {
    from Track into Message,
    from Batch into Message,
    from Track into BatchMessage,
}
