use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteOutcome {
    Created,
    Skipped,
    Overwritten,
}

pub fn write_file(path: &Path, content: &str, overwrite: bool) -> Result<WriteOutcome> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create parent directories")?;
    }

    if path.exists() {
        if overwrite {
            fs::write(path, content).context("Failed to write file")?;
            Ok(WriteOutcome::Overwritten)
        } else {
            Ok(WriteOutcome::Skipped)
        }
    } else {
        fs::write(path, content).context("Failed to write file")?;
        Ok(WriteOutcome::Created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_file_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let result = write_file(&file_path, "hello", false).unwrap();
        assert_eq!(result, WriteOutcome::Created);
        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "hello");
    }

    #[test]
    fn test_write_file_skips_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "original").unwrap();

        let result = write_file(&file_path, "new", false).unwrap();
        assert_eq!(result, WriteOutcome::Skipped);
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "original");
    }

    #[test]
    fn test_write_file_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "original").unwrap();

        let result = write_file(&file_path, "new", true).unwrap();
        assert_eq!(result, WriteOutcome::Overwritten);
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "new");
    }
}
