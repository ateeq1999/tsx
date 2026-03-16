use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::plugin::manifest::PluginManifest;
use crate::plugin::validate_plugin;
use crate::utils::paths::find_project_root;

/// The project-local plugin registry.
///
/// Plugins are installed into `<project_root>/.tsx/plugins/<package-name>/`.
/// Each plugin directory must contain a `plugin.json` manifest and a `templates/` folder.
pub struct PluginRegistry {
    plugins_dir: PathBuf,
}

impl PluginRegistry {
    /// Open the plugin registry for the current project.
    pub fn open() -> Result<Self> {
        let root = find_project_root()?;
        let plugins_dir = root.join(".tsx").join("plugins");
        std::fs::create_dir_all(&plugins_dir)?;
        Ok(PluginRegistry { plugins_dir })
    }

    /// Open the plugin registry for a specific root (used in tests).
    pub fn open_at(root: &Path) -> Result<Self> {
        let plugins_dir = root.join(".tsx").join("plugins");
        std::fs::create_dir_all(&plugins_dir)?;
        Ok(PluginRegistry { plugins_dir })
    }

    /// List all installed plugins.
    pub fn list(&self) -> Vec<PluginManifest> {
        let Ok(entries) = std::fs::read_dir(&self.plugins_dir) else {
            return vec![];
        };
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| PluginManifest::load(&e.path()).ok())
            .collect()
    }

    /// Install a plugin from a local directory (validates before installing).
    pub fn install_from_dir(&self, source: &Path) -> Result<PluginManifest> {
        let manifest = validate_plugin(source).map_err(|errs| {
            let msgs: Vec<String> = errs.iter().map(|e| format!("{}: {}", e.field, e.message)).collect();
            anyhow::anyhow!("Plugin validation failed:\n{}", msgs.join("\n"))
        })?;

        let dest = self.plugins_dir.join(&manifest.package);
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }
        copy_dir_all(source, &dest)?;
        Ok(manifest)
    }

    /// Install a plugin from npm (downloads via `npm pack` then extracts).
    pub fn install_from_npm(&self, package: &str) -> Result<PluginManifest> {
        let tmp = tempdir()?;

        // npm pack downloads the tarball
        let status = std::process::Command::new("npm")
            .args(["pack", package, "--pack-destination", &tmp.to_string_lossy()])
            .status()?;

        if !status.success() {
            anyhow::bail!("npm pack failed for package '{}'", package);
        }

        // Find the downloaded .tgz
        let tgz = std::fs::read_dir(&tmp)?
            .filter_map(|e| e.ok())
            .find(|e| {
                e.path()
                    .extension()
                    .map(|x| x == "tgz")
                    .unwrap_or(false)
            })
            .ok_or_else(|| anyhow::anyhow!("No .tgz found after npm pack"))?;

        // Extract with tar
        let extract_dir = tmp.join("extracted");
        std::fs::create_dir_all(&extract_dir)?;
        let tar_status = std::process::Command::new("tar")
            .args(["-xzf", &tgz.path().to_string_lossy(), "-C", &extract_dir.to_string_lossy(), "--strip-components=1"])
            .status()?;

        if !tar_status.success() {
            anyhow::bail!("Failed to extract plugin tarball");
        }

        self.install_from_dir(&extract_dir)
    }

    /// Remove an installed plugin by package name.
    pub fn remove(&self, package: &str) -> Result<()> {
        let dir = self.plugins_dir.join(package);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' is not installed", package)
        }
    }

    /// Get the templates directory for a specific installed plugin.
    pub fn plugin_templates_dir(&self, package: &str) -> Option<PathBuf> {
        let dir = self.plugins_dir.join(package).join("templates");
        dir.is_dir().then_some(dir)
    }

    /// Return the plugin manifest for a package if installed.
    pub fn get(&self, package: &str) -> Option<PluginManifest> {
        PluginManifest::load(&self.plugins_dir.join(package)).ok()
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn tempdir() -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("tsx-plugin-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::manifest::PluginManifest;
    use tempfile::TempDir;

    fn make_plugin_dir(dir: &Path, package: &str) -> PathBuf {
        let plugin_dir = dir.join(package);
        std::fs::create_dir_all(plugin_dir.join("templates/features")).unwrap();
        std::fs::write(plugin_dir.join("templates/features/custom.jinja"), "").unwrap();

        let manifest = PluginManifest {
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            package: package.to_string(),
            tsx_version: ">=0.1.0".to_string(),
            overrides: std::collections::HashMap::new(),
            generators: vec![],
            peer_dependencies: vec![],
            author: "".to_string(),
            docs: "".to_string(),
        };
        manifest.save(&plugin_dir).unwrap();
        plugin_dir
    }

    #[test]
    fn install_and_list() {
        let project = TempDir::new().unwrap();
        std::fs::write(project.path().join("package.json"), "{}").unwrap();
        let registry = PluginRegistry::open_at(project.path()).unwrap();

        let src = TempDir::new().unwrap();
        make_plugin_dir(src.path(), "tsx-plugin-test");
        let plugin_src = src.path().join("tsx-plugin-test");

        registry.install_from_dir(&plugin_src).unwrap();
        let plugins = registry.list();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].package, "tsx-plugin-test");
    }

    #[test]
    fn remove_plugin() {
        let project = TempDir::new().unwrap();
        let registry = PluginRegistry::open_at(project.path()).unwrap();

        let src = TempDir::new().unwrap();
        make_plugin_dir(src.path(), "tsx-plugin-rm");
        registry.install_from_dir(&src.path().join("tsx-plugin-rm")).unwrap();

        registry.remove("tsx-plugin-rm").unwrap();
        assert!(registry.list().is_empty());
    }
}
