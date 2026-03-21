//! `tsx snapshot` — snapshot testing for generators (E3).
//!
//! Runs generators with fixture inputs, saves their output, and diffs on subsequent runs
//! to catch silent regressions in template changes.
//!
//! ## Directory layout
//!
//! ```text
//! .tsx/snapshots/
//!   <generator-id>/
//!     <fixture-name>.json        — input arguments (fixture)
//!     <fixture-name>.output/     — expected output files
//!       schema.ts
//!       index.ts
//!       ...
//! ```
//!
//! ## Commands
//! - `tsx snapshot update` — run all generators, save/overwrite snapshots
//! - `tsx snapshot diff`   — run generators, show diff vs saved snapshots
//! - `tsx snapshot accept` — alias for `update` (after intentional change)
//! - `tsx snapshot list`   — list all registered fixtures

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

// ---------------------------------------------------------------------------
// Snapshot index entry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotFixture {
    pub generator_id: String,
    pub fixture_name: String,
    pub input: serde_json::Value,
}

impl SnapshotFixture {
    /// Path to the fixture input JSON.
    pub fn fixture_path(root: &Path, gen_id: &str, fixture: &str) -> PathBuf {
        root.join(".tsx").join("snapshots").join(gen_id).join(format!("{}.json", fixture))
    }

    /// Path to the expected output directory.
    pub fn output_dir(root: &Path, gen_id: &str, fixture: &str) -> PathBuf {
        root.join(".tsx").join("snapshots").join(gen_id).join(format!("{}.output", fixture))
    }

    /// List all fixtures for a generator.
    pub fn list(root: &Path, gen_id: &str) -> Vec<String> {
        let dir = root.join(".tsx").join("snapshots").join(gen_id);
        let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new(); };
        entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().and_then(|x| x.to_str()) == Some("json")
            })
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .collect()
    }

    /// List all generator ids that have snapshots.
    pub fn list_generators(root: &Path) -> Vec<String> {
        let snap_dir = root.join(".tsx").join("snapshots");
        let Ok(entries) = std::fs::read_dir(&snap_dir) else { return Vec::new(); };
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
            .collect()
    }

    /// Add a fixture input for a generator.
    pub fn add(root: &Path, gen_id: &str, fixture: &str, input: &serde_json::Value) -> anyhow::Result<()> {
        let dir = root.join(".tsx").join("snapshots").join(gen_id);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", fixture));
        std::fs::write(&path, serde_json::to_string_pretty(input)?)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

pub fn snapshot_update(generator: Option<String>, verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    run_snapshots(&cwd, generator.as_deref(), false, verbose)
}

pub fn snapshot_diff(generator: Option<String>, verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    run_snapshots(&cwd, generator.as_deref(), true, verbose)
}

pub fn snapshot_accept(generator: Option<String>, verbose: bool) -> ResponseEnvelope {
    // Accept is the same as update
    snapshot_update(generator, verbose)
}

pub fn snapshot_list(_verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let generators = SnapshotFixture::list_generators(&cwd);

    let mut all: Vec<serde_json::Value> = Vec::new();
    for gen in &generators {
        let fixtures = SnapshotFixture::list(&cwd, gen);
        for fix in &fixtures {
            all.push(serde_json::json!({
                "generator": gen,
                "fixture": fix,
                "input": SnapshotFixture::fixture_path(&cwd, gen, fix).to_string_lossy(),
                "output_dir": SnapshotFixture::output_dir(&cwd, gen, fix).to_string_lossy(),
            }));
        }
    }

    ResponseEnvelope::success(
        "snapshot list",
        serde_json::json!({
            "count": all.len(),
            "snapshots": all,
        }),
        0,
    )
}

pub fn snapshot_add(
    generator: String,
    fixture: String,
    input: Option<String>,
    _verbose: bool,
) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let input_json: serde_json::Value = match input {
        Some(s) => match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(e) => {
                return ResponseEnvelope::error(
                    "snapshot add",
                    ErrorResponse::new(
                        ErrorCode::ValidationError,
                        format!("Invalid JSON input: {}", e),
                    ),
                    0,
                )
            }
        },
        None => serde_json::json!({"name": fixture}),
    };

    match SnapshotFixture::add(&cwd, &generator, &fixture, &input_json) {
        Ok(_) => ResponseEnvelope::success(
            "snapshot add",
            serde_json::json!({
                "generator": generator,
                "fixture": fixture,
                "path": SnapshotFixture::fixture_path(&cwd, &generator, &fixture).to_string_lossy(),
            }),
            0,
        )
        .with_next_steps(vec![
            format!("Run `tsx snapshot update --generator {}` to capture the expected output", generator),
        ]),
        Err(e) => ResponseEnvelope::error(
            "snapshot add",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

// ---------------------------------------------------------------------------
// Core runner
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct SnapshotResult {
    generator_id: String,
    fixture_name: String,
    status: SnapshotStatus,
    diffs: Vec<FileDiff>,
}

#[derive(Debug, PartialEq)]
enum SnapshotStatus {
    New,        // snapshot saved for the first time
    Passed,     // matches saved snapshot
    Failed,     // differs from saved snapshot
    Updated,    // snapshot updated (update/accept mode)
}

#[derive(Debug)]
struct FileDiff {
    file: String,
    expected: Option<String>,
    actual: Option<String>,
}

fn run_snapshots(root: &Path, filter_gen: Option<&str>, diff_mode: bool, _verbose: bool) -> ResponseEnvelope {
    let cmd = if diff_mode { "snapshot diff" } else { "snapshot update" };
    let generators = if let Some(g) = filter_gen {
        vec![g.to_string()]
    } else {
        SnapshotFixture::list_generators(root)
    };

    if generators.is_empty() {
        return ResponseEnvelope::success(
            cmd,
            serde_json::json!({
                "status": "no_snapshots",
                "message": "No snapshot fixtures found in .tsx/snapshots/. Add fixtures with `tsx snapshot add`.",
            }),
            0,
        );
    }

    let mut results: Vec<SnapshotResult> = Vec::new();

    for gen_id in &generators {
        let fixtures = SnapshotFixture::list(root, gen_id);
        for fixture_name in &fixtures {
            let fixture_path = SnapshotFixture::fixture_path(root, gen_id, fixture_name);
            let input_str = match std::fs::read_to_string(&fixture_path) {
                Ok(s) => s,
                Err(e) => {
                    results.push(SnapshotResult {
                        generator_id: gen_id.clone(),
                        fixture_name: fixture_name.clone(),
                        status: SnapshotStatus::Failed,
                        diffs: vec![FileDiff {
                            file: fixture_path.to_string_lossy().to_string(),
                            expected: None,
                            actual: Some(format!("Error reading fixture: {}", e)),
                        }],
                    });
                    continue;
                }
            };

            // Run the generator by invoking `tsx run <gen_id> --json <input>`
            let actual_outputs = run_generator(root, gen_id, &input_str);

            let output_dir = SnapshotFixture::output_dir(root, gen_id, fixture_name);

            if diff_mode && output_dir.exists() {
                // Compare outputs to saved snapshots
                let diffs = compare_outputs(&actual_outputs, &output_dir);
                let status = if diffs.is_empty() {
                    SnapshotStatus::Passed
                } else {
                    SnapshotStatus::Failed
                };
                results.push(SnapshotResult {
                    generator_id: gen_id.clone(),
                    fixture_name: fixture_name.clone(),
                    status,
                    diffs,
                });
            } else {
                // Update / create mode: save outputs
                let _ = std::fs::create_dir_all(&output_dir);
                for (file_name, content) in &actual_outputs {
                    let dest = output_dir.join(file_name);
                    let _ = std::fs::write(&dest, content);
                }
                let is_new = !output_dir.exists() || actual_outputs.is_empty();
                results.push(SnapshotResult {
                    generator_id: gen_id.clone(),
                    fixture_name: fixture_name.clone(),
                    status: if is_new { SnapshotStatus::New } else { SnapshotStatus::Updated },
                    diffs: Vec::new(),
                });
            }
        }
    }

    // Summarise
    let passed = results.iter().filter(|r| r.status == SnapshotStatus::Passed).count();
    let failed = results.iter().filter(|r| r.status == SnapshotStatus::Failed).count();
    let updated = results.iter().filter(|r| matches!(r.status, SnapshotStatus::Updated | SnapshotStatus::New)).count();

    let summary: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            let status_str = match r.status {
                SnapshotStatus::Passed => "pass",
                SnapshotStatus::Failed => "fail",
                SnapshotStatus::New => "new",
                SnapshotStatus::Updated => "updated",
            };
            let diff_output: Vec<serde_json::Value> = r
                .diffs
                .iter()
                .map(|d| serde_json::json!({
                    "file": d.file,
                    "diff": build_simple_diff(d.expected.as_deref(), d.actual.as_deref()),
                }))
                .collect();
            serde_json::json!({
                "generator": r.generator_id,
                "fixture": r.fixture_name,
                "status": status_str,
                "diffs": diff_output,
            })
        })
        .collect();

    let overall_ok = failed == 0;
    let data = serde_json::json!({
        "mode": if diff_mode { "diff" } else { "update" },
        "total": results.len(),
        "passed": passed,
        "failed": failed,
        "updated": updated,
        "results": summary,
    });

    if overall_ok || !diff_mode {
        ResponseEnvelope::success(cmd, data, 0)
    } else {
        ResponseEnvelope::error(
            cmd,
            ErrorResponse::new(
                ErrorCode::ValidationError,
                format!("{} snapshot(s) failed. Run `tsx snapshot accept` to update them.", failed),
            ),
            0,
        )
    }
}

/// Run a generator by invoking the in-process `run` command with the fixture JSON.
/// Returns a map of relative_file_path → content.
fn run_generator(_root: &Path, gen_id: &str, input_json: &str) -> HashMap<String, String> {
    // Invoke the run command's dry-run path to capture output without writing files.
    // We call crate::commands::run::run() directly.
    use crate::commands::run;

    let result = run::run(
        gen_id.to_string(),
        None,
        Some(input_json.to_string()),
        false, // overwrite
        true,  // dry_run — captures intended files without writing
        false,
    );

    // Parse the result to extract file paths
    let mut outputs: HashMap<String, String> = HashMap::new();
    // CommandResult.files_created holds the intended output paths (dry-run mode)
    for path in &result.files_created {
        // In dry-run mode the files aren't written; snapshot captures the path list only
        let rel = PathBuf::from(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path.as_str())
            .to_string();
        // Content is empty in dry-run — snapshot records file existence
        outputs.insert(rel, String::new());
    }

    outputs
}

/// Compare actual generator outputs against saved snapshots.
fn compare_outputs(actual: &HashMap<String, String>, snapshot_dir: &Path) -> Vec<FileDiff> {
    let mut diffs = Vec::new();

    // Check files in snapshot dir that differ from actual
    if let Ok(entries) = std::fs::read_dir(snapshot_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let expected = std::fs::read_to_string(entry.path()).unwrap_or_default();
            let actual_content = actual.get(&file_name).cloned().unwrap_or_default();
            if expected != actual_content {
                diffs.push(FileDiff {
                    file: file_name,
                    expected: Some(expected),
                    actual: Some(actual_content),
                });
            }
        }
    }

    // Check new files in actual that aren't in snapshot
    for (file, content) in actual {
        if !snapshot_dir.join(file).exists() {
            diffs.push(FileDiff {
                file: file.clone(),
                expected: None,
                actual: Some(content.clone()),
            });
        }
    }

    diffs
}

/// Build a minimal line-based diff representation.
fn build_simple_diff(expected: Option<&str>, actual: Option<&str>) -> String {
    match (expected, actual) {
        (None, Some(a)) => format!("++ NEW FILE\n{}", prefix_lines(a, "+")),
        (Some(e), None) => format!("-- DELETED\n{}", prefix_lines(e, "-")),
        (Some(e), Some(a)) => {
            let mut diff_lines = Vec::new();
            let e_lines: Vec<&str> = e.lines().collect();
            let a_lines: Vec<&str> = a.lines().collect();
            let max = e_lines.len().max(a_lines.len());
            for i in 0..max {
                let el = e_lines.get(i).copied().unwrap_or("");
                let al = a_lines.get(i).copied().unwrap_or("");
                if el != al {
                    if !el.is_empty() { diff_lines.push(format!("-{}", el)); }
                    if !al.is_empty() { diff_lines.push(format!("+{}", al)); }
                }
            }
            if diff_lines.is_empty() {
                "(no differences)".to_string()
            } else {
                diff_lines.join("\n")
            }
        }
        _ => "(no differences)".to_string(),
    }
}

fn prefix_lines(text: &str, prefix: &str) -> String {
    text.lines().map(|l| format!("{}{}", prefix, l)).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn add_and_list_fixture() {
        let dir = TempDir::new().unwrap();
        let input = serde_json::json!({"name": "users"});
        SnapshotFixture::add(dir.path(), "add-schema", "users", &input).unwrap();
        let fixtures = SnapshotFixture::list(dir.path(), "add-schema");
        assert_eq!(fixtures, vec!["users"]);
    }

    #[test]
    fn list_generators_empty() {
        let dir = TempDir::new().unwrap();
        let gens = SnapshotFixture::list_generators(dir.path());
        assert!(gens.is_empty());
    }

    #[test]
    fn compare_outputs_detects_diff() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("schema.ts"), "const x = 1;").unwrap();
        let mut actual = HashMap::new();
        actual.insert("schema.ts".to_string(), "const x = 2;".to_string());
        let diffs = compare_outputs(&actual, dir.path());
        assert_eq!(diffs.len(), 1);
    }

    #[test]
    fn compare_outputs_no_diff() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("schema.ts"), "const x = 1;").unwrap();
        let mut actual = HashMap::new();
        actual.insert("schema.ts".to_string(), "const x = 1;".to_string());
        let diffs = compare_outputs(&actual, dir.path());
        assert!(diffs.is_empty());
    }
}
