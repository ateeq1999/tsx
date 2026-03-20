use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::find_project_root;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomVersion {
    pub name: String,
    pub current: String,
    pub latest: String,
    pub breaking: bool,
    pub status: UpgradeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpgradeStatus {
    UpToDate,
    UpdateAvailable,
    BreakingChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeResult {
    pub atoms_version: String,
    pub project_pinned_version: Option<String>,
    pub atoms: Vec<AtomVersion>,
    pub migration_guide: Option<String>,
}

pub fn upgrade(check_only: bool, verbose: bool) -> CommandResult {
    let start = Instant::now();

    // Load the bundled metadata to get the current atom versions.
    let metadata_str = include_str!("../../../../../templates/metadata.json");
    let metadata: serde_json::Value = match serde_json::from_str(metadata_str) {
        Ok(v) => v,
        Err(e) => {
            return CommandResult::err("upgrade", format!("Failed to parse metadata.json: {}", e))
        }
    };

    let atoms_version = metadata
        .get("atoms")
        .and_then(|a| a.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Check if the project has a pinned version in package.json.
    let project_pinned = find_project_root().ok().and_then(|root| {
        let pkg = std::fs::read_to_string(root.join("package.json")).ok()?;
        let v: serde_json::Value = serde_json::from_str(&pkg).ok()?;
        v.get("tsx")
            .and_then(|t| t.get("atomsVersion"))
            .and_then(|av| av.as_str())
            .map(|s| s.to_string())
    });

    // Build the atom status list from metadata.
    let atom_entries = metadata
        .get("atoms")
        .and_then(|a| a.get("entries"))
        .and_then(|e| e.as_object())
        .map(|entries| {
            entries
                .iter()
                .map(|(name, info)| {
                    let current_ver = project_pinned.clone().unwrap_or_else(|| atoms_version.clone());
                    let latest_ver = info
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("1.0.0")
                        .to_string();
                    let breaking = info
                        .get("breaking")
                        .and_then(|b| b.as_bool())
                        .unwrap_or(false);

                    let status = if current_ver == latest_ver {
                        UpgradeStatus::UpToDate
                    } else if breaking {
                        UpgradeStatus::BreakingChange
                    } else {
                        UpgradeStatus::UpdateAvailable
                    };

                    AtomVersion {
                        name: name.clone(),
                        current: current_ver,
                        latest: latest_ver,
                        breaking,
                        status,
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let has_breaking = atom_entries
        .iter()
        .any(|a| matches!(a.status, UpgradeStatus::BreakingChange));

    let migration_guide = if has_breaking {
        Some(
            "Breaking changes detected. Review the changelog in templates/metadata.json and update generated files manually before upgrading.".to_string()
        )
    } else {
        None
    };

    // Write the pinned version and extract templates if not check_only.
    if !check_only {
        if let Ok(root) = find_project_root() {
            // Pin version in package.json
            let pkg_path = root.join("package.json");
            if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                if let Ok(mut pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(obj) = pkg.as_object_mut() {
                        obj.entry("tsx")
                            .or_insert_with(|| serde_json::json!({}))
                            .as_object_mut()
                            .map(|t| {
                                t.insert(
                                    "atomsVersion".to_string(),
                                    serde_json::json!(atoms_version),
                                )
                            });

                        if let Ok(updated) = serde_json::to_string_pretty(&pkg) {
                            let _ = std::fs::write(&pkg_path, updated);
                        }
                    }
                }
            }

            // Extract embedded templates to .tsx/templates/ so the project has
            // local copies of the current atom templates.
            let tsx_templates_dir = root.join(".tsx").join("templates");
            let embedded = crate::render::embedded::get_embedded_templates();
            for (name, content) in &embedded {
                let dest = tsx_templates_dir.join(name);
                if let Some(parent) = dest.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&dest, content);
            }
        }
    }

    let result = UpgradeResult {
        atoms_version: atoms_version.clone(),
        project_pinned_version: project_pinned,
        atoms: atom_entries,
        migration_guide,
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    let response = ResponseEnvelope::success(
        "upgrade",
        serde_json::to_value(result).unwrap(),
        duration_ms,
    );

    if verbose {
        let context = crate::json::response::Context {
            project_root: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        response.with_context(context).print();
    } else {
        response.print();
    }

    let mut cmd_result = CommandResult::ok("upgrade", vec![]);
    if check_only {
        cmd_result.next_steps = vec![
            "Run 'tsx upgrade' (without --check) to pin the current version.".to_string(),
        ];
    } else {
        cmd_result.next_steps = vec![
            format!(
                "Atoms pinned to {} in package.json and templates written to .tsx/templates/.",
                atoms_version
            ),
        ];
    }
    cmd_result
}
