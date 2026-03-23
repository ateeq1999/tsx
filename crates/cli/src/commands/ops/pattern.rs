//! `tsx pattern` — user-defined generator patterns (D1–D4).
//!
//! Patterns let users teach the CLI new generators without writing a full framework package.
//! They are stored at `.tsx/patterns/<id>/pack.json` alongside any `.forge` template files.
//!
//! ## Subcommands
//! - `tsx pattern new <id>` — scaffold a new pack with starter `pack.json` + `main.forge`
//! - `tsx pattern run <id>` — run a pack command (renders templates + injects markers)
//! - `tsx pattern install <source>` — install from local path or `github:user/repo#path@ref`
//! - `tsx pattern lint <id>` — validate pack templates and manifest
//! - `tsx pattern list` — list all local packs
//! - `tsx pattern show <id>` — show pack details
//! - `tsx pattern remove <id>` — remove a pack

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
    let packs = forge::PackManifest::list(&cwd);

    let items: Vec<serde_json::Value> = packs
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "version": p.version,
                "description": p.description,
                "framework": p.framework,
                "commands": p.commands.keys().collect::<Vec<_>>(),
                "outputs": p.outputs.len(),
            })
        })
        .collect();

    ResponseEnvelope::success(
        "pattern list",
        serde_json::json!({
            "count": items.len(),
            "patterns": items,
        }),
        0,
    )
}

pub fn pattern_show(id: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match forge::PackManifest::load(&cwd, &id) {
        Some(pack) => {
            let pack_dir = forge::PackManifest::dir(&cwd, &id);
            let forge_files: Vec<String> = std::fs::read_dir(&pack_dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("forge"))
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .collect();
            ResponseEnvelope::success(
                "pattern show",
                serde_json::json!({
                    "id": pack.id,
                    "name": pack.name,
                    "version": pack.version,
                    "description": pack.description,
                    "framework": pack.framework,
                    "author": pack.author,
                    "tags": pack.tags,
                    "args": pack.args.iter().map(|a| serde_json::json!({
                        "name": a.name,
                        "type": a.arg_type,
                        "required": a.required,
                        "default": a.default,
                        "description": a.description,
                    })).collect::<Vec<_>>(),
                    "outputs": pack.outputs.iter().map(|o| serde_json::json!({
                        "id": o.id,
                        "template": o.template,
                        "path": o.path,
                    })).collect::<Vec<_>>(),
                    "commands": pack.commands.iter().map(|(k, c)| serde_json::json!({
                        "name": k,
                        "description": c.description,
                        "outputs": c.outputs,
                        "default": c.default,
                    })).collect::<Vec<_>>(),
                    "markers": pack.markers.len(),
                    "forge_files": forge_files,
                }),
                0,
            )
        }
        None => ResponseEnvelope::error(
            "pattern show",
            ErrorResponse::new(
                ErrorCode::UnknownCommand,
                format!("Pack '{}' not found in .tsx/patterns/", id),
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
// New pack system commands
// ---------------------------------------------------------------------------

/// Scaffold a new pack directory with a starter `pack.json` and `main.forge`.
pub fn pattern_new(
    id: String,
    name: Option<String>,
    description: Option<String>,
    framework: Option<String>,
    _verbose: bool,
) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let pack_dir = forge::PackManifest::dir(&cwd, &id);

    if pack_dir.exists() {
        return ResponseEnvelope::error(
            "pattern new",
            ErrorResponse::new(ErrorCode::ValidationError, format!("Pack '{}' already exists at {}", id, pack_dir.display())),
            0,
        );
    }

    let mut commands = std::collections::HashMap::new();
    commands.insert("all".to_string(), forge::PackCommand {
        description: "Generate all outputs".to_string(),
        outputs: vec!["main".to_string()],
        default: true,
    });

    let pack = forge::PackManifest {
        id: id.clone(),
        name: name.unwrap_or_else(|| id.clone()),
        version: "1.0.0".to_string(),
        description: description.unwrap_or_else(|| format!("Pattern pack: {}", id)),
        author: String::new(),
        framework: framework.unwrap_or_default(),
        tags: Vec::new(),
        args: vec![forge::PackArg {
            name: "name".to_string(),
            arg_type: "string".to_string(),
            required: true,
            default: None,
            description: "Feature name".to_string(),
            options: Vec::new(),
        }],
        outputs: vec![forge::PackOutput {
            id: "main".to_string(),
            template: "main.forge".to_string(),
            path: "src/{{ name | snake_case }}.ts".to_string(),
        }],
        commands,
        markers: Vec::new(),
        post_hooks: std::collections::HashMap::new(),
    };

    if let Err(e) = pack.save(&cwd) {
        return ResponseEnvelope::error(
            "pattern new",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        );
    }

    let forge_content = "// {{ name | pascal_case }}\nexport const {{ name | pascal_case }} = () => {\n  // TODO: implement\n};\n";
    let forge_path = pack_dir.join("main.forge");
    if let Err(e) = std::fs::write(&forge_path, forge_content) {
        return ResponseEnvelope::error(
            "pattern new",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        );
    }

    ResponseEnvelope::success(
        "pattern new",
        serde_json::json!({
            "id": id,
            "pack_dir": pack_dir.to_string_lossy(),
            "files_created": ["pack.json", "main.forge"],
        }),
        0,
    )
    .with_next_steps(vec![
        format!("Edit the template at {}", forge_path.display()),
        format!("Run with: tsx pattern run {}", id),
    ])
}

/// Run a pack command — render templates, inject markers, run post-hooks.
pub fn pattern_run(
    id: String,
    command: Option<String>,
    arg_pairs: Vec<String>, // "key=value" pairs
    dry_run: bool,
    overwrite: bool,
    _verbose: bool,
) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let Some(pack) = forge::PackManifest::load(&cwd, &id) else {
        return ResponseEnvelope::error(
            "pattern run",
            ErrorResponse::new(ErrorCode::ProjectNotFound, format!("Pack '{}' not found in .tsx/patterns/", id)),
            0,
        );
    };

    let pack_dir = forge::PackManifest::dir(&cwd, &id);

    let mut args = HashMap::new();
    for pair in &arg_pairs {
        if let Some(eq) = pair.find('=') {
            let key = pair[..eq].trim().to_string();
            let val = pair[eq + 1..].to_string();
            args.insert(key, serde_json::Value::String(val));
        }
    }

    let opts = forge::RunOpts { dry_run, overwrite, command };

    match forge::run_pack(&pack, &pack_dir, args, &cwd, &opts) {
        Ok(result) => ResponseEnvelope::success(
            "pattern run",
            serde_json::json!({
                "dry_run": dry_run,
                "files_written": result.files_written.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                "files_skipped": result.files_skipped.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                "markers_injected": result.markers_injected.iter().map(|(p, l)| serde_json::json!({
                    "file": p.to_string_lossy(), "line": l,
                })).collect::<Vec<_>>(),
                "hooks_run": result.hooks_run,
            }),
            0,
        ),
        Err(e) => ResponseEnvelope::error(
            "pattern run",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

/// Install a pack from a local path or `github:user/repo[#subpath][@ref]`.
pub fn pattern_install(source: String, id_override: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if source.starts_with("github:") {
        pattern_install_github(&source, id_override, &cwd)
    } else {
        pattern_install_local(PathBuf::from(&source), id_override, &cwd)
    }
}

fn pattern_install_local(src: PathBuf, id_override: Option<String>, root: &Path) -> ResponseEnvelope {
    let Some(pack) = forge::PackManifest::load_from_dir(&src) else {
        return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                format!("No valid pack.json found in {}", src.display()),
            ),
            0,
        );
    };

    let id = id_override.unwrap_or_else(|| pack.id.clone());
    let dest = forge::PackManifest::dir(root, &id);

    if let Err(e) = copy_dir_all(&src, &dest) {
        return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        );
    }

    let source_meta = forge::PackSource {
        kind: "local".to_string(),
        source: src.to_string_lossy().to_string(),
        ref_: String::new(),
        installed_at: chrono_now(),
    };
    let _ = source_meta.save(root, &id);

    ResponseEnvelope::success(
        "pattern install",
        serde_json::json!({
            "id": id,
            "version": pack.version,
            "source": "local",
            "path": dest.to_string_lossy(),
        }),
        0,
    )
}

fn pattern_install_github(source: &str, id_override: Option<String>, root: &Path) -> ResponseEnvelope {
    // Parse: github:user/repo[#sub/path[@ref]]
    let spec = source.trim_start_matches("github:");

    // Split #subpath first
    let (repo_and_ref, subpath_raw) = if let Some(hash) = spec.find('#') {
        (&spec[..hash], Some(&spec[hash + 1..]))
    } else {
        (spec, None)
    };

    // Split @ref from repo
    let (repo, git_ref) = if let Some(at) = repo_and_ref.rfind('@') {
        (&repo_and_ref[..at], &repo_and_ref[at + 1..])
    } else {
        (repo_and_ref, "HEAD")
    };

    // Split @ref from subpath if present
    let (subpath, git_ref) = match subpath_raw {
        Some(s) => {
            if let Some(at) = s.rfind('@') {
                (Some(&s[..at]), &s[at + 1..])
            } else {
                (Some(s), git_ref)
            }
        }
        None => (None, git_ref),
    };

    let tarball_url = format!("https://api.github.com/repos/{}/tarball/{}", repo, git_ref);

    // Download tarball into a temp dir
    let tmp_dir = match tempfile_dir() {
        Ok(d) => d,
        Err(e) => return ResponseEnvelope::error("pattern install", ErrorResponse::new(ErrorCode::InternalError, e), 0),
    };

    let tarball_bytes = match download_bytes(&tarball_url) {
        Ok(b) => b,
        Err(e) => return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, format!("Download failed: {e}")),
            0,
        ),
    };

    // Extract tarball
    let gz = flate2::read::GzDecoder::new(std::io::Cursor::new(&tarball_bytes));
    let mut archive = tar::Archive::new(gz);
    if let Err(e) = archive.unpack(&tmp_dir) {
        return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, format!("Extract failed: {e}")),
            0,
        );
    }

    // GitHub tarballs extract into a single top-level directory like `user-repo-<sha>/`
    let extracted_root = match std::fs::read_dir(&tmp_dir)
        .ok()
        .and_then(|mut e| e.next())
        .and_then(|e| e.ok())
        .map(|e| e.path())
    {
        Some(p) => p,
        None => return ResponseEnvelope::error(
            "pattern install",
            ErrorResponse::new(ErrorCode::InternalError, "Tarball appears empty"),
            0,
        ),
    };

    let pack_src = match subpath {
        Some(s) => extracted_root.join(s),
        None => extracted_root,
    };

    pattern_install_local(pack_src, id_override, root)
}

/// Validate a pack: check template files exist and render without errors.
pub fn pattern_lint(id: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let Some(pack) = forge::PackManifest::load(&cwd, &id) else {
        return ResponseEnvelope::error(
            "pattern lint",
            ErrorResponse::new(ErrorCode::ProjectNotFound, format!("Pack '{}' not found in .tsx/patterns/", id)),
            0,
        );
    };

    let pack_dir = forge::PackManifest::dir(&cwd, &id);
    let mut errors: Vec<String> = Vec::new();

    // 1. Check all template files exist on disk
    for output in &pack.outputs {
        if !pack_dir.join(&output.template).exists() {
            errors.push(format!("Template '{}' missing for output '{}'", output.template, output.id));
        }
    }

    // 2. Load engine and attempt render with dummy context
    let mut engine = forge::Engine::new();
    match engine.load_dir(&pack_dir) {
        Err(e) => errors.push(format!("Engine load error: {e}")),
        Ok(_) => {
            let mut ctx = forge::ForgeContext::new();
            for arg in &pack.args {
                ctx.insert_mut(&arg.name, &serde_json::Value::String(format!("dummy_{}", arg.name)));
            }
            for output in &pack.outputs {
                if let Err(e) = engine.render(&output.template, &ctx) {
                    errors.push(format!("Render error in '{}': {e}", output.template));
                }
            }
        }
    }

    // 3. Check marker files reference valid output paths (warn only)
    let mut warnings: Vec<String> = Vec::new();
    for marker in &pack.markers {
        let marker_path = cwd.join(&marker.file);
        if !marker_path.exists() {
            warnings.push(format!("Marker file '{}' not present in project (may be created later)", marker.file));
        }
    }

    if errors.is_empty() {
        ResponseEnvelope::success(
            "pattern lint",
            serde_json::json!({
                "id": id,
                "status": "ok",
                "warnings": warnings,
            }),
            0,
        )
    } else {
        ResponseEnvelope::error(
            "pattern lint",
            ErrorResponse::new(ErrorCode::ValidationError, errors.join("\n")),
            0,
        )
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

/// Recursively copy `src` directory into `dst`.
fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(src)?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Create a unique temporary directory under the system temp path.
fn tempfile_dir() -> Result<PathBuf, String> {
    let base = std::env::temp_dir().join(format!("tsx-install-{}", chrono_now()));
    std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    Ok(base)
}

/// Download URL to bytes using reqwest blocking.
fn download_bytes(url: &str) -> Result<Vec<u8>, String> {
    reqwest::blocking::Client::builder()
        .user_agent("tsx-cli/0.1")
        .build()
        .map_err(|e| e.to_string())?
        .get(url)
        .send()
        .map_err(|e| e.to_string())?
        .bytes()
        .map(|b| b.to_vec())
        .map_err(|e| e.to_string())
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
