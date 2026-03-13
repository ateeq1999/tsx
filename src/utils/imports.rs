use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn inject_import(file_path: &Path, import_line: &str) -> Result<()> {
    let content = fs::read_to_string(file_path)?;

    if content.contains(import_line.trim()) {
        return Ok(());
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();
    let mut found_first_import = false;
    let mut inserted = false;

    for line in lines {
        if line.starts_with("import ") && !found_first_import {
            found_first_import = true;
        }

        if found_first_import
            && !inserted
            && !line.starts_with("import ")
            && !line.trim().is_empty()
        {
            new_lines.push(import_line);
            inserted = true;
        }

        new_lines.push(line);
    }

    if !inserted {
        new_lines.insert(0, import_line);
    }

    fs::write(file_path, new_lines.join("\n"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_inject_import_new() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ts");

        fs::write(&file_path, "export const x = 1;\n").unwrap();

        inject_import(&file_path, "import { foo } from 'foo';").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("import { foo } from 'foo';"));
    }

    #[test]
    fn test_inject_import_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ts");

        fs::write(
            &file_path,
            "import { foo } from 'foo';\nexport const x = 1;\n",
        )
        .unwrap();

        inject_import(&file_path, "import { foo } from 'foo';").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content.matches("import").count(), 1);
    }
}
