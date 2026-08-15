use std::path::PathBuf;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::stack::{StackProfile, UserStack};

pub fn stack_show(_verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match StackProfile::load(&cwd) {
        Some(profile) => {
            let user_stack = UserStack::load(&cwd);
            let effective_style = user_stack
                .as_ref()
                .map(|us| us.effective_style(&profile.style))
                .map(|s| serde_json::to_value(s).unwrap_or_default());
            let mut data = serde_json::to_value(&profile).unwrap_or_default();
            if let Some(us) = &user_stack {
                data["user_stack"] = serde_json::to_value(us).unwrap_or_default();
            }
            if let Some(style) = effective_style {
                data["effective_style"] = style;
            }
            ResponseEnvelope::success("stack show", data, 0)
        }
        None => ResponseEnvelope::error(
            "stack show",
            ErrorResponse::new(
                ErrorCode::ProjectNotFound,
                "No .tsx/stack.json found — run `tsx stack init` first",
            ),
            0,
        ),
    }
}
