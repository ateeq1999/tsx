use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn update_barrel(dir: &Path, export_line: &str) -> Result<()> {
    let index_path = dir.join("index.ts");

    let content = if index_path.exists() {
        fs::read_to_string(&index_path)?
    } else {
        String::new()
    };

    if content.contains(export_line.trim()) {
        return Ok(());
    }

    let new_content = if content.is_empty() {
        format!("{}\n", export_line)
    } else {
        format!("{}\n{}", content.trim(), export_line)
    };

    fs::write(&index_path, new_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_update_barrel_new() {
        let temp_dir = TempDir::new().unwrap();

        update_barrel(temp_dir.path(), "export * from './foo';").unwrap();

        let content = fs::read_to_string(temp_dir.path().join("index.ts")).unwrap();
        assert!(content.contains("export * from './foo';"));
    }

    #[test]
    fn test_update_barrel_existing() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("index.ts");

        fs::write(&index_path, "export * from './bar';\n").unwrap();

        update_barrel(temp_dir.path(), "export * from './foo';").unwrap();

        let content = fs::read_to_string(&index_path).unwrap();
        assert!(content.contains("./bar"));
        assert!(content.contains("./foo"));
    }
}
