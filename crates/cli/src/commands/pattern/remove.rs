use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::types::PatternDefinition;

pub fn pattern_remove(id: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let dir = PatternDefinition::dir(&cwd, &id);

    if !dir.exists() {
        return ResponseEnvelope::error(
            "pattern remove",
            ErrorResponse::new(
                ErrorCode::UnknownCommand,
                format!("Pattern '{}' not found in .tsx/patterns/", id),
            ),
            0,
        );
    }

    match std::fs::remove_dir_all(&dir) {
        Ok(_) => ResponseEnvelope::success(
            "pattern remove",
            serde_json::json!({ "removed": id }),
            0,
        ),
        Err(e) => ResponseEnvelope::error(
            "pattern remove",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}
