use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Resolves the templates directory using this priority order:
/// 1. `<exe_dir>/templates` — templates shipped next to the binary
/// 2. `<root>/.tsx/templates` — project-local copies written by `tsx upgrade`
/// 3. `<root>/templates` — project-level overrides
pub fn get_templates_dir(root: &Path) -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    if let Some(dir) = exe_dir {
        let templates = dir.join("templates");
        if templates.exists() {
            return templates;
        }
    }

    let tsx_templates = root.join(".tsx").join("templates");
    if tsx_templates.exists() {
        return tsx_templates;
    }

    root.join("templates")
}

/// Returns the frameworks directory (next to the binary, or `./frameworks` in cwd).
/// Mirrors the logic in `FrameworkLoader::default()`.
pub fn get_frameworks_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    exe_dir
        .map(|d| d.join("frameworks"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("frameworks"))
}

pub fn find_project_root() -> Result<PathBuf> {
    let mut current_dir = std::env::current_dir().context("Failed to get current directory")?;

    loop {
        let package_json = current_dir.join("package.json");
        if package_json.is_file() {
            return Ok(current_dir);
        }

        if !current_dir.has_root() {
            break;
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    anyhow::bail!("Could not find project root: no package.json found. Run this command from a project directory.")
}

pub fn resolve_output_path(root: &Path, relative: &str) -> PathBuf {
    root.join(relative)
}

/// Returns all installed plugin template directories under `<root>/.tsx/plugins/<pkg>/templates/`.
pub fn get_plugin_template_dirs(root: &Path) -> Vec<PathBuf> {
    let plugins_dir = root.join(".tsx").join("plugins");
    let Ok(entries) = std::fs::read_dir(&plugins_dir) else {
        return vec![];
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.path().join("templates"))
        .filter(|p| p.is_dir())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_project_root() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        std::fs::write(project_root.join("package.json"), "{}").unwrap();
        std::fs::create_dir_all(project_root.join("src/components")).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(project_root.join("src/components")).unwrap();

        let found_root = find_project_root().unwrap();
        assert_eq!(found_root, project_root);

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_resolve_output_path() {
        let root = PathBuf::from("/project");
        let result = resolve_output_path(&root, "db/schema/products.ts");
        assert_eq!(result, PathBuf::from("/project/db/schema/products.ts"));
    }
}
