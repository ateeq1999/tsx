use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_init(name: String, dest: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let dest_path = match dest {
        Some(d) => std::path::PathBuf::from(d),
        None => std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(&name),
    };

    match forge::init_template(&name, &dest_path) {
        Ok(()) => {
            let data = serde_json::json!({
                "id": name,
                "path": dest_path.to_string_lossy(),
                "files_created": ["manifest.json", "README.md"],
            });
            ResponseEnvelope::success("template:init", data, start.elapsed().as_millis() as u64)
        }
        Err(e) => ResponseEnvelope::error(
            "template:init",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            start.elapsed().as_millis() as u64,
        ),
    }
}
