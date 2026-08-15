use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// An entry in `.tsx/packages.json`, the index of packages installed from the tsx registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct InstalledPkg {
    pub name: String,
    pub version: String,
    pub registry_url: String,
    pub installed_at: String,
}

fn pkg_index_path(root: &Path) -> PathBuf {
    root.join(".tsx").join("packages.json")
}

pub(super) fn load_pkg_index(root: &Path) -> Vec<InstalledPkg> {
    let path = pkg_index_path(root);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub(super) fn save_pkg_index(root: &Path, pkgs: &[InstalledPkg]) -> anyhow::Result<()> {
    let path = pkg_index_path(root);
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(&path, serde_json::to_string_pretty(pkgs)?)?;
    Ok(())
}
