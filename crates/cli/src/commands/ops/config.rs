//! `tsx config` — manage `~/.tsx/config.json` (global user settings).
//!
//! Subcommands:
//! - `tsx config get <key>`        — print a single config value
//! - `tsx config set <key> <val>`  — write a value
//! - `tsx config list`             — print all key=value pairs

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

// ---------------------------------------------------------------------------
// Config schema
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TsxConfig {
    /// Default registry URL (default: https://tsx-tsnv.onrender.com)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    /// Default framework slug to assume when none is detected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_framework: Option<String>,
    /// Preferred output style: "json" | "pretty"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_style: Option<String>,
    /// Preferred TUI theme: "default" | "dark" | "light"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tui_theme: Option<String>,
    /// Whether to emit token-usage estimates in responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_estimates: Option<bool>,
    /// Extra arbitrary keys for forward compatibility
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Public entrypoints
// ---------------------------------------------------------------------------

pub fn config_get(key: String) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let cfg = load_config();

    let value = get_key(&cfg, &key);
    match value {
        Some(v) => {
            let result = serde_json::json!({ "key": key, "value": v });
            ResponseEnvelope::success("config get", result, start.elapsed().as_millis() as u64)
        }
        None => ResponseEnvelope::error(
            "config get",
            ErrorResponse::new(
                ErrorCode::TemplateNotFound,
                format!("Key '{}' not found in config. Run `tsx config list` to see all keys.", key),
            ),
            start.elapsed().as_millis() as u64,
        ),
    }
}

pub fn config_set(key: String, value: String) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let mut cfg = load_config();

    set_key(&mut cfg, &key, value.clone());

    if let Err(e) = save_config(&cfg) {
        return ResponseEnvelope::error(
            "config set",
            ErrorResponse::new(ErrorCode::InternalError, format!("Could not save config: {}", e)),
            start.elapsed().as_millis() as u64,
        );
    }

    let result = serde_json::json!({ "key": key, "value": value, "saved_to": config_path().to_string_lossy() });
    ResponseEnvelope::success("config set", result, start.elapsed().as_millis() as u64)
}

pub fn config_list() -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let cfg = load_config();
    let path = config_path();

    let mut map: HashMap<String, serde_json::Value> = HashMap::new();
    if let Some(v) = &cfg.registry { map.insert("registry".into(), serde_json::json!(v)); }
    if let Some(v) = &cfg.default_framework { map.insert("default_framework".into(), serde_json::json!(v)); }
    if let Some(v) = &cfg.output_style { map.insert("output_style".into(), serde_json::json!(v)); }
    if let Some(v) = &cfg.tui_theme { map.insert("tui_theme".into(), serde_json::json!(v)); }
    if let Some(v) = cfg.token_estimates { map.insert("token_estimates".into(), serde_json::json!(v)); }
    for (k, v) in &cfg.extra { map.insert(k.clone(), v.clone()); }

    let result = serde_json::json!({
        "config_path": path.to_string_lossy(),
        "values": map,
    });
    ResponseEnvelope::success("config list", result, start.elapsed().as_millis() as u64)
}

pub fn config_reset(key: Option<String>) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    if let Some(k) = key {
        let mut cfg = load_config();
        reset_key(&mut cfg, &k);
        if let Err(e) = save_config(&cfg) {
            return ResponseEnvelope::error(
                "config reset",
                ErrorResponse::new(ErrorCode::InternalError, format!("Could not save config: {}", e)),
                start.elapsed().as_millis() as u64,
            );
        }
        let result = serde_json::json!({ "reset": k });
        ResponseEnvelope::success("config reset", result, start.elapsed().as_millis() as u64)
    } else {
        // Reset entire file
        let empty = TsxConfig::default();
        if let Err(e) = save_config(&empty) {
            return ResponseEnvelope::error(
                "config reset",
                ErrorResponse::new(ErrorCode::InternalError, format!("Could not save config: {}", e)),
                start.elapsed().as_millis() as u64,
            );
        }
        let result = serde_json::json!({ "reset": "all", "path": config_path().to_string_lossy() });
        ResponseEnvelope::success("config reset", result, start.elapsed().as_millis() as u64)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn config_path() -> PathBuf {
    dirs_home()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".tsx")
        .join("config.json")
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .ok()
}

fn load_config() -> TsxConfig {
    let path = config_path();
    if !path.exists() {
        return TsxConfig::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(cfg: &TsxConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

fn get_key(cfg: &TsxConfig, key: &str) -> Option<serde_json::Value> {
    match key {
        "registry" => cfg.registry.as_ref().map(|v| serde_json::json!(v)),
        "default_framework" => cfg.default_framework.as_ref().map(|v| serde_json::json!(v)),
        "output_style" => cfg.output_style.as_ref().map(|v| serde_json::json!(v)),
        "tui_theme" => cfg.tui_theme.as_ref().map(|v| serde_json::json!(v)),
        "token_estimates" => cfg.token_estimates.map(|v| serde_json::json!(v)),
        other => cfg.extra.get(other).cloned(),
    }
}

fn set_key(cfg: &mut TsxConfig, key: &str, value: String) {
    match key {
        "registry" => cfg.registry = Some(value),
        "default_framework" => cfg.default_framework = Some(value),
        "output_style" => cfg.output_style = Some(value),
        "tui_theme" => cfg.tui_theme = Some(value),
        "token_estimates" => cfg.token_estimates = Some(value == "true" || value == "1"),
        other => { cfg.extra.insert(other.to_string(), serde_json::json!(value)); }
    }
}

fn reset_key(cfg: &mut TsxConfig, key: &str) {
    match key {
        "registry" => cfg.registry = None,
        "default_framework" => cfg.default_framework = None,
        "output_style" => cfg.output_style = None,
        "tui_theme" => cfg.tui_theme = None,
        "token_estimates" => cfg.token_estimates = None,
        other => { cfg.extra.remove(other); }
    }
}
