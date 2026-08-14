use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::utils::{download_bytes, read_registry_url, urlencoding_simple};

/// Search the registry for pattern packs.
pub fn pattern_search(query: String, registry: Option<String>, framework: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let registry_url = registry.unwrap_or_else(|| read_registry_url(&cwd));

    let mut search_url = format!(
        "{}/v1/patterns/search?q={}",
        registry_url.trim_end_matches('/'),
        urlencoding_simple(&query),
    );
    if let Some(fw) = &framework {
        search_url.push_str(&format!("&framework={}", urlencoding_simple(fw)));
    }

    match download_bytes(&search_url) {
        Ok(bytes) => {
            match serde_json::from_slice::<serde_json::Value>(&bytes) {
                Ok(json) => ResponseEnvelope::success("pattern search", json, 0),
                Err(e) => ResponseEnvelope::error(
                    "pattern search",
                    ErrorResponse::new(ErrorCode::InternalError, format!("Parse error: {e}")),
                    0,
                ),
            }
        }
        Err(e) => ResponseEnvelope::error(
            "pattern search",
            ErrorResponse::new(ErrorCode::InternalError, format!("Request failed: {e}")),
            0,
        ),
    }
}
