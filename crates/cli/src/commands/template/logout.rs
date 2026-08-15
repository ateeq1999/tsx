use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_logout(_verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let mut cfg = forge::load_global_config();
    cfg.preferred_templates.remove("__registry_token__");

    match forge::save_global_config(&cfg) {
        Ok(()) => {
            let data = serde_json::json!({ "authenticated": false });
            ResponseEnvelope::success("template:logout", data, start.elapsed().as_millis() as u64)
        }
        Err(e) => ResponseEnvelope::error(
            "template:logout",
            ErrorResponse::new(ErrorCode::InternalError, e),
            start.elapsed().as_millis() as u64,
        ),
    }
}
