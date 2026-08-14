use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data model (matches D3 spec) — the legacy `.tsx/patterns/<id>/pattern.json`
// format, distinct from the newer `forge::PackManifest` (`pack.json`) system.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternArg {
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternOutput {
    pub path: String,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternSlot {
    pub file: String,
    pub marker: String,
    pub insert: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternDefinition {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub args: Vec<PatternArg>,
    #[serde(default)]
    pub outputs: Vec<PatternOutput>,
    #[serde(default)]
    pub slots: Vec<PatternSlot>,
    #[serde(default)]
    pub post_hooks: Vec<String>,
    #[serde(default)]
    pub version: String,
}

impl PatternDefinition {
    /// Directory for this pattern: `.tsx/patterns/<id>/`
    pub fn dir(root: &Path, id: &str) -> PathBuf {
        root.join(".tsx").join("patterns").join(id)
    }

    /// Pattern manifest path: `.tsx/patterns/<id>/pattern.json`
    pub fn manifest_path(root: &Path, id: &str) -> PathBuf {
        Self::dir(root, id).join("pattern.json")
    }

    /// Load a pattern by id from the project root.
    pub fn load(root: &Path, id: &str) -> Option<Self> {
        let path = Self::manifest_path(root, id);
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save the pattern manifest.
    pub fn save(&self, root: &Path) -> anyhow::Result<()> {
        let dir = Self::dir(root, &self.id);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("pattern.json");
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// List all pattern ids in `.tsx/patterns/`.
    pub fn list_ids(root: &Path) -> Vec<String> {
        let patterns_dir = root.join(".tsx").join("patterns");
        let Ok(entries) = std::fs::read_dir(&patterns_dir) else {
            return Vec::new();
        };
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter(|e| e.path().join("pattern.json").exists())
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn pattern_save_and_load() {
        let dir = TempDir::new().unwrap();
        let pattern = PatternDefinition {
            id: "add-service".to_string(),
            description: "Test pattern".to_string(),
            args: vec![PatternArg { name: "name".to_string(), arg_type: "string".to_string(), description: None }],
            outputs: vec![PatternOutput { path: "src/{{name}}.ts".to_string(), template: "service.forge".to_string() }],
            slots: Vec::new(),
            post_hooks: Vec::new(),
            version: "1.0.0".to_string(),
        };
        pattern.save(dir.path()).unwrap();
        let loaded = PatternDefinition::load(dir.path(), "add-service").unwrap();
        assert_eq!(loaded.id, "add-service");
        assert_eq!(loaded.args.len(), 1);
    }

    #[test]
    fn list_ids_finds_saved_patterns() {
        let dir = TempDir::new().unwrap();
        let p = PatternDefinition { id: "my-pattern".to_string(), ..Default::default() };
        p.save(dir.path()).unwrap();
        let ids = PatternDefinition::list_ids(dir.path());
        assert!(ids.contains(&"my-pattern".to_string()));
    }
}
