//! Multi-file generation manifest (`manifest.json`).
//!
//! A template bundle declares its outputs in `manifest.json`.  The manifest
//! is deserialized into [`TemplateManifest`], and [`render_multi`] uses it
//! to render one template and write N output files with path interpolation.
//!
//! # Example manifest.json
//!
//! ```json
//! {
//!   "id": "my-forms",
//!   "name": "My TanStack Forms",
//!   "version": "1.0.0",
//!   "generates": [
//!     {
//!       "id": "form",
//!       "template": "form.forge",
//!       "outputs": [
//!         { "path": "src/schemas/{name}.ts" },
//!         { "path": "src/components/{name}/form.tsx", "condition": "has_component" }
//!       ]
//!     }
//!   ]
//! }
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engine::Engine;
use crate::context::ForgeContext;
use crate::error::ForgeError;

// ---------------------------------------------------------------------------
// Manifest types
// ---------------------------------------------------------------------------

/// Top-level `manifest.json` for a template bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub generates: Vec<MultiOutput>,
    #[serde(default)]
    pub dependencies: ManifestDependencies,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// One generation target inside the manifest: one template → N output files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiOutput {
    pub id: String,
    #[serde(default)]
    pub description: String,
    pub template: String,
    pub outputs: Vec<OutputPath>,
    /// Optional JSON Schema for validating the variables passed to this target.
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

/// A single output path declaration within a [`MultiOutput`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputPath {
    /// Path template supporting `{variable}` interpolation.
    /// Example: `"src/components/{name}/form.tsx"`
    pub path: String,
    /// Optional condition variable name.  When set, the output is skipped
    /// unless the context contains a truthy value for this key.
    #[serde(default)]
    pub condition: Option<String>,
}

/// npm / template dependencies declared in the manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestDependencies {
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub templates: Vec<String>,
}

// ---------------------------------------------------------------------------
// A rendered output file
// ---------------------------------------------------------------------------

/// One file produced by [`render_multi`].
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// Absolute or project-relative path (after variable interpolation).
    pub path: PathBuf,
    /// Rendered file content.
    pub content: String,
}

// ---------------------------------------------------------------------------
// Manifest loading
// ---------------------------------------------------------------------------

/// Load and deserialize a `manifest.json` from `dir`.
pub fn load_manifest(dir: &Path) -> Result<TemplateManifest, ForgeError> {
    let path = dir.join("manifest.json");
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| ForgeError::LoadError(format!("{}: {e}", path.display())))?;
    serde_json::from_str(&raw)
        .map_err(|e| ForgeError::LoadError(format!("manifest.json parse error: {e}")))
}

// ---------------------------------------------------------------------------
// Path interpolation
// ---------------------------------------------------------------------------

/// Resolve `{variable}` placeholders in a path template using the provided vars.
///
/// Unknown placeholders are left as-is.
pub fn interpolate_path(path_template: &str, vars: &HashMap<String, String>) -> String {
    let mut out = path_template.to_string();
    for (key, val) in vars {
        out = out.replace(&format!("{{{key}}}"), val);
    }
    out
}

// ---------------------------------------------------------------------------
// Multi-file rendering
// ---------------------------------------------------------------------------

/// Render a single [`MultiOutput`] entry, producing one [`GeneratedFile`] per
/// declared output path (respecting `condition` fields).
///
/// `template_dir` is the root directory to load the `.forge` template from.
/// `vars` provides `{placeholder}` values for output path interpolation and
/// is also inserted into the Tera context under their respective keys.
pub fn render_multi(
    output: &MultiOutput,
    template_dir: &Path,
    ctx: &ForgeContext,
    vars: &HashMap<String, String>,
) -> Result<Vec<GeneratedFile>, ForgeError> {
    // Build engine and load templates from the bundle directory
    let mut engine = Engine::new();
    engine.load_dir(template_dir)?;

    let content = engine.render(&output.template, ctx)?;

    let mut files = Vec::new();

    for op in &output.outputs {
        // Condition check — skip if the var is absent or falsy
        if let Some(cond_key) = &op.condition {
            let present = vars
                .get(cond_key)
                .map(|v| !v.is_empty() && v != "false" && v != "0")
                .unwrap_or(false);
            if !present {
                continue;
            }
        }

        let resolved = interpolate_path(&op.path, vars);
        files.push(GeneratedFile {
            path: PathBuf::from(resolved),
            content: content.clone(),
        });
    }

    Ok(files)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_path_replaces_vars() {
        let vars: HashMap<String, String> = [
            ("name".to_string(), "product".to_string()),
            ("type".to_string(), "form".to_string()),
        ]
        .into();
        let result = interpolate_path("src/components/{name}/{type}.tsx", &vars);
        assert_eq!(result, "src/components/product/form.tsx");
    }

    #[test]
    fn interpolate_path_unknown_vars_left_as_is() {
        let vars: HashMap<String, String> = HashMap::new();
        let result = interpolate_path("src/{name}/form.tsx", &vars);
        assert_eq!(result, "src/{name}/form.tsx");
    }

    #[test]
    fn manifest_deserializes() {
        let json = r#"{
            "id": "my-forms",
            "name": "My Forms",
            "version": "1.0.0",
            "generates": [
                {
                    "id": "form",
                    "template": "form.forge",
                    "outputs": [
                        { "path": "src/schemas/{name}.ts" },
                        { "path": "src/components/{name}/form.tsx", "condition": "with_component" }
                    ]
                }
            ]
        }"#;
        let manifest: TemplateManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.id, "my-forms");
        assert_eq!(manifest.generates.len(), 1);
        assert_eq!(manifest.generates[0].outputs.len(), 2);
        assert_eq!(
            manifest.generates[0].outputs[1].condition.as_deref(),
            Some("with_component")
        );
    }

    #[test]
    fn render_multi_skips_conditioned_output() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        // Write a minimal template
        std::fs::write(dir.path().join("hello.forge"), "Hello {{ name }}!").unwrap();

        let output = MultiOutput {
            id: "test".to_string(),
            description: String::new(),
            template: "hello.forge".to_string(),
            outputs: vec![
                OutputPath { path: "out/{name}.txt".to_string(), condition: None },
                OutputPath {
                    path: "opt/{name}.txt".to_string(),
                    condition: Some("opt_flag".to_string()),
                },
            ],
            config: None,
        };

        let vars: HashMap<String, String> = [("name".to_string(), "world".to_string())].into();
        let ctx = ForgeContext::new().insert("name", "world");

        let files = render_multi(&output, dir.path(), &ctx, &vars).unwrap();
        // Only the unconditional output should be produced
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from("out/world.txt"));
        assert_eq!(files[0].content, "Hello world!");
    }

    #[test]
    fn render_multi_includes_conditioned_output_when_flag_set() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("hello.forge"), "Hello {{ name }}!").unwrap();

        let output = MultiOutput {
            id: "test".to_string(),
            description: String::new(),
            template: "hello.forge".to_string(),
            outputs: vec![OutputPath {
                path: "opt/{name}.txt".to_string(),
                condition: Some("opt_flag".to_string()),
            }],
            config: None,
        };

        let vars: HashMap<String, String> = [
            ("name".to_string(), "world".to_string()),
            ("opt_flag".to_string(), "true".to_string()),
        ]
        .into();
        let ctx = ForgeContext::new().insert("name", "world");

        let files = render_multi(&output, dir.path(), &ctx, &vars).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from("opt/world.txt"));
    }
}
