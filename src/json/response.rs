use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::json::error::ErrorResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub timestamp: String,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl Metadata {
    pub fn new(duration_ms: u64) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| chrono_lite_timestamp(d.as_secs()))
                .unwrap_or_else(|_| "2026-01-01T00:00:00Z".to_string()),
            duration_ms,
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}

fn chrono_lite_timestamp(secs: u64) -> String {
    const SECS_PER_YEAR: u64 = 365 * 24 * 60 * 60;
    const SECS_PER_DAY: u64 = 24 * 60 * 60;

    let mut remaining = secs;
    let years = remaining / SECS_PER_YEAR;
    remaining %= SECS_PER_YEAR;
    let days = remaining / SECS_PER_DAY;
    remaining %= SECS_PER_DAY;
    let hours = remaining / 3600;
    remaining %= 3600;
    let minutes = remaining / 60;
    let seconds = remaining % 60;

    format!(
        "2026-01-{:02}T{:02}:{:02}:{:02}Z",
        days + 1,
        hours,
        minutes,
        seconds
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub project_root: String,
    pub tsx_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub success: bool,
    pub version: String,
    pub command: String,
    pub result: serde_json::Value,
    pub metadata: Metadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub next_steps: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
}

impl ResponseEnvelope {
    pub fn success(
        command: impl Into<String>,
        result: serde_json::Value,
        duration_ms: u64,
    ) -> Self {
        Self {
            success: true,
            version: "1.0".to_string(),
            command: command.into(),
            result,
            metadata: Metadata::new(duration_ms),
            context: None,
            next_steps: Vec::new(),
            error: None,
            dry_run: None,
        }
    }

    pub fn error(command: impl Into<String>, error: ErrorResponse, duration_ms: u64) -> Self {
        Self {
            success: false,
            version: "1.0".to_string(),
            command: command.into(),
            result: serde_json::Value::Null,
            metadata: Metadata::new(duration_ms),
            context: None,
            next_steps: Vec::new(),
            error: Some(error),
            dry_run: None,
        }
    }

    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_next_steps(mut self, steps: Vec<String>) -> Self {
        self.next_steps = steps;
        self
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_warning(warning);
        self
    }

    pub fn with_dry_run(mut self, is_dry_run: bool) -> Self {
        self.dry_run = Some(is_dry_run);
        self
    }

    pub fn print(&self) {
        println!("{}", serde_json::to_string_pretty(self).unwrap_or_default());
    }
}
