use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn pattern_show(id: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    match forge::PackManifest::load(&cwd, &id) {
        Some(pack) => {
            let pack_dir = forge::PackManifest::dir(&cwd, &id);
            let forge_files: Vec<String> = std::fs::read_dir(&pack_dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("forge"))
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .collect();
            ResponseEnvelope::success(
                "pattern show",
                serde_json::json!({
                    "id": pack.id,
                    "name": pack.name,
                    "version": pack.version,
                    "description": pack.description,
                    "framework": pack.framework,
                    "author": pack.author,
                    "tags": pack.tags,
                    "args": pack.args.iter().map(|a| serde_json::json!({
                        "name": a.name,
                        "type": a.arg_type,
                        "required": a.required,
                        "default": a.default,
                        "description": a.description,
                    })).collect::<Vec<_>>(),
                    "outputs": pack.outputs.iter().map(|o| serde_json::json!({
                        "id": o.id,
                        "template": o.template,
                        "path": o.path,
                    })).collect::<Vec<_>>(),
                    "commands": pack.commands.iter().map(|(k, c)| serde_json::json!({
                        "name": k,
                        "description": c.description,
                        "outputs": c.outputs,
                        "default": c.default,
                    })).collect::<Vec<_>>(),
                    "markers": pack.markers.len(),
                    "forge_files": forge_files,
                }),
                0,
            )
        }
        None => ResponseEnvelope::error(
            "pattern show",
            ErrorResponse::new(
                ErrorCode::UnknownCommand,
                format!("Pack '{}' not found in .tsx/patterns/", id),
            ),
            0,
        ),
    }
}
