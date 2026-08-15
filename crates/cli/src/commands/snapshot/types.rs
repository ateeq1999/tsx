use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A registered snapshot fixture: `.tsx/snapshots/<generator-id>/<fixture-name>.json`
/// plus its expected output directory `<fixture-name>.output/`.
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
}
