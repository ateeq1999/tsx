//! Template discovery and installation.
//!
//! Scans template directories in priority order:
//! - `./.tsx/templates/` — project-local (highest priority)
//! - `~/.tsx/templates/` — global user templates
//!
//! Each subdirectory is a template bundle; its `manifest.json` provides metadata.
//! Duplicates (same `id`) are deduplicated — first source encountered wins.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::ForgeError;
use crate::manifest::TemplateManifest;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Where a template was discovered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateSource {
    /// Found in `./.tsx/templates/`
    Project,
    /// Found in `~/.tsx/templates/`
    Global,
    /// Built into the tsx binary
    Framework,
}

impl std::fmt::Display for TemplateSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateSource::Project => write!(f, "project"),
            TemplateSource::Global => write!(f, "global"),
            TemplateSource::Framework => write!(f, "framework"),
        }
    }
}

/// Metadata about a single discovered template bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Unique id (from `manifest.json` or directory name).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    pub version: String,
    pub description: String,
    /// Where the template was found.
    pub source: TemplateSource,
    /// Absolute path to the template directory.
    pub path: PathBuf,
    /// Parsed manifest, if one existed.
    pub manifest: Option<TemplateManifest>,
}

// ---------------------------------------------------------------------------
// Discovery
// ---------------------------------------------------------------------------

/// Discover all templates across project and global sources.
///
/// Project-local templates shadow global ones with the same id.
pub fn discover_templates() -> Vec<TemplateInfo> {
    let mut results: Vec<TemplateInfo> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    if let Some(dir) = project_templates_dir() {
        collect_from_dir(&dir, TemplateSource::Project, &mut results, &mut seen);
    }
    if let Some(dir) = global_templates_dir() {
        collect_from_dir(&dir, TemplateSource::Global, &mut results, &mut seen);
    }

    results
}

/// Discover templates from one specific source.
pub fn discover_from_source(source: TemplateSource) -> Vec<TemplateInfo> {
    let dir = match &source {
        TemplateSource::Project => project_templates_dir(),
        TemplateSource::Global  => global_templates_dir(),
        TemplateSource::Framework => return Vec::new(),
    };
    let Some(dir) = dir else { return Vec::new() };

    let mut results = Vec::new();
    let mut seen = HashSet::new();
    collect_from_dir(&dir, source, &mut results, &mut seen);
    results
}

/// Find a template by id, respecting priority order.
pub fn find_template(id: &str) -> Option<TemplateInfo> {
    discover_templates().into_iter().find(|t| t.id == id)
}

// ---------------------------------------------------------------------------
// Directory paths
// ---------------------------------------------------------------------------

/// `~/.tsx/templates/`
pub fn global_templates_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".tsx").join("templates"))
}

/// `./.tsx/templates/`
pub fn project_templates_dir() -> Option<PathBuf> {
    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join(".tsx").join("templates"))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .ok()
}

// ---------------------------------------------------------------------------
// Internal scan
// ---------------------------------------------------------------------------

fn collect_from_dir(
    dir: &Path,
    source: TemplateSource,
    results: &mut Vec<TemplateInfo>,
    seen: &mut HashSet<String>,
) {
    if !dir.is_dir() {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest = load_manifest_optional(&path);
        let id = manifest
            .as_ref()
            .map(|m| m.id.clone())
            .unwrap_or_else(|| dir_stem(&path));

        if !seen.insert(id.clone()) {
            continue; // higher-priority source already registered
        }

        results.push(TemplateInfo {
            name: manifest.as_ref().map(|m| m.name.clone()).unwrap_or_else(|| id.clone()),
            version: manifest.as_ref().map(|m| m.version.clone()).unwrap_or_else(|| "0.0.0".into()),
            description: manifest.as_ref().map(|m| m.description.clone()).unwrap_or_default(),
            id,
            source: source.clone(),
            path,
            manifest,
        });
    }
}

fn load_manifest_optional(dir: &Path) -> Option<TemplateManifest> {
    let content = std::fs::read_to_string(dir.join("manifest.json")).ok()?;
    serde_json::from_str(&content).ok()
}

fn dir_stem(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

// ---------------------------------------------------------------------------
// Installation
// ---------------------------------------------------------------------------

/// Install a template bundle from a local directory into `~/.tsx/templates/`.
pub fn install_from_dir(src: &Path) -> Result<TemplateInfo, ForgeError> {
    let global_dir = global_templates_dir()
        .ok_or_else(|| ForgeError::LoadError("Cannot determine home directory".into()))?;

    let manifest = load_manifest_optional(src).ok_or_else(|| {
        ForgeError::LoadError(format!(
            "No manifest.json found in {}",
            src.display()
        ))
    })?;

    let dest = global_dir.join(&manifest.id);
    if dest.exists() {
        return Err(ForgeError::OutputConflict(format!(
            "Template '{}' is already installed at {}",
            manifest.id,
            dest.display()
        )));
    }

    copy_dir_all(src, &dest)
        .map_err(|e| ForgeError::LoadError(format!("Copy failed: {e}")))?;

    Ok(TemplateInfo {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        source: TemplateSource::Global,
        path: dest,
        manifest: Some(manifest),
    })
}

/// Remove a template by id from `~/.tsx/templates/`.
pub fn uninstall(id: &str) -> Result<(), ForgeError> {
    let global_dir = global_templates_dir()
        .ok_or_else(|| ForgeError::LoadError("Cannot determine home directory".into()))?;
    let target = global_dir.join(id);
    if !target.exists() {
        return Err(ForgeError::TemplateNotFound(format!(
            "Template '{}' is not installed",
            id
        )));
    }
    std::fs::remove_dir_all(&target)
        .map_err(|e| ForgeError::LoadError(format!("Could not remove {}: {e}", target.display())))
}

/// Scaffold a new template directory with a minimal `manifest.json`.
pub fn init_template(name: &str, dest: &Path) -> Result<(), ForgeError> {
    std::fs::create_dir_all(dest)
        .map_err(|e| ForgeError::LoadError(format!("Could not create {}: {e}", dest.display())))?;

    let manifest = serde_json::json!({
        "id": name,
        "name": name,
        "version": "0.1.0",
        "description": "",
        "generates": []
    });
    let content = serde_json::to_string_pretty(&manifest)
        .map_err(|e| ForgeError::LoadError(e.to_string()))?;
    std::fs::write(dest.join("manifest.json"), content)
        .map_err(|e| ForgeError::LoadError(e.to_string()))?;

    // Scaffold a minimal README
    std::fs::write(
        dest.join("README.md"),
        format!("# {name}\n\nA forge template bundle.\n"),
    )
    .map_err(|e| ForgeError::LoadError(e.to_string()))?;

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Template schema (JSON Schema for agent autocomplete)
// ---------------------------------------------------------------------------

/// Return a JSON Schema object describing the input for `command` in the template `id`.
/// Returns `None` if the template or command is not found.
pub fn template_schema(id: &str, command: &str) -> Option<serde_json::Value> {
    let info = find_template(id)?;
    let manifest = info.manifest?;
    let output = manifest.generates.into_iter().find(|o| o.id == command)?;
    // Try to read the first template file and extract its @schema
    let tmpl_path = info.path.join(&output.template);
    let src = std::fs::read_to_string(&tmpl_path).ok()?;
    crate::validate::extract_schema(&src).ok().flatten()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_returns_vec() {
        // Must not panic even when dirs do not exist
        let _ = discover_templates();
    }

    #[test]
    fn dir_stem_works() {
        let p = PathBuf::from("/home/.tsx/templates/my-tmpl");
        assert_eq!(dir_stem(&p), "my-tmpl");
    }

    #[test]
    fn source_display() {
        assert_eq!(TemplateSource::Global.to_string(), "global");
        assert_eq!(TemplateSource::Project.to_string(), "project");
        assert_eq!(TemplateSource::Framework.to_string(), "framework");
    }

    #[test]
    fn find_template_returns_none_for_unknown() {
        assert!(find_template("__nonexistent_xyz__").is_none());
    }
}
