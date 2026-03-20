//! `tsx pattern` — user-defined generator patterns (D1–D4).
//!
//! Patterns let users teach the CLI new generators without writing a full framework package.
//! They are stored at `.tsx/patterns/<id>/pattern.json` alongside any `.forge` template files.
//!
//! ## Subcommands
//! - `tsx pattern add` — register a pattern from an existing template + arg spec
//! - `tsx pattern record` / `tsx pattern record --stop` — watch file changes, extract pattern
//! - `tsx pattern list` — list all local patterns
//! - `tsx pattern show <id>` — show pattern details
//! - `tsx pattern run <id>` — run a pattern (delegates to the `run` infrastructure)
//! - `tsx pattern share` — publish a pattern to the tsx registry as a micro-package (stub)
//! - `tsx pattern remove <id>` — remove a pattern

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

// ---------------------------------------------------------------------------
// Data model (matches D3 spec)
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

// ---------------------------------------------------------------------------
// Record session state — stored at `.tsx/patterns/.record`
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct RecordSession {
    name: String,
    started_at: String,
    /// Snapshot of files at record start: path → content-hash
    baseline: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

pub fn pattern_add(
    name: String,
    description: Option<String>,
    template: Option<String>,
    args_spec: Option<String>,
    _verbose: bool,
) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Parse args spec: "name:string, entity:string, methods:string[]"
    let args = parse_args_spec(args_spec.as_deref().unwrap_or(""));

    // Determine output template name
    let template_file = template.as_deref().unwrap_or("template.forge");
    let template_base = PathBuf::from(template_file)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("template.forge")
        .to_string();

    let pattern = PatternDefinition {
        id: name.clone(),
        description: description.unwrap_or_else(|| format!("User-defined pattern: {}", name)),
        args,
        outputs: vec![PatternOutput {
            path: format!("{{{{paths.{}}}}}/{{{{kebab(name)}}}}.ts", name.replace('-', "_")),
            template: template_base.clone(),
        }],
        slots: Vec::new(),
        post_hooks: Vec::new(),
        version: "1.0.0".to_string(),
    };

    match pattern.save(&cwd) {
        Ok(_) => {
            let pattern_dir = PatternDefinition::dir(&cwd, &name);

            // Copy the template file into the pattern directory if it exists and is external
            if let Some(tmpl) = &template {
                let src = PathBuf::from(tmpl);
                if src.exists() && src != pattern_dir.join(&template_base) {
                    let _ = std::fs::copy(&src, pattern_dir.join(&template_base));
                }
            }

            ResponseEnvelope::success(
                "pattern add",
                serde_json::json!({
                    "id": name,
                    "manifest": PatternDefinition::manifest_path(&cwd, &name).to_string_lossy(),
                    "template_dir": pattern_dir.to_string_lossy(),
                    "pattern": serde_json::to_value(&pattern).unwrap_or_default(),
                }),
                0,
            )
            .with_next_steps(vec![
                format!("Edit the template at {}", pattern_dir.join(&template_base).display()),
                format!("Run the pattern with: tsx run {}", name),
                format!("Share it: tsx pattern share --name {}", name),
            ])
        }
        Err(e) => ResponseEnvelope::error(
            "pattern add",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

pub fn pattern_record_start(name: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let session_file = cwd.join(".tsx").join("patterns").join(".record");

    if session_file.exists() {
        return ResponseEnvelope::error(
            "pattern record",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                "A recording session is already active. Run `tsx pattern record --stop` first.",
            ),
            0,
        );
    }

    // Snapshot the current working directory (top-level files only for speed)
    let baseline = snapshot_dir(&cwd);
    let session = RecordSession {
        name: name.clone(),
        started_at: chrono_now(),
        baseline,
    };

    if let Some(parent) = session_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match std::fs::write(&session_file, serde_json::to_string_pretty(&session).unwrap_or_default()) {
        Ok(_) => ResponseEnvelope::success(
            "pattern record",
            serde_json::json!({
                "status": "recording",
                "name": name,
                "message": "Recording started. Create or edit files, then run `tsx pattern record --stop`.",
            }),
            0,
        ),
        Err(e) => ResponseEnvelope::error(
            "pattern record",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

pub fn pattern_record_stop(_verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let session_file = cwd.join(".tsx").join("patterns").join(".record");

    let session_content = match std::fs::read_to_string(&session_file) {
        Ok(s) => s,
        Err(_) => {
            return ResponseEnvelope::error(
                "pattern record",
                ErrorResponse::new(
                    ErrorCode::ProjectNotFound,
                    "No active recording session. Run `tsx pattern record --name <name>` first.",
                ),
                0,
            )
        }
    };

    let session: RecordSession = match serde_json::from_str(&session_content) {
        Ok(s) => s,
        Err(e) => {
            return ResponseEnvelope::error(
                "pattern record",
                ErrorResponse::new(ErrorCode::InternalError, format!("Corrupt session file: {}", e)),
                0,
            )
        }
    };

    // Diff the current state against the baseline
    let current = snapshot_dir(&cwd);
    let mut new_files: Vec<String> = Vec::new();
    let mut modified_files: Vec<String> = Vec::new();

    for (path, hash) in &current {
        if let Some(old_hash) = session.baseline.get(path) {
            if old_hash != hash {
                modified_files.push(path.clone());
            }
        } else {
            new_files.push(path.clone());
        }
    }

    let _ = std::fs::remove_file(&session_file);

    // If new files were created, create a pattern from the first one
    let all_changed: Vec<String> = new_files.iter().chain(modified_files.iter()).cloned().collect();

    if all_changed.is_empty() {
        return ResponseEnvelope::success(
            "pattern record",
            serde_json::json!({
                "status": "stopped",
                "name": session.name,
                "changed_files": 0,
                "message": "No file changes detected. Pattern not created.",
            }),
            0,
        );
    }

    // Create a pattern definition from the recorded changes
    let pattern = PatternDefinition {
        id: session.name.clone(),
        description: format!("Recorded pattern: {}", session.name),
        args: vec![PatternArg {
            name: "name".to_string(),
            arg_type: "string".to_string(),
            description: Some("Feature name".to_string()),
        }],
        outputs: all_changed
            .iter()
            .map(|f| PatternOutput {
                path: templatize_path(f),
                template: PathBuf::from(f)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("template.forge")
                    .to_string()
                    + ".forge",
            })
            .collect(),
        slots: Vec::new(),
        post_hooks: Vec::new(),
        version: "1.0.0".to_string(),
    };

    // Copy changed files into pattern directory as template stubs
    let pattern_dir = PatternDefinition::dir(&cwd, &session.name);
    let _ = std::fs::create_dir_all(&pattern_dir);
    for file in &all_changed {
        let src = cwd.join(file);
        if src.exists() {
            let dest_name = format!("{}.forge", src.file_name().and_then(|n| n.to_str()).unwrap_or("template"));
            let _ = std::fs::copy(&src, pattern_dir.join(&dest_name));
        }
    }

    match pattern.save(&cwd) {
        Ok(_) => ResponseEnvelope::success(
            "pattern record",
            serde_json::json!({
                "status": "captured",
                "name": session.name,
                "changed_files": all_changed.len(),
                "new_files": new_files,
                "modified_files": modified_files,
                "pattern": serde_json::to_value(&pattern).unwrap_or_default(),
            }),
            0,
        )
        .with_next_steps(vec![
            format!(
                "Edit templates in {}",
                pattern_dir.display()
            ),
            format!("Add {{{{name}}}} and other placeholders to the templates"),
            format!("Run with: tsx run {}", session.name),
        ]),
        Err(e) => ResponseEnvelope::error(
            "pattern record",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

pub fn pattern_list(_verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let ids = PatternDefinition::list_ids(&cwd);

    let patterns: Vec<serde_json::Value> = ids
        .iter()
        .filter_map(|id| PatternDefinition::load(&cwd, id))
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "description": p.description,
                "args": p.args.iter().map(|a| format!("{}:{}", a.name, a.arg_type)).collect::<Vec<_>>(),
                "outputs": p.outputs.len(),
            })
        })
        .collect();

    ResponseEnvelope::success(
        "pattern list",
        serde_json::json!({
            "count": patterns.len(),
            "patterns": patterns,
        }),
        0,
    )
}

pub fn pattern_show(id: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match PatternDefinition::load(&cwd, &id) {
        Some(p) => ResponseEnvelope::success(
            "pattern show",
            serde_json::to_value(&p).unwrap_or_default(),
            0,
        ),
        None => ResponseEnvelope::error(
            "pattern show",
            ErrorResponse::new(
                ErrorCode::UnknownCommand,
                format!("Pattern '{}' not found in .tsx/patterns/", id),
            ),
            0,
        ),
    }
}

pub fn pattern_remove(id: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dir = PatternDefinition::dir(&cwd, &id);

    if !dir.exists() {
        return ResponseEnvelope::error(
            "pattern remove",
            ErrorResponse::new(
                ErrorCode::UnknownCommand,
                format!("Pattern '{}' not found in .tsx/patterns/", id),
            ),
            0,
        );
    }

    match std::fs::remove_dir_all(&dir) {
        Ok(_) => ResponseEnvelope::success(
            "pattern remove",
            serde_json::json!({ "removed": id }),
            0,
        ),
        Err(e) => ResponseEnvelope::error(
            "pattern remove",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

pub fn pattern_share(name: String, version: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let ver = version.unwrap_or_else(|| "1.0.0".to_string());

    match PatternDefinition::load(&cwd, &name) {
        None => ResponseEnvelope::error(
            "pattern share",
            ErrorResponse::new(
                ErrorCode::UnknownCommand,
                format!("Pattern '{}' not found. Run `tsx pattern list` to see available patterns.", name),
            ),
            0,
        ),
        Some(_) => ResponseEnvelope::success(
            "pattern share",
            serde_json::json!({
                "name": name,
                "version": ver,
                "status": "Publishing patterns to the tsx registry is coming soon.",
                "workaround": "You can share the .tsx/patterns/<id>/ directory manually or publish it as an npm package.",
                "npm_example": format!("cd .tsx/patterns/{} && npm publish --access public", name),
            }),
            0,
        ),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_args_spec(spec: &str) -> Vec<PatternArg> {
    if spec.trim().is_empty() {
        return Vec::new();
    }
    spec.split(',')
        .filter_map(|part| {
            let part = part.trim();
            if let Some(colon) = part.find(':') {
                let name = part[..colon].trim().to_string();
                let arg_type = part[colon + 1..].trim().to_string();
                if !name.is_empty() {
                    return Some(PatternArg { name, arg_type, description: None });
                }
            } else if !part.is_empty() {
                return Some(PatternArg {
                    name: part.to_string(),
                    arg_type: "string".to_string(),
                    description: None,
                });
            }
            None
        })
        .collect()
}

/// Create a lightweight snapshot of a directory: relative path → simple content hash.
fn snapshot_dir(dir: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return map; };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Ok(rel) = path.strip_prefix(dir) {
                let key = rel.to_string_lossy().replace('\\', "/");
                // Simple hash: file size + first 64 bytes
                if let Ok(content) = std::fs::read(&path) {
                    let hash = format!("{}-{}", content.len(), &hex_first64(&content));
                    map.insert(key, hash);
                }
            }
        }
    }
    map
}

fn hex_first64(data: &[u8]) -> String {
    data.iter()
        .take(64)
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// Templatize a file path: replace common name-like segments with {{name}}.
fn templatize_path(path: &str) -> String {
    // Simple heuristic: replace the filename stem with {{kebab(name)}}
    let p = PathBuf::from(path);
    if let Some(parent) = p.parent() {
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("ts");
        let parent_str = parent.to_string_lossy();
        if parent_str.is_empty() || parent_str == "." {
            return format!("{{{{kebab(name)}}}}.{}", ext);
        }
        return format!("{}/{{{{kebab(name)}}}}.{}", parent_str, ext);
    }
    path.to_string()
}

fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parse_args_spec_basic() {
        let args = parse_args_spec("name:string, entity:string, methods:string[]");
        assert_eq!(args.len(), 3);
        assert_eq!(args[0].name, "name");
        assert_eq!(args[1].arg_type, "string");
        assert_eq!(args[2].name, "methods");
    }

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
