use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::types::PatternDefinition;

pub fn pattern_share(name: String, version: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let ver = version.unwrap_or_else(|| "1.0.0".to_string());

    match PatternDefinition::load(&cwd, &name) {
        None => ResponseEnvelope::error(
            "pattern share",
            ErrorResponse::new(
                ErrorCode::UnknownCommand,
                format!("Pattern '{}' not found. Run `tsx pattern list` to see available patterns.", name),
            ),
            0,
        ),
        Some(_) => ResponseEnvelope::success(
            "pattern share",
            serde_json::json!({
                "name": name,
                "version": ver,
                "status": "Publishing patterns to the tsx registry is coming soon.",
                "workaround": "You can share the .tsx/patterns/<id>/ directory manually or publish it as an npm package.",
                "npm_example": format!("cd .tsx/patterns/{} && npm publish --access public", name),
            }),
            0,
        ),
    }
}
