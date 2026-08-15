use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_config_show(_verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let global = forge::load_global_config();
    let project = forge::load_project_config();
    let resolved = forge::resolve_config(None, None);

    let data = serde_json::json!({
        "global_config_path": forge::global_config_path().to_string_lossy(),
        "project_config_path": forge::project_config_path().to_string_lossy(),
        "global": serde_json::to_value(&global).unwrap_or_default(),
        "project": serde_json::to_value(&project).unwrap_or_default(),
        "resolved": {
            "registry_url": resolved.registry_url,
            "template_for": resolved.template_for,
        },
    });
    ResponseEnvelope::success("template:config:show", data, start.elapsed().as_millis() as u64)
}

pub fn template_config_set(key: String, value: String, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let mut cfg = forge::load_global_config();

    match key.as_str() {
        "registry_url" => cfg.registry_url = Some(value.clone()),
        other => {
            cfg.preferred_templates.insert(other.to_string(), value.clone());
        }
    }

    match forge::save_global_config(&cfg) {
        Ok(()) => {
            let data = serde_json::json!({ "key": key, "value": value });
            ResponseEnvelope::success("template:config:set", data, start.elapsed().as_millis() as u64)
        }
        Err(e) => ResponseEnvelope::error(
            "template:config:set",
            ErrorResponse::new(ErrorCode::InternalError, e),
            start.elapsed().as_millis() as u64,
        ),
    }
}

pub fn template_config_init(overwrite: bool, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let global_path = forge::global_config_path();
    let project_path = forge::project_config_path();

    if global_path.exists() && !overwrite {
        return ResponseEnvelope::error(
            "template:config:init",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                format!("{} already exists. Pass --overwrite to replace.", global_path.display()),
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    let created_global = forge::save_global_config(&forge::GlobalConfig::default()).is_ok();
    let created_project = forge::save_project_config(&forge::ProjectConfig::default()).is_ok();

    let data = serde_json::json!({
        "created": {
            "global": if created_global { global_path.to_string_lossy().to_string() } else { String::new() },
            "project": if created_project { project_path.to_string_lossy().to_string() } else { String::new() },
        }
    });
    ResponseEnvelope::success("template:config:init", data, start.elapsed().as_millis() as u64)
}
