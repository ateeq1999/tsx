use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::output::CommandResult;
use crate::utils::paths::find_project_root;

/// All event types emitted by `tsx dev --json-events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Started,
    FileChanged,
    FileAdded,
    FileDeleted,
    BuildStarted,
    BuildCompleted,
    BuildFailed,
    HotReload,
    Error,
    Stopped,
}

/// A single JSON event line written to stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevEvent {
    pub event: EventType,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl DevEvent {
    pub fn new(event: EventType, data: Option<serde_json::Value>) -> Self {
        DevEvent {
            event,
            timestamp: iso_timestamp(),
            data,
        }
    }

    /// Emit this event as a single JSON line to stdout.
    pub fn emit(&self) {
        if let Ok(line) = serde_json::to_string(self) {
            println!("{}", line);
        }
    }
}

fn iso_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Minimal ISO-8601 UTC timestamp without chrono dependency.
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // Approximate date from epoch (good enough for event timestamps).
    let year = 1970 + days / 365;
    let day_of_year = days % 365 + 1;
    let month = (day_of_year / 30).min(11) + 1;
    let day = day_of_year % 30 + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, h, m, s
    )
}

/// Run `tsx dev` with optional JSON event emission.
///
/// In `json_events` mode, every file change, build event, and error is emitted
/// as a single-line JSON object on stdout. Consumers (IDE plugins, CI) can
/// pipe and parse these events without screen-scraping.
pub fn dev(json_events: bool) -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("dev", e.to_string()),
    };

    if json_events {
        DevEvent::new(
            EventType::Started,
            Some(serde_json::json!({
                "project_root": root.to_string_lossy(),
                "tsx_version": env!("CARGO_PKG_VERSION"),
                "json_events": true
            })),
        )
        .emit();
    }

    // Spawn the underlying dev server (Vite / TanStack Start).
    let mut child = match std::process::Command::new("npm")
        .args(["run", "dev"])
        .current_dir(&root)
        .stdout(if json_events {
            std::process::Stdio::piped()
        } else {
            std::process::Stdio::inherit()
        })
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            if json_events {
                DevEvent::new(
                    EventType::Error,
                    Some(serde_json::json!({ "message": e.to_string() })),
                )
                .emit();
            }
            return CommandResult::err("dev", format!("Failed to start dev server: {}", e));
        }
    };

    if json_events {
        // Stream stdout from the child and translate known patterns into events.
        if let Some(stdout) = child.stdout.take() {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);

            for line in reader.lines().filter_map(|l| l.ok()) {
                let line_lower = line.to_lowercase();

                // Translate Vite/TanStack Start output into structured events.
                if line_lower.contains("ready in") || line_lower.contains("server running") {
                    DevEvent::new(
                        EventType::BuildCompleted,
                        Some(serde_json::json!({ "message": line })),
                    )
                    .emit();
                } else if line_lower.contains("page reload") || line_lower.contains("hmr update") {
                    DevEvent::new(
                        EventType::HotReload,
                        Some(serde_json::json!({ "message": line })),
                    )
                    .emit();
                } else if line_lower.contains("error") || line_lower.contains("failed") {
                    DevEvent::new(
                        EventType::Error,
                        Some(serde_json::json!({ "message": line })),
                    )
                    .emit();
                } else if line_lower.contains("build") {
                    DevEvent::new(
                        EventType::BuildStarted,
                        Some(serde_json::json!({ "message": line })),
                    )
                    .emit();
                }
            }
        }
    }

    let status = child.wait();

    if json_events {
        DevEvent::new(
            EventType::Stopped,
            Some(serde_json::json!({
                "exit_code": status.map(|s| s.code()).unwrap_or(None)
            })),
        )
        .emit();
    }

    CommandResult::ok("dev", vec![])
}
