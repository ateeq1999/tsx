use std::path::PathBuf;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::stack::StackProfile;

pub fn stack_add(package: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut profile = StackProfile::load(&cwd).unwrap_or_default();
    profile.add_package(&package);

    match profile.save(&cwd) {
        Ok(_) => ResponseEnvelope::success(
            "stack add",
            serde_json::json!({
                "added": package,
                "packages": profile.packages
            }),
            0,
        ),
        Err(e) => ResponseEnvelope::error(
            "stack add",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}
