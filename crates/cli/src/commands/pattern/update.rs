use std::path::PathBuf;

use crate::json::response::ResponseEnvelope;

use super::install::{pattern_install_github, pattern_install_local, pattern_install_registry};

/// Update installed packs from their original source.
pub fn pattern_update(id: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let ids: Vec<String> = if let Some(specific) = id {
        vec![specific]
    } else {
        forge::PackManifest::list(&cwd).into_iter().map(|p| p.id).collect()
    };

    if ids.is_empty() {
        return ResponseEnvelope::success(
            "pattern update",
            serde_json::json!({ "message": "No packs installed. Run `tsx pattern install` first." }),
            0,
        );
    }

    let mut results: Vec<serde_json::Value> = Vec::new();

    for id in ids {
        let Some(source_meta) = forge::PackSource::load(&cwd, &id) else {
            results.push(serde_json::json!({
                "id": id, "status": "skipped", "reason": "no .source.json (manually placed pack)"
            }));
            continue;
        };

        let current_version = forge::PackManifest::load(&cwd, &id)
            .map(|p| p.version)
            .unwrap_or_default();

        let resp = match source_meta.kind.as_str() {
            "local" => pattern_install_local(PathBuf::from(&source_meta.source), Some(id.clone()), &cwd),
            "github" => pattern_install_github(&source_meta.source, Some(id.clone()), &cwd),
            "registry" => {
                // Strip pinned @version to fetch latest
                let slug = source_meta.source.trim_start_matches('@');
                let base = if let Some(at) = slug.rfind('@') { &slug[..at] } else { slug };
                pattern_install_registry(&format!("@{}", base), Some(id.clone()), &cwd)
            }
            _ => {
                results.push(serde_json::json!({
                    "id": id, "status": "skipped",
                    "reason": format!("unknown source kind: {}", source_meta.kind)
                }));
                continue;
            }
        };

        if resp.success {
            let new_version = forge::PackManifest::load(&cwd, &id)
                .map(|p| p.version)
                .unwrap_or_default();
            if !current_version.is_empty() && new_version != current_version {
                results.push(serde_json::json!({
                    "id": id, "status": "updated",
                    "from": current_version, "to": new_version,
                }));
            } else {
                results.push(serde_json::json!({
                    "id": id, "status": "up-to-date", "version": new_version,
                }));
            }
        } else {
            results.push(serde_json::json!({
                "id": id, "status": "error",
                "error": resp.error.as_ref().map(|e| e.message.as_str()).unwrap_or("unknown"),
            }));
        }
    }

    ResponseEnvelope::success(
        "pattern update",
        serde_json::json!({ "results": results }),
        0,
    )
}
