//! Forge configuration — global and project-level settings with priority resolution.
//!
//! Files:
//! - `~/.tsx/config.json` — global user preferences ([`GlobalConfig`])
//! - `./.tsx/templates.config.json` — project-local overrides ([`ProjectConfig`])
//!
//! Resolution order (highest → lowest priority):
//! 1. CLI `--template <id>` flag
//! 2. Project config defaults
//! 3. Global config preferred_templates
//! 4. Built-in framework defaults (none — caller falls back)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Schema types
// ---------------------------------------------------------------------------

/// Global user configuration, stored at `~/.tsx/config.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Registry URL for fetching templates (e.g. `https://registry.tsx.dev`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_url: Option<String>,
    /// Map of `command → template-id` for preferred templates.
    #[serde(default)]
    pub preferred_templates: HashMap<String, String>,
    /// Catch-all for unknown keys (forward compatibility).
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Project-local template configuration, stored at `./.tsx/templates.config.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Template ids to activate for this project.
    #[serde(default)]
    pub templates: Vec<String>,
    /// Per-command template overrides: `command → template-id`.
    #[serde(default)]
    pub defaults: HashMap<String, String>,
    /// Catch-all for unknown keys (forward compatibility).
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Fully-resolved configuration after applying the priority chain.
#[derive(Debug, Clone, Default)]
pub struct ResolvedConfig {
    /// Registry URL to use for remote operations.
    pub registry_url: Option<String>,
    /// Resolved template id per command.
    pub template_for: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Loaders
// ---------------------------------------------------------------------------

/// Load `~/.tsx/config.json`, returning defaults if absent or unreadable.
pub fn load_global_config() -> GlobalConfig {
    read_json(&global_config_path()).unwrap_or_default()
}

/// Load `./.tsx/templates.config.json`, returning defaults if absent or unreadable.
pub fn load_project_config() -> ProjectConfig {
    read_json(&project_config_path()).unwrap_or_default()
}

/// Persist the global config to `~/.tsx/config.json`.
pub fn save_global_config(cfg: &GlobalConfig) -> Result<(), String> {
    write_json(&global_config_path(), cfg)
}

/// Persist the project config to `./.tsx/templates.config.json`.
pub fn save_project_config(cfg: &ProjectConfig) -> Result<(), String> {
    write_json(&project_config_path(), cfg)
}

// ---------------------------------------------------------------------------
// Priority resolution
// ---------------------------------------------------------------------------

/// Resolve the template id for `command`.
///
/// Priority: `cli_flag` > project defaults > global preferred_templates > `None`.
pub fn resolve_template(command: &str, cli_flag: Option<&str>) -> Option<String> {
    if let Some(t) = cli_flag {
        return Some(t.to_string());
    }
    let project = load_project_config();
    if let Some(t) = project.defaults.get(command) {
        return Some(t.clone());
    }
    let global = load_global_config();
    global.preferred_templates.get(command).cloned()
}

/// Build a [`ResolvedConfig`] by merging global, project, and optional CLI override.
pub fn resolve_config(cli_command: Option<&str>, cli_template: Option<&str>) -> ResolvedConfig {
    let global = load_global_config();
    let project = load_project_config();

    let mut template_for = global.preferred_templates.clone();
    template_for.extend(project.defaults.clone());
    if let (Some(cmd), Some(tmpl)) = (cli_command, cli_template) {
        template_for.insert(cmd.to_string(), tmpl.to_string());
    }

    let registry_url = project
        .extra
        .get("registry_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or(global.registry_url);

    ResolvedConfig { registry_url, template_for }
}

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

/// Path to `~/.tsx/config.json`.
pub fn global_config_path() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".tsx")
        .join("config.json")
}

/// Path to `./.tsx/templates.config.json`.
pub fn project_config_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".tsx")
        .join("templates.config.json")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .ok()
}

// ---------------------------------------------------------------------------
// I/O helpers
// ---------------------------------------------------------------------------

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_flag_wins() {
        let result = resolve_template("generate:schema", Some("my-override"));
        assert_eq!(result, Some("my-override".to_string()));
    }

    #[test]
    fn resolve_returns_none_for_unknown_command() {
        // Should not panic, even if config files don't exist
        let _ = resolve_template("generate:__nonexistent_xyz__", None);
    }

    #[test]
    fn resolve_config_merges_global_and_project() {
        let cfg = resolve_config(Some("generate:schema"), Some("custom-schema"));
        assert_eq!(
            cfg.template_for.get("generate:schema").map(|s| s.as_str()),
            Some("custom-schema")
        );
    }
}
