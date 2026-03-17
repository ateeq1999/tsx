use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCommand {
    pub command: String,
    #[serde(default)]
    pub options: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPayload {
    pub commands: Vec<BatchCommand>,
    #[serde(default)]
    pub stop_on_failure: bool,
    /// When true (and stop_on_failure is true), delete all files created by
    /// earlier commands if a later command fails, leaving the project clean.
    #[serde(default)]
    pub rollback_on_failure: bool,
}

impl Default for BatchPayload {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            stop_on_failure: false,
            rollback_on_failure: false,
        }
    }
}
