use anyhow::Result;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn format_typescript(content: &str) -> Result<String> {
    let mut child = Command::new("npx")
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
        .spawn()?;

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(content.as_bytes())?;

    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Prettier formatting failed: {}", stderr)
    }
}

pub fn format_tsx(content: &str) -> Result<String> {
    let mut child = Command::new("npx")
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
        .spawn()?;

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(content.as_bytes())?;

    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Prettier formatting failed: {}", stderr)
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
