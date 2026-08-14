use crate::json::response::ResponseEnvelope;

use super::utils::collect_builtin_pack_paths;

pub fn pattern_list(_verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let packs = forge::PackManifest::list(&cwd);

    let items: Vec<serde_json::Value> = packs
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "version": p.version,
                "description": p.description,
                "framework": p.framework,
                "commands": p.commands.keys().collect::<Vec<_>>(),
                "outputs": p.outputs.len(),
            })
        })
        .collect();

    ResponseEnvelope::success(
        "pattern list",
        serde_json::json!({
            "count": items.len(),
            "patterns": items,
        }),
        0,
    )
}

/// List pattern packs embedded in the binary.
pub fn pattern_list_builtin(_verbose: bool) -> ResponseEnvelope {
    let items: Vec<serde_json::Value> = collect_builtin_pack_paths()
        .iter()
        .map(|rel| {
            let name = rel.replace('\\', "/");
            serde_json::json!({ "id": name, "source": format!("builtin:{}", name) })
        })
        .collect();
    ResponseEnvelope::success(
        "pattern list",
        serde_json::json!({ "count": items.len(), "builtin": true, "patterns": items }),
        0,
    )
}
