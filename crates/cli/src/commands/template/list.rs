use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_list(source: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let templates = match source.as_deref() {
        Some("global") => forge::discover_from_source(forge::TemplateSource::Global),
        Some("project") => forge::discover_from_source(forge::TemplateSource::Project),
        Some("framework") => forge::discover_from_source(forge::TemplateSource::Framework),
        Some(other) => {
            return ResponseEnvelope::error(
                "template:list",
                ErrorResponse::new(
                    ErrorCode::ValidationError,
                    format!("Unknown source '{}'. Use: global, project, or framework", other),
                ),
                0,
            );
        }
        None => forge::discover_templates(),
    };

    let items: Vec<serde_json::Value> = templates
        .iter()
        .map(|t| {
            serde_json::json!({
                "id": t.id,
                "name": t.name,
                "version": t.version,
                "description": t.description,
                "source": t.source.to_string(),
                "path": t.path.to_string_lossy(),
            })
        })
        .collect();

    let data = serde_json::json!({
        "count": items.len(),
        "templates": items,
    });

    ResponseEnvelope::success("template:list", data, start.elapsed().as_millis() as u64)
}
