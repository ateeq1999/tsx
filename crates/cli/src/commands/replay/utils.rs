//! Helpers shared by `replay record` and `replay run`.

use std::path::Path;

use super::types::ReplayStep;

pub(super) fn execute_step(
    step: &ReplayStep,
    dry_run: bool,
    _root: &Path,
) -> Result<Vec<String>, String> {
    use crate::commands::batch;

    let result = batch::execute_command_pub(
        &step.command,
        &step.args,
        false, // overwrite: false by default on replay
        dry_run,
    );
    result.map_err(|(_, msg)| msg)
}

pub(super) fn load_history_steps(history_path: &Path) -> Vec<ReplayStep> {
    if !history_path.exists() {
        return Vec::new();
    }
    let content = std::fs::read_to_string(history_path).unwrap_or_default();
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<ReplayStep>(line).ok())
        .collect()
}

pub(super) fn detect_framework(root: &Path) -> String {
    let pkg = root.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&pkg) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                if deps.contains_key("@tanstack/start") || deps.contains_key("@tanstack/react-start") {
                    return "tanstack-start".to_string();
                }
                if deps.contains_key("next") {
                    return "next".to_string();
                }
                if deps.contains_key("remix") || deps.contains_key("@remix-run/react") {
                    return "remix".to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

pub(super) fn current_timestamp_slug() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // YYYYMMDD-HHMMSS approximation
    let days = secs / 86400;
    let time = secs % 86400;
    let h = time / 3600;
    let m = (time % 3600) / 60;
    let s = time % 60;
    format!("{}-{:02}{:02}{:02}", days, h, m, s)
}

pub(super) fn current_timestamp_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;
    // Approximate: count from 1970-01-01
    format!("1970-01-{:02}T{:02}:{:02}:{:02}Z", (days_since_epoch % 365) + 1, h, m, s)
}
