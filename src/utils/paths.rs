use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

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

        current_dir = current_dir
            .parent()
            .context("Failed to get parent directory")?
            .to_path_buf();
    }

    anyhow::bail!("Could not find project root (package.json not found)")
}

pub fn resolve_output_path(root: &Path, relative: &str) -> PathBuf {
    root.join(relative)
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
