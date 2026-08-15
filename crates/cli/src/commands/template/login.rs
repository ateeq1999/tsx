use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_login(token: String, registry: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let registry_url = registry
        .unwrap_or_else(|| "https://registry.tsx.dev".to_string());

    // Store the token in global config under the registry_url key.
    let mut cfg = forge::load_global_config();
    cfg.registry_url = Some(registry_url.clone());
    // Token stored as a preferred_template entry so we don't need a new field.
    cfg.preferred_templates.insert("__registry_token__".to_string(), token);

    match forge::save_global_config(&cfg) {
        Ok(()) => {
            let data = serde_json::json!({
                "registry": registry_url,
                "authenticated": true,
            });
            ResponseEnvelope::success("template:login", data, start.elapsed().as_millis() as u64)
        }
        Err(e) => ResponseEnvelope::error(
            "template:login",
            ErrorResponse::new(ErrorCode::InternalError, e),
            start.elapsed().as_millis() as u64,
        ),
    }
}
