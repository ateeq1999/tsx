use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_install(source: String, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let result = if source.starts_with("http://") || source.starts_with("https://") {
        forge::install_from_url(&source)
    } else if let Some(repo) = source.strip_prefix("github:") {
        forge::install_from_github(repo)
    } else {
        let src_path = std::path::Path::new(&source);
        if src_path.exists() {
            forge::install_from_dir(src_path)
        } else {
            // Treat as an npm package name
            forge::install_from_npm(&source)
        }
    };

    match result {
        Ok(info) => {
            let data = serde_json::json!({
                "installed": {
                    "id": info.id,
                    "name": info.name,
                    "version": info.version,
                    "path": info.path.to_string_lossy(),
                }
            });
            ResponseEnvelope::success("template:install", data, start.elapsed().as_millis() as u64)
        }
        Err(e) => ResponseEnvelope::error(
            "template:install",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            start.elapsed().as_millis() as u64,
        ),
    }
}
