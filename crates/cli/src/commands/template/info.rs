use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_info(name: String, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    match forge::find_template(&name) {
        None => ResponseEnvelope::error(
            "template:info",
            ErrorResponse::new(
                ErrorCode::TemplateNotFound,
                format!("Template '{}' not found. Run `tsx template list` to see available templates.", name),
            ),
            start.elapsed().as_millis() as u64,
        ),
        Some(info) => {
            let data = serde_json::json!({
                "id": info.id,
                "name": info.name,
                "version": info.version,
                "description": info.description,
                "source": info.source.to_string(),
                "path": info.path.to_string_lossy(),
                "manifest": info.manifest,
            });
            ResponseEnvelope::success("template:info", data, start.elapsed().as_millis() as u64)
        }
    }
}
