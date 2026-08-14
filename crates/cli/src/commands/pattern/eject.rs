use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

/// Eject a pack — delete generated files and reverse marker injections.
pub fn pattern_eject(id: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let pack_dir = forge::PackManifest::dir(&cwd, &id);

    if !pack_dir.exists() {
        return ResponseEnvelope::error(
            "pattern eject",
            ErrorResponse::new(ErrorCode::ProjectNotFound, format!("Pack '{}' not found in .tsx/patterns/", id)),
            0,
        );
    }

    let generated_path = pack_dir.join(".generated");
    let generated_content = match std::fs::read_to_string(&generated_path) {
        Ok(s) => s,
        Err(_) => return ResponseEnvelope::error(
            "pattern eject",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                format!("No .generated manifest for pack '{}'. Run `tsx pattern run {}` first.", id, id),
            ),
            0,
        ),
    };

    let generated: serde_json::Value = match serde_json::from_str(&generated_content) {
        Ok(v) => v,
        Err(e) => return ResponseEnvelope::error(
            "pattern eject",
            ErrorResponse::new(ErrorCode::InternalError, format!("Corrupt .generated manifest: {e}")),
            0,
        ),
    };

    let mut deleted: Vec<String> = Vec::new();
    let mut markers_removed: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    // 1. Delete generated output files
    if let Some(files) = generated["files"].as_array() {
        for file in files {
            if let Some(rel) = file.as_str() {
                let abs = cwd.join(rel);
                if abs.exists() {
                    match std::fs::remove_file(&abs) {
                        Ok(_) => deleted.push(rel.to_string()),
                        Err(e) => errors.push(format!("Failed to delete {rel}: {e}")),
                    }
                }
            }
        }
    }

    // 2. Reverse marker injections (remove injected lines from target files)
    if let Some(markers) = generated["markers"].as_array() {
        for marker in markers {
            let file = match marker["file"].as_str() { Some(f) => f, None => continue };
            let line = match marker["line"].as_str() { Some(l) => l, None => continue };
            let abs = cwd.join(file);
            if !abs.exists() { continue; }
            match std::fs::read_to_string(&abs) {
                Ok(content) => {
                    let filtered: Vec<&str> = content.lines().filter(|l| l.trim() != line.trim()).collect();
                    let new_content = if content.ends_with('\n') {
                        format!("{}\n", filtered.join("\n"))
                    } else {
                        filtered.join("\n")
                    };
                    match std::fs::write(&abs, new_content) {
                        Ok(_) => markers_removed.push(format!("{}: {}", file, line)),
                        Err(e) => errors.push(format!("Failed to update {file}: {e}")),
                    }
                }
                Err(e) => errors.push(format!("Failed to read {file}: {e}")),
            }
        }
    }

    let _ = std::fs::remove_file(&generated_path);

    if errors.is_empty() {
        ResponseEnvelope::success(
            "pattern eject",
            serde_json::json!({
                "id": id,
                "files_deleted": deleted,
                "markers_removed": markers_removed,
            }),
            0,
        )
    } else {
        ResponseEnvelope::error(
            "pattern eject",
            ErrorResponse::new(ErrorCode::InternalError, errors.join("\n")),
            0,
        )
    }
}
