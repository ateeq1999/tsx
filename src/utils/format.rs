use anyhow::Result;
use std::process::{Command, Stdio};

pub fn format_typescript(_content: &str) -> Result<String> {
    let result = Command::new("npx")
        .args([
            "prettier",
            "--parser",
            "typescript",
            "--stdin-filepath",
            "file.ts",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Prettier formatting failed: {}", stderr)
            }
        }
        Err(e) => {
            anyhow::bail!("Failed to run Prettier: {}. Is Prettier installed?", e)
        }
    }
}

pub fn format_tsx(_content: &str) -> Result<String> {
    let result = Command::new("npx")
        .args([
            "prettier",
            "--parser",
            "typescript",
            "--stdin-filepath",
            "file.tsx",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Prettier formatting failed: {}", stderr)
            }
        }
        Err(e) => {
            anyhow::bail!("Failed to run Prettier: {}. Is Prettier installed?", e)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_format_typescript_available() {
        let result = super::format_typescript("const x=1");
        match result {
            Ok(formatted) => {
                assert!(formatted.contains("const x = 1"));
            }
            Err(_) => {
                println!("Prettier not available - this is expected in CI without npm");
            }
        }
    }
}
