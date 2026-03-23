//! Pattern Pack manifest — `pack.json`.
//!
//! A **Pattern Pack** is a named bundle of `.forge` template files plus a `pack.json`
//! manifest that declares its args, outputs, commands, and marker injection points.
//!
//! ## Directory layout
//!
//! ```text
//! .tsx/patterns/<id>/
//! ├── pack.json          ← this manifest
//! ├── <template>.forge   ← one per output
//! ├── atoms/             ← shared atoms (@include targets)
//! └── layouts/           ← base templates (@extends targets)
//! ```
//!
//! ## Minimal `pack.json`
//!
//! ```json
//! {
//!   "id": "my-pack",
//!   "args": [{ "name": "name", "type": "string" }],
//!   "outputs": [{ "id": "main", "template": "main.forge", "path": "src/{{ name | snake_case }}.ts" }],
//!   "commands": { "all": { "description": "Generate all", "outputs": ["main"], "default": true } }
//! }
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Top-level pack manifest — deserialized from `pack.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackManifest {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub framework: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub args: Vec<PackArg>,
    #[serde(default)]
    pub outputs: Vec<PackOutput>,
    /// Named commands, each targeting a subset of outputs.
    /// Key is the command name; use `"default": true` on one to mark it as default.
    #[serde(default)]
    pub commands: HashMap<String, PackCommand>,
    /// Marker injection points in existing project files.
    #[serde(default)]
    pub markers: Vec<PackMarker>,
    /// Post-generation shell commands, keyed by command name.
    /// Use key `"all"` for hooks that run regardless of which command was used.
    #[serde(default)]
    pub post_hooks: HashMap<String, Vec<String>>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// A single input argument declared by the pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackArg {
    pub name: String,
    /// `"string"` | `"bool"` | `"enum"` | `"string[]"`
    #[serde(rename = "type")]
    pub arg_type: String,
    #[serde(default)]
    pub required: bool,
    /// Default value — used when the caller does not supply the arg.
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub description: String,
    /// Allowed values for `"enum"` type.
    #[serde(default)]
    pub options: Vec<String>,
}

/// A single file this pack can generate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackOutput {
    pub id: String,
    /// Relative path to the `.forge` template inside the pack directory.
    pub template: String,
    /// Output path in the target project — may contain `{{ name | snake_case }}` etc.
    pub path: String,
}

/// A named command that generates a subset of the pack's outputs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackCommand {
    #[serde(default)]
    pub description: String,
    /// Which output ids this command generates. Empty means all outputs.
    #[serde(default)]
    pub outputs: Vec<String>,
    /// Mark this command as the default (runs when no command name is given).
    #[serde(default)]
    pub default: bool,
}

/// A forge marker injection: finds `marker` comment in `file` and inserts `insert` after it.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackMarker {
    /// The comment anchor to search for, e.g. `"// [tsx:schemas]"`.
    pub marker: String,
    /// Project-relative path to the file containing the marker.
    pub file: String,
    /// Line to insert after the marker. May use `{{ name | snake_case }}` interpolation.
    pub insert: String,
}

// ---------------------------------------------------------------------------
// PackManifest impl
// ---------------------------------------------------------------------------

impl PackManifest {
    /// Directory for this pack inside the project: `.tsx/patterns/<id>/`.
    pub fn dir(root: &Path, id: &str) -> PathBuf {
        root.join(".tsx").join("patterns").join(id)
    }

    /// `pack.json` manifest path for a given id.
    pub fn manifest_path(root: &Path, id: &str) -> PathBuf {
        Self::dir(root, id).join("pack.json")
    }

    /// Load a pack by id from a project root. Returns `None` if not found or parse fails.
    pub fn load(root: &Path, id: &str) -> Option<Self> {
        let path = Self::manifest_path(root, id);
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Load a pack from a specific directory that contains `pack.json`.
    pub fn load_from_dir(dir: &Path) -> Option<Self> {
        let path = dir.join("pack.json");
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Serialize and write `pack.json` to `.tsx/patterns/<id>/pack.json`.
    pub fn save(&self, root: &Path) -> anyhow::Result<()> {
        let dir = Self::dir(root, &self.id);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("pack.json");
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// List all pack manifests found in `.tsx/patterns/`.
    pub fn list(root: &Path) -> Vec<Self> {
        let patterns_dir = root.join(".tsx").join("patterns");
        let Ok(entries) = std::fs::read_dir(&patterns_dir) else {
            return Vec::new();
        };
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| Self::load_from_dir(&e.path()))
            .collect()
    }

    /// Resolve which command to execute.
    ///
    /// Priority: named command → command marked `"default": true` → `"all"` key → first command.
    pub fn resolve_command(&self, name: Option<&str>) -> Option<&PackCommand> {
        if let Some(n) = name {
            return self.commands.get(n);
        }
        // Find the command marked as default
        if let Some(cmd) = self.commands.values().find(|c| c.default) {
            return Some(cmd);
        }
        // Fall back to "all"
        if let Some(cmd) = self.commands.get("all") {
            return Some(cmd);
        }
        // Last resort: first command
        self.commands.values().next()
    }

    /// Return the `PackOutput` entries targeted by `cmd`.
    /// If `cmd.outputs` is empty, returns all outputs.
    pub fn command_outputs<'a>(&'a self, cmd: &'a PackCommand) -> Vec<&'a PackOutput> {
        if cmd.outputs.is_empty() {
            return self.outputs.iter().collect();
        }
        cmd.outputs
            .iter()
            .filter_map(|id| self.outputs.iter().find(|o| &o.id == id))
            .collect()
    }

    /// Apply defaults for any missing args. Returns a map of all arg values.
    pub fn apply_defaults(
        &self,
        supplied: HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        let mut result = supplied;
        for arg in &self.args {
            if result.contains_key(&arg.name) {
                continue;
            }
            if let Some(default) = &arg.default {
                result.insert(arg.name.clone(), default.clone());
            }
        }
        result
    }

    /// Validate that all required args are present. Returns list of missing arg names.
    pub fn missing_required(&self, args: &HashMap<String, serde_json::Value>) -> Vec<String> {
        self.args
            .iter()
            .filter(|a| a.required && !args.contains_key(&a.name))
            .map(|a| a.name.clone())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Source tracking (for install/update)
// ---------------------------------------------------------------------------

/// Tracks where a pack was installed from, stored in `.tsx/patterns/<id>/.source.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackSource {
    /// `"local"` | `"github"` | `"registry"`
    pub kind: String,
    /// Original source string (path, `github:user/repo#path`, or `@scope/name`)
    pub source: String,
    /// Version or ref at install time
    #[serde(default)]
    pub ref_: String,
    /// ISO 8601 timestamp
    #[serde(default)]
    pub installed_at: String,
}

impl PackSource {
    pub fn path(root: &Path, id: &str) -> PathBuf {
        PackManifest::dir(root, id).join(".source.json")
    }

    pub fn save(&self, root: &Path, id: &str) -> anyhow::Result<()> {
        let path = Self::path(root, id);
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn load(root: &Path, id: &str) -> Option<Self> {
        let path = Self::path(root, id);
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_pack() -> PackManifest {
        let mut commands = HashMap::new();
        commands.insert("all".to_string(), PackCommand {
            description: "All outputs".to_string(),
            outputs: vec!["main".to_string()],
            default: true,
        });
        PackManifest {
            id: "test-pack".to_string(),
            name: "Test Pack".to_string(),
            version: "1.0.0".to_string(),
            description: "A test pack".to_string(),
            author: "test".to_string(),
            framework: "tanstack-start".to_string(),
            tags: vec!["test".to_string()],
            args: vec![PackArg {
                name: "name".to_string(),
                arg_type: "string".to_string(),
                required: true,
                default: None,
                description: "Entity name".to_string(),
                options: vec![],
            }],
            outputs: vec![PackOutput {
                id: "main".to_string(),
                template: "main.forge".to_string(),
                path: "src/{{ name | snake_case }}.ts".to_string(),
            }],
            commands,
            markers: vec![PackMarker {
                marker: "// [tsx:schemas]".to_string(),
                file: "src/index.ts".to_string(),
                insert: "export * from './{{ name | snake_case }}';".to_string(),
            }],
            post_hooks: HashMap::new(),
        }
    }

    #[test]
    fn save_and_load() {
        let dir = TempDir::new().unwrap();
        let pack = sample_pack();
        pack.save(dir.path()).unwrap();

        let loaded = PackManifest::load(dir.path(), "test-pack").unwrap();
        assert_eq!(loaded.id, "test-pack");
        assert_eq!(loaded.args.len(), 1);
        assert_eq!(loaded.outputs.len(), 1);
        assert_eq!(loaded.markers.len(), 1);
    }

    #[test]
    fn list_finds_saved_pack() {
        let dir = TempDir::new().unwrap();
        sample_pack().save(dir.path()).unwrap();
        let packs = PackManifest::list(dir.path());
        assert!(packs.iter().any(|p| p.id == "test-pack"));
    }

    #[test]
    fn resolve_default_command() {
        let pack = sample_pack();
        let cmd = pack.resolve_command(None).unwrap();
        assert!(cmd.default);
        assert_eq!(cmd.outputs, vec!["main"]);
    }

    #[test]
    fn resolve_named_command() {
        let pack = sample_pack();
        let cmd = pack.resolve_command(Some("all")).unwrap();
        assert_eq!(cmd.outputs, vec!["main"]);
    }

    #[test]
    fn apply_defaults_fills_missing() {
        let pack = PackManifest {
            args: vec![PackArg {
                name: "has_done".to_string(),
                arg_type: "bool".to_string(),
                required: false,
                default: Some(serde_json::Value::Bool(true)),
                description: "".to_string(),
                options: vec![],
            }],
            ..Default::default()
        };
        let supplied = HashMap::new();
        let result = pack.apply_defaults(supplied);
        assert_eq!(result.get("has_done"), Some(&serde_json::Value::Bool(true)));
    }

    #[test]
    fn missing_required_detects_gap() {
        let pack = sample_pack();
        let args = HashMap::new();
        let missing = pack.missing_required(&args);
        assert!(missing.contains(&"name".to_string()));

        let mut args_with_name = HashMap::new();
        args_with_name.insert("name".to_string(), serde_json::Value::String("todo".to_string()));
        assert!(pack.missing_required(&args_with_name).is_empty());
    }
}
