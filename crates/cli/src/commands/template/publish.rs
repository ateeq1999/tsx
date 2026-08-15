use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_publish(
    name: String,
    version: String,
    path: Option<String>,
    _verbose: bool,
) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let cfg = forge::load_global_config();
    let token = cfg.preferred_templates.get("__registry_token__").cloned();
    let registry_url = cfg
        .registry_url
        .unwrap_or_else(|| "https://registry.tsx.dev".to_string());

    if token.is_none() {
        return ResponseEnvelope::error(
            "template:publish",
            ErrorResponse::new(
                ErrorCode::Unauthorized,
                "Not logged in. Run `tsx template login --token <TOKEN>` first.",
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    let src = path.map(std::path::PathBuf::from).unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    });

    if !src.join("manifest.json").exists() {
        return ResponseEnvelope::error(
            "template:publish",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                format!("No manifest.json found in {}", src.display()),
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    // Stub: publishing is not yet implemented server-side.
    let data = serde_json::json!({
        "id": name,
        "version": version,
        "registry": registry_url,
        "status": "pending",
        "message": "Registry publish is not yet available. Check back in a future release.",
    });
    ResponseEnvelope::success("template:publish", data, start.elapsed().as_millis() as u64)
}
