use std::path::PathBuf;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::types::SnapshotFixture;

pub fn snapshot_add(
    generator: String,
    fixture: String,
    input: Option<String>,
    _verbose: bool,
) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let input_json: serde_json::Value = match input {
        Some(s) => match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(e) => {
                return ResponseEnvelope::error(
                    "snapshot add",
                    ErrorResponse::new(
                        ErrorCode::ValidationError,
                        format!("Invalid JSON input: {}", e),
                    ),
                    0,
                )
            }
        },
        None => serde_json::json!({"name": fixture}),
    };

    match SnapshotFixture::add(&cwd, &generator, &fixture, &input_json) {
        Ok(_) => ResponseEnvelope::success(
            "snapshot add",
            serde_json::json!({
                "generator": generator,
                "fixture": fixture,
                "path": SnapshotFixture::fixture_path(&cwd, &generator, &fixture).to_string_lossy(),
            }),
            0,
        )
        .with_next_steps(vec![
            format!("Run `tsx snapshot update --generator {}` to capture the expected output", generator),
        ]),
        Err(e) => ResponseEnvelope::error(
            "snapshot add",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}
