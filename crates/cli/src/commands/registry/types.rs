use std::path::{Path, PathBuf};

/// Metadata stored in `.tsx/registries.json` tracking installed community registries.
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct InstalledRegistry {
    pub slug: String,
    pub package: String,
    pub version: String,
    pub source: String,
    pub installed_at: String,
}

pub(super) fn registries_index_path(root: &Path) -> PathBuf {
    root.join(".tsx").join("registries.json")
}

pub(super) fn load_registries_index(root: &Path) -> Vec<InstalledRegistry> {
    let path = registries_index_path(root);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub(super) fn save_registries_index(root: &Path, registries: &[InstalledRegistry]) -> anyhow::Result<()> {
    let path = registries_index_path(root);
    std::fs::create_dir_all(path.parent().unwrap())?;
    let content = serde_json::to_string_pretty(registries)?;
    std::fs::write(&path, content)?;
    Ok(())
}
