//! Structured JSON events emitted by the watcher.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Started,
    Changed,
    Error,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEvent {
    pub event: EventKind,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub roots: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub timestamp: String,
}

impl WatchEvent {
    pub fn started(roots: &[String]) -> Self {
        Self {
            event: EventKind::Started,
            paths: Vec::new(),
            roots: roots.to_vec(),
            error: None,
            timestamp: now_iso(),
        }
    }

    pub fn changed(paths: &[PathBuf]) -> Self {
        Self {
            event: EventKind::Changed,
            paths: paths.iter().map(|p| p.to_string_lossy().to_string()).collect(),
            roots: Vec::new(),
            error: None,
            timestamp: now_iso(),
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            event: EventKind::Error,
            paths: Vec::new(),
            roots: Vec::new(),
            error: Some(msg.to_string()),
            timestamp: now_iso(),
        }
    }

    pub fn stopped() -> Self {
        Self {
            event: EventKind::Stopped,
            paths: Vec::new(),
            roots: Vec::new(),
            error: None,
            timestamp: now_iso(),
        }
    }
}

fn now_iso() -> String {
    // Simple ISO 8601 approximation without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Format as epoch seconds (agents can parse this)
    format!("{}", secs)
}
