use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The `plugin.json` manifest format for TSX template plugins.
///
/// A plugin is an npm package that contains a `plugin.json` at its root
/// plus a `templates/` directory following the same atom/molecule/feature
/// hierarchy as the built-in templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    /// Plugin display name.
    pub name: String,

    /// Semantic version (e.g. "1.2.0").
    pub version: String,

    /// One-line description shown in `tsx list plugins`.
    pub description: String,

    /// npm package name used for discovery and installation.
    pub package: String,

    /// TSX CLI version range this plugin is compatible with (semver range).
    pub tsx_version: String,

    /// Template overrides — maps generator id to template path inside the plugin.
    /// e.g. { "add:schema": "templates/features/schema.jinja" }
    #[serde(default)]
    pub overrides: std::collections::HashMap<String, String>,

    /// Additional generators provided by this plugin (beyond the built-ins).
    #[serde(default)]
    pub generators: Vec<PluginGenerator>,

    /// Npm dependencies this plugin requires in the target project.
    #[serde(default)]
    pub peer_dependencies: Vec<String>,

    /// Plugin author.
    #[serde(default)]
    pub author: String,

    /// Link to plugin documentation.
    #[serde(default)]
    pub docs: String,
}

/// A generator contributed by a plugin (extends `tsx list generators`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginGenerator {
    pub id: String,
    pub description: String,
    pub template: String,
    pub options: serde_json::Value,
}

impl PluginManifest {
    /// Load a `plugin.json` from a directory.
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join("plugin.json");
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read plugin.json at {:?}", path))?;
        serde_json::from_str(&content).with_context(|| "Failed to parse plugin.json")
    }

    /// Write a `plugin.json` to a directory.
    pub fn save(&self, dir: &Path) -> Result<()> {
        let path = dir.join("plugin.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content).with_context(|| format!("Cannot write {:?}", path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_roundtrip() {
        let manifest = PluginManifest {
            name: "My Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            package: "tsx-plugin-test".to_string(),
            tsx_version: ">=0.1.0".to_string(),
            overrides: [("add:schema".to_string(), "templates/features/schema.jinja".to_string())]
                .into(),
            generators: vec![],
            peer_dependencies: vec!["drizzle-orm".to_string()],
            author: "Test".to_string(),
            docs: "https://example.com".to_string(),
        };

        let dir = TempDir::new().unwrap();
        manifest.save(dir.path()).unwrap();
        let loaded = PluginManifest::load(dir.path()).unwrap();
        assert_eq!(loaded.name, "My Plugin");
        assert_eq!(loaded.version, "1.0.0");
        assert!(loaded.overrides.contains_key("add:schema"));
    }
}
