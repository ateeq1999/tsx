use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_schema(name: String, command: String, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    match forge::template_schema(&name, &command) {
        None => ResponseEnvelope::error(
            "template:schema",
            ErrorResponse::new(
                ErrorCode::TemplateNotFound,
                format!("No schema found for template '{}' command '{}'", name, command),
            ),
            start.elapsed().as_millis() as u64,
        ),
        Some(schema) => {
            ResponseEnvelope::success("template:schema", schema, start.elapsed().as_millis() as u64)
        }
    }
}
