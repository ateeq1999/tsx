use std::path::PathBuf;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::stack::StackProfile;

pub fn stack_remove(package: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut profile = StackProfile::load(&cwd).unwrap_or_else(|| {
        StackProfile::default()
    });
    let before = profile.packages.len();
    profile
        .packages
        .retain(|p| p.split('@').next().unwrap_or(p) != package.as_str());
    let removed = before - profile.packages.len();

    if removed == 0 {
        return ResponseEnvelope::error(
            "stack remove",
            ErrorResponse::new(
                ErrorCode::UnknownCommand,
                format!("Package '{}' not found in stack", package),
            ),
            0,
        );
    }

    match profile.save(&cwd) {
        Ok(_) => ResponseEnvelope::success(
            "stack remove",
            serde_json::json!({
                "removed": package,
                "packages": profile.packages
            }),
            0,
        ),
        Err(e) => ResponseEnvelope::error(
            "stack remove",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}
