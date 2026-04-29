use crate::json::response::ResponseEnvelope;
use crate::json::error::{ErrorResponse, ErrorCode};
use std::env;
use std::path::PathBuf;
use std::process::Command;
#[cfg(not(windows))]
use std::io::Write;

#[cfg(windows)]
const PATH_SEP: char = ';';
#[cfg(not(windows))]
const PATH_SEP: char = ':';

#[cfg(windows)]
fn get_system_path() -> std::io::Result<String> {
    // Use setx to get the system PATH
    let output = Command::new("cmd")
        .args(["/C", "echo %PATH%"])
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to get PATH"))
    }
}

#[cfg(windows)]
fn set_system_path(new_path: &str) -> std::io::Result<()> {
    // Use setx to set the system PATH (requires admin)
    let status = Command::new("setx")
        .args(["/M", "PATH", new_path])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to set system PATH. Try running as Administrator.",
        ))
    }
}

#[cfg(unix)]
fn get_system_path() -> std::io::Result<String> {
    env::var("PATH").map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))
}

#[cfg(unix)]
fn set_system_path(new_path: &str) -> std::io::Result<()> {
    // Add to shell profile
    let home = env::var("HOME").map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
    let profile = if std::path::Path::new(&format!("{}/.zshrc", home)).exists() {
        format!("{}/.zshrc", home)
    } else {
        format!("{}/.bashrc", home)
    };

    let line = format!("\nexport PATH=\"{}:$PATH\"\n", new_path);
    std::fs::OpenOptions::new()
        .append(true)
        .open(&profile)?
        .write_all(line.as_bytes())?;
    Ok(())
}

pub fn path_add(directory: Option<String>, permanent: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    // Get directory to add
    let dir = match directory {
        Some(d) => d,
        None => match env::current_dir() {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(e) => {
                return ResponseEnvelope::error(
                    "path",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("Failed to get current directory: {}", e),
                    ),
                    start.elapsed().as_millis() as u64,
                );
            }
        },
    };

    // Canonicalize the path
    let path_buf = PathBuf::from(&dir);
    let canonical = match path_buf.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return ResponseEnvelope::error(
                "path",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Invalid directory '{}': {}", dir, e),
                ),
                start.elapsed().as_millis() as u64,
            );
        }
    };
    let dir_str = canonical.to_string_lossy().to_string();

    // Check if already in PATH
    let current_path = match env::var("PATH") {
        Ok(p) => p,
        Err(_) => String::new(),
    };

    if current_path.split(PATH_SEP).any(|p| PathBuf::from(p) == canonical) {
        let result = serde_json::json!({
            "directory": dir_str,
            "permanent": permanent,
            "status": "already_in_path",
            "current_path": current_path
        });
        return ResponseEnvelope::success("path", result, start.elapsed().as_millis() as u64);
    }

    // Add to PATH
    if permanent {
        #[cfg(windows)]
        {
            let new_path = format!("{}{}{}", current_path, PATH_SEP, dir_str);
            if let Err(e) = set_system_path(&new_path) {
                return ResponseEnvelope::error(
                    "path",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("Failed to set permanent PATH: {}", e),
                    ),
                    start.elapsed().as_millis() as u64,
                );
            }
        }
        #[cfg(unix)]
        {
            if let Err(e) = set_system_path(&dir_str) {
                return ResponseEnvelope::error(
                    "path",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("Failed to set permanent PATH: {}", e),
                    ),
                    start.elapsed().as_millis() as u64,
                );
            }
        }
    } else {
        // Just for current session
        let new_path = format!("{}{}{}", current_path, PATH_SEP, dir_str);
        env::set_var("PATH", &new_path);
    }

    let result = serde_json::json!({
        "directory": dir_str,
        "permanent": permanent,
        "status": "added",
        "current_path": env::var("PATH").unwrap_or_default()
    });

    ResponseEnvelope::success("path", result, start.elapsed().as_millis() as u64)
}

pub fn path_list() -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let current_path = match env::var("PATH") {
        Ok(p) => p,
        Err(_) => String::new(),
    };

    let paths: Vec<&str> = current_path.split(PATH_SEP).collect();

    let result = serde_json::json!({
        "current_path": current_path,
        "entries": paths
    });

    ResponseEnvelope::success("path", result, start.elapsed().as_millis() as u64)
}
