//! **E5 — Multi-file atomic transactions** (`GeneratorPlan`).
//!
//! Generators declare their output files upfront; the engine then writes all
//! or nothing.  If any write fails the already-written files are rolled back.
//!
//! ```rust
//! use forge::plan::{GeneratorPlan, OverwritePolicy};
//!
//! let plan = GeneratorPlan::new("add-schema")
//!     .writes("app/db/schema/users.ts")
//!     .writes_optional("app/db/schema/index.ts")
//!     .conflicts_if_exists("app/db/schema/users.ts");
//!
//! let outputs = [
//!     ("app/db/schema/users.ts", "export const usersTable = ..."),
//!     ("app/db/schema/index.ts", "export * from './users'"),
//! ];
//!
//! plan.execute(&outputs, OverwritePolicy::Skip).unwrap();
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Overwrite policy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverwritePolicy {
    /// Skip files that already exist (default, safest)
    Skip,
    /// Overwrite any existing file
    Overwrite,
    /// Fail if any declared output already exists and `conflicts_if_exists` was set
    Fail,
}

// ---------------------------------------------------------------------------
// Planned output entry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PlannedOutput {
    pub path: String,
    pub required: bool,
    pub conflicts_if_exists: bool,
}

// ---------------------------------------------------------------------------
// GeneratorPlan
// ---------------------------------------------------------------------------

/// Declares the intent of a generator before execution.
///
/// Enables atomic writes (all or nothing) and diff preview.
#[derive(Debug, Clone)]
pub struct GeneratorPlan {
    pub generator_id: String,
    pub outputs: Vec<PlannedOutput>,
}

impl GeneratorPlan {
    pub fn new(generator_id: impl Into<String>) -> Self {
        Self { generator_id: generator_id.into(), outputs: Vec::new() }
    }

    /// Declare a required output file.
    pub fn writes(mut self, path: impl Into<String>) -> Self {
        self.outputs.push(PlannedOutput {
            path: path.into(),
            required: true,
            conflicts_if_exists: false,
        });
        self
    }

    /// Declare an optional output file (skipped without error if rendering fails).
    pub fn writes_optional(mut self, path: impl Into<String>) -> Self {
        self.outputs.push(PlannedOutput {
            path: path.into(),
            required: false,
            conflicts_if_exists: false,
        });
        self
    }

    /// Mark a path as conflicting if it already exists (respects `OverwritePolicy::Fail`).
    pub fn conflicts_if_exists(mut self, path: impl Into<String>) -> Self {
        let p = path.into();
        for out in &mut self.outputs {
            if out.path == p {
                out.conflicts_if_exists = true;
                return self;
            }
        }
        // Add implicitly if not already declared
        self.outputs.push(PlannedOutput {
            path: p,
            required: true,
            conflicts_if_exists: true,
        });
        self
    }

    // ---------------------------------------------------------------------------
    // Pre-flight checks
    // ---------------------------------------------------------------------------

    /// Check for conflicts before writing anything.
    /// Returns `Err` listing paths that would be overwritten under `OverwritePolicy::Fail`.
    pub fn check_conflicts(
        &self,
        root: &Path,
        policy: &OverwritePolicy,
    ) -> Result<(), PlanError> {
        if *policy != OverwritePolicy::Fail {
            return Ok(());
        }
        let conflicts: Vec<String> = self
            .outputs
            .iter()
            .filter(|o| o.conflicts_if_exists)
            .filter(|o| root.join(&o.path).exists())
            .map(|o| o.path.clone())
            .collect();

        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(PlanError::Conflicts(conflicts))
        }
    }

    // ---------------------------------------------------------------------------
    // Diff preview
    // ---------------------------------------------------------------------------

    /// Compute a unified diff for each output without writing any files.
    /// Returns a map of `path → diff_string`.
    pub fn diff(
        &self,
        root: &Path,
        outputs: &[(&str, &str)],
    ) -> HashMap<String, String> {
        let content_map: HashMap<&str, &str> = outputs.iter().copied().collect();
        let mut result = HashMap::new();

        for planned in &self.outputs {
            let new_content = match content_map.get(planned.path.as_str()) {
                Some(c) => *c,
                None => continue,
            };
            let full_path = root.join(&planned.path);
            let old_content = std::fs::read_to_string(&full_path).unwrap_or_default();
            let diff = unified_diff(&planned.path, &old_content, new_content);
            result.insert(planned.path.clone(), diff);
        }
        result
    }

    // ---------------------------------------------------------------------------
    // Atomic execution
    // ---------------------------------------------------------------------------

    /// Execute the plan: write all outputs atomically.
    ///
    /// On partial failure, rolls back any files that were already written in
    /// this transaction.
    pub fn execute(
        &self,
        root: &Path,
        outputs: &[(&str, &str)],
        policy: OverwritePolicy,
    ) -> Result<PlanResult, PlanError> {
        self.check_conflicts(root, &policy)?;

        let content_map: HashMap<&str, &str> = outputs.iter().copied().collect();
        let mut written: Vec<PathBuf> = Vec::new();
        let mut skipped: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        for planned in &self.outputs {
            let new_content = match content_map.get(planned.path.as_str()) {
                Some(c) => *c,
                None => {
                    if planned.required {
                        // Rollback and fail
                        rollback(&written);
                        return Err(PlanError::MissingOutput(planned.path.clone()));
                    }
                    continue;
                }
            };

            let full_path = root.join(&planned.path);

            // Ensure parent directory exists
            if let Some(parent) = full_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    rollback(&written);
                    return Err(PlanError::Io(planned.path.clone(), e.to_string()));
                }
            }

            // Check existence
            if full_path.exists() {
                match policy {
                    OverwritePolicy::Skip => {
                        skipped.push(planned.path.clone());
                        continue;
                    }
                    OverwritePolicy::Fail => {
                        if planned.conflicts_if_exists {
                            rollback(&written);
                            return Err(PlanError::Conflicts(vec![planned.path.clone()]));
                        }
                    }
                    OverwritePolicy::Overwrite => {}
                }
            }

            match std::fs::write(&full_path, new_content) {
                Ok(()) => written.push(full_path),
                Err(e) => {
                    rollback(&written);
                    return Err(PlanError::Io(planned.path.clone(), e.to_string()));
                }
            }
        }

        Ok(PlanResult {
            generator_id: self.generator_id.clone(),
            written: written.iter().map(|p| p.to_string_lossy().to_string()).collect(),
            skipped,
            warnings,
        })
    }
}

// ---------------------------------------------------------------------------
// Result / Error types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PlanResult {
    pub generator_id: String,
    pub written: Vec<String>,
    pub skipped: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub enum PlanError {
    Conflicts(Vec<String>),
    MissingOutput(String),
    Io(String, String),
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::Conflicts(paths) => write!(f, "Conflicts: {}", paths.join(", ")),
            PlanError::MissingOutput(p) => write!(f, "Missing output: {}", p),
            PlanError::Io(p, e) => write!(f, "IO error writing {}: {}", p, e),
        }
    }
}

impl std::error::Error for PlanError {}

// ---------------------------------------------------------------------------
// Rollback
// ---------------------------------------------------------------------------

fn rollback(written: &[PathBuf]) {
    for path in written.iter().rev() {
        let _ = std::fs::remove_file(path);
    }
}

// ---------------------------------------------------------------------------
// Unified diff helper
// ---------------------------------------------------------------------------

pub fn unified_diff(path: &str, old: &str, new: &str) -> String {
    if old == new {
        return format!("--- {}\n(no changes)\n", path);
    }

    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let mut out = format!("--- a/{}\n+++ b/{}\n", path, path);
    let max = old_lines.len().max(new_lines.len());

    let mut chunk_start: Option<usize> = None;
    let mut chunk: Vec<String> = Vec::new();

    for i in 0..max {
        let ol = old_lines.get(i).copied();
        let nl = new_lines.get(i).copied();
        if ol != nl {
            chunk_start.get_or_insert(i + 1);
            if let Some(l) = ol { chunk.push(format!("-{}", l)); }
            if let Some(l) = nl { chunk.push(format!("+{}", l)); }
        } else if !chunk.is_empty() {
            out.push_str(&format!("@@ -{} +{} @@\n", chunk_start.unwrap_or(1), chunk_start.unwrap_or(1)));
            for line in &chunk { out.push_str(line); out.push('\n'); }
            chunk.clear();
            chunk_start = None;
        }
    }
    if !chunk.is_empty() {
        out.push_str(&format!("@@ -{} +{} @@\n", chunk_start.unwrap_or(1), chunk_start.unwrap_or(1)));
        for line in &chunk { out.push_str(line); out.push('\n'); }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn writes_all_files_atomically() {
        let dir = TempDir::new().unwrap();
        let plan = GeneratorPlan::new("test-gen")
            .writes("a.ts")
            .writes("b.ts");

        let outputs = [("a.ts", "const a = 1;"), ("b.ts", "const b = 2;")];
        let result = plan.execute(dir.path(), &outputs, OverwritePolicy::Skip).unwrap();

        assert_eq!(result.written.len(), 2);
        assert!(dir.path().join("a.ts").exists());
        assert!(dir.path().join("b.ts").exists());
    }

    #[test]
    fn skips_existing_files_under_skip_policy() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.ts"), "existing").unwrap();

        let plan = GeneratorPlan::new("test-gen").writes("a.ts");
        let outputs = [("a.ts", "new content")];
        let result = plan.execute(dir.path(), &outputs, OverwritePolicy::Skip).unwrap();

        assert_eq!(result.skipped.len(), 1);
        assert_eq!(std::fs::read_to_string(dir.path().join("a.ts")).unwrap(), "existing");
    }

    #[test]
    fn overwrites_under_overwrite_policy() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.ts"), "old").unwrap();

        let plan = GeneratorPlan::new("test-gen").writes("a.ts");
        let outputs = [("a.ts", "new")];
        plan.execute(dir.path(), &outputs, OverwritePolicy::Overwrite).unwrap();

        assert_eq!(std::fs::read_to_string(dir.path().join("a.ts")).unwrap(), "new");
    }

    #[test]
    fn conflicts_if_exists_fails_under_fail_policy() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.ts"), "existing").unwrap();

        let plan = GeneratorPlan::new("test-gen")
            .writes("a.ts")
            .conflicts_if_exists("a.ts");
        let outputs = [("a.ts", "new")];
        let result = plan.execute(dir.path(), &outputs, OverwritePolicy::Fail);
        assert!(matches!(result, Err(PlanError::Conflicts(_))));
    }

    #[test]
    fn diff_shows_changes() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.ts"), "old line").unwrap();

        let plan = GeneratorPlan::new("test-gen").writes("a.ts");
        let diffs = plan.diff(dir.path(), &[("a.ts", "new line")]);
        assert!(diffs["a.ts"].contains("+new line"));
        assert!(diffs["a.ts"].contains("-old line"));
    }
}
