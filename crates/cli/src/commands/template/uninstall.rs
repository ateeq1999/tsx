use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_uninstall(name: String, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    match forge::uninstall(&name) {
        Ok(()) => {
            let data = serde_json::json!({ "uninstalled": name });
            ResponseEnvelope::success("template:uninstall", data, start.elapsed().as_millis() as u64)
        }
        Err(e) => ResponseEnvelope::error(
            "template:uninstall",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            start.elapsed().as_millis() as u64,
        ),
    }
}
