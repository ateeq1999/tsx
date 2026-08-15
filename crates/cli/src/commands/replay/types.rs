use serde::{Deserialize, Serialize};

pub(super) const LOCK_FILE: &str = ".tsx/replay-recording.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySession {
    pub tsx_version: String,
    pub framework: String,
    pub recorded_at: String,
    pub steps: Vec<ReplayStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayStep {
    pub command: String,
    pub args: serde_json::Value,
    pub outputs: Vec<String>,
}

/// Lock file written while a recording session is in progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct RecordingLock {
    pub out: String,
    pub started_at: String,
}
