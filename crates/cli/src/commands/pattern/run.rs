use std::collections::HashMap;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::utils::chrono_now;

/// Run a pack command — render templates, inject markers, run post-hooks.
pub fn pattern_run(
    id: String,
    command: Option<String>,
    arg_pairs: Vec<String>, // "key=value" pairs
    dry_run: bool,
    overwrite: bool,
    diff: bool,
    _verbose: bool,
) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let Some(pack) = forge::PackManifest::load(&cwd, &id) else {
        return ResponseEnvelope::error(
            "pattern run",
            ErrorResponse::new(ErrorCode::ProjectNotFound, format!("Pack '{}' not found in .tsx/patterns/", id)),
            0,
        );
    };

    let pack_dir = forge::PackManifest::dir(&cwd, &id);

    let mut args = HashMap::new();
    for pair in &arg_pairs {
        if let Some(eq) = pair.find('=') {
            let key = pair[..eq].trim().to_string();
            let val = pair[eq + 1..].to_string();
            args.insert(key, serde_json::Value::String(val));
        }
    }

    let opts = forge::RunOpts { dry_run, overwrite, command, diff };

    match forge::run_pack(&pack, &pack_dir, args, &cwd, &opts) {
        Ok(result) => {
            // Write .generated manifest for `tsx pattern eject` support
            if !dry_run && !result.files_written.is_empty() {
                let generated = serde_json::json!({
                    "pack_id": id,
                    "generated_at": chrono_now(),
                    "files": result.files_written.iter()
                        .map(|p| p.strip_prefix(&cwd).unwrap_or(p).to_string_lossy().replace('\\', "/"))
                        .collect::<Vec<_>>(),
                    "markers": result.markers_injected.iter().map(|(p, line)| serde_json::json!({
                        "file": p.strip_prefix(&cwd).unwrap_or(p).to_string_lossy().replace('\\', "/"),
                        "line": line,
                    })).collect::<Vec<_>>(),
                });
                let _ = std::fs::write(
                    pack_dir.join(".generated"),
                    serde_json::to_string_pretty(&generated).unwrap_or_default(),
                );
            }
            ResponseEnvelope::success(
                "pattern run",
                serde_json::json!({
                    "dry_run": dry_run,
                    "diff": diff,
                    "files_written": result.files_written.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                    "files_skipped": result.files_skipped.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                    "markers_injected": result.markers_injected.iter().map(|(p, l)| serde_json::json!({
                        "file": p.to_string_lossy(), "line": l,
                    })).collect::<Vec<_>>(),
                    "hooks_run": result.hooks_run,
                    "diffs": result.diffs.iter().map(|(p, d)| serde_json::json!({
                        "file": p.to_string_lossy(), "diff": d,
                    })).collect::<Vec<_>>(),
                }),
                0,
            )
        },
        Err(e) => ResponseEnvelope::error(
            "pattern run",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}
