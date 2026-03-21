//! PackageStore — discovers and loads installed tsx registry packages.
//!
//! Resolution order (highest → lowest priority):
//!   1. `.tsx/packages/<id>/`    — project-local pinned version
//!   2. `~/.tsx/packages/<id>/`  — global cache (shared across projects)
//!   3. `<exe-dir>/packages/<id>/` — bundled first-party packages shipped next to the binary

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tsx_shared::{PackageManifest, PackageSummary};

use crate::utils::paths::get_global_packages_dir;

pub struct PackageStore {
    /// Ordered list of directories to scan (high → low priority).
    roots: Vec<PathBuf>,
    /// Cache: package id → (manifest, install_path)
    cache: HashMap<String, (PackageManifest, PathBuf)>,
}

impl PackageStore {
    /// Build a store seeded from the standard resolution roots.
    pub fn new(project_root: Option<&Path>) -> Self {
        let mut roots = Vec::new();

        // 1. Project-local override
        if let Some(root) = project_root {
            roots.push(root.join(".tsx").join("packages"));
        } else if let Ok(cwd) = std::env::current_dir() {
            roots.push(cwd.join(".tsx").join("packages"));
        }

        // 2. Global user cache
        roots.push(get_global_packages_dir());

        // 3. Packages shipped next to the binary
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                roots.push(dir.join("packages"));
            }
        }

        // 4. Fallback: `frameworks/` next to binary or in cwd (legacy support)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                roots.push(dir.join("frameworks"));
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            roots.push(cwd.join("frameworks"));
        }

        let mut store = Self { roots, cache: HashMap::new() };
        store.scan_all();
        store
    }

    /// Scan all roots and populate the cache.
    fn scan_all(&mut self) {
        // Scan in reverse priority so higher-priority roots overwrite lower ones.
        let roots = self.roots.clone();
        for root in roots.iter().rev() {
            self.scan_dir(root);
        }
    }

    fn scan_dir(&mut self, dir: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let pkg_path = entry.path();
            if !pkg_path.is_dir() { continue; }
            if let Some(manifest) = load_manifest(&pkg_path) {
                self.cache.insert(manifest.id.clone(), (manifest, pkg_path));
            }
        }
    }

    /// List all installed packages.
    pub fn list(&self) -> Vec<PackageSummary> {
        self.cache
            .values()
            .map(|(m, path)| PackageSummary::from_manifest(m, path))
            .collect()
    }

    /// Get a package by id.
    pub fn get(&self, id: &str) -> Option<(&PackageManifest, &Path)> {
        self.cache.get(id).map(|(m, p)| (m, p.as_path()))
    }

    /// Find the package + generator spec that provides a command id.
    /// Returns `(manifest, install_path, generator_spec_json)`.
    pub fn resolve_command(
        &self,
        command_id: &str,
    ) -> Option<(&PackageManifest, &Path, serde_json::Value)> {
        for (manifest, pkg_path) in self.cache.values() {
            // Check manifest commands list
            let provides = manifest.commands.iter().any(|c| c.id == command_id);
            // Also check generators/*.json files for compatibility with old format
            let spec = load_generator_spec(pkg_path, command_id);
            if provides || spec.is_some() {
                let spec_val = spec.unwrap_or_else(|| {
                    // Build minimal spec from manifest command entry
                    if let Some(cmd) = manifest.commands.iter().find(|c| c.id == command_id) {
                        serde_json::json!({
                            "id": cmd.id,
                            "command": cmd.id,
                            "description": cmd.description,
                            "template": cmd.template,
                            "output_paths": [],
                            "next_steps": []
                        })
                    } else {
                        serde_json::Value::Null
                    }
                });
                if !spec_val.is_null() {
                    return Some((manifest, pkg_path.as_path(), spec_val));
                }
            }
        }
        None
    }

    /// All template directories from installed packages (used by the render engine).
    pub fn template_dirs(&self) -> Vec<PathBuf> {
        self.cache
            .values()
            .map(|(_, path)| path.join("templates"))
            .filter(|p| p.is_dir())
            .collect()
    }

    /// Find packages that claim any of the given npm package names.
    pub fn packages_for_npm(&self, npm_names: &[&str]) -> Vec<PackageSummary> {
        self.cache
            .values()
            .filter(|(m, _)| m.npm_packages.iter().any(|n| npm_names.contains(&n.as_str())))
            .map(|(m, path)| PackageSummary::from_manifest(m, path))
            .collect()
    }

    /// Install a package tarball that has already been extracted to `src_dir`
    /// into the global cache.  Returns the summary.
    pub fn install_from_dir(src_dir: &Path) -> anyhow::Result<PackageSummary> {
        let manifest = load_manifest(src_dir)
            .ok_or_else(|| anyhow::anyhow!("No manifest.json found in {}", src_dir.display()))?;

        let dest = get_global_packages_dir().join(&manifest.id);
        if dest != src_dir {
            if dest.exists() {
                std::fs::remove_dir_all(&dest)?;
            }
            copy_dir_all(src_dir, &dest)?;
        }

        Ok(PackageSummary::from_manifest(&manifest, &dest))
    }
}

impl Default for PackageStore {
    fn default() -> Self {
        Self::new(None)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Load `manifest.json` from a package directory.
/// Supports both the new format and the legacy `manifest.json` from frameworks/.
pub fn load_manifest(pkg_dir: &Path) -> Option<PackageManifest> {
    let path = pkg_dir.join("manifest.json");
    let content = std::fs::read_to_string(&path).ok()?;
    // Try new PackageManifest format first
    if let Ok(m) = serde_json::from_str::<PackageManifest>(&content) {
        if !m.id.is_empty() {
            return Some(m);
        }
    }
    // Legacy: frameworks used a different manifest shape — synthesize a PackageManifest
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    let id = val.get("id").and_then(|v| v.as_str())?.to_string();
    let name = val.get("name").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
    let version = val.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();
    let description = val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let category = val.get("category").and_then(|v| v.as_str()).unwrap_or("framework").to_string();
    let docs = val.get("docs").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let github = val.get("github").and_then(|v| v.as_str()).map(|s| s.to_string());

    // Build npm_packages from peer_dependencies keys (legacy)
    let npm_packages: Vec<String> = val
        .get("peer_dependencies")
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    // Build commands from provides list
    let commands: Vec<tsx_shared::CommandEntry> = val
        .get("provides")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|cmd| tsx_shared::CommandEntry {
                    id: cmd.to_string(),
                    description: cmd.to_string(),
                    template: String::new(),
                })
                .collect()
        })
        .unwrap_or_default();

    Some(PackageManifest {
        id,
        name,
        version,
        description,
        category,
        docs,
        github,
        npm_packages,
        commands,
        ..Default::default()
    })
}

/// Load a generator spec from `<pkg_dir>/generators/<command_id>.json`.
/// Tries the command id directly (e.g., `add:schema` → `add:schema.json`)
/// and also with colon replaced by hyphen (`add-schema.json`).
fn load_generator_spec(pkg_dir: &Path, command_id: &str) -> Option<serde_json::Value> {
    let gen_dir = pkg_dir.join("generators");
    let candidates = [
        gen_dir.join(format!("{}.json", command_id)),
        gen_dir.join(format!("{}.json", command_id.replace(':', "-"))),
        gen_dir.join(format!("{}.json", command_id.replace("add:", "add-"))),
    ];
    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                return Some(val);
            }
        }
    }
    None
}

fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)?.flatten() {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
