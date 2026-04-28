use crate::json::response::ResponseEnvelope;
use crate::json::error::{ErrorResponse, ErrorCode};
use std::process::{Command, Stdio};

pub fn adb_kill() -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let output = Command::new("adb")
        .arg("kill-server")
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let result = serde_json::json!({
                    "action": "kill-server",
                    "status": "success",
                    "output": stdout
                });
                ResponseEnvelope::success("adb", result, start.elapsed().as_millis() as u64)
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
                ResponseEnvelope::error(
                    "adb",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("ADB kill-server failed: {}", stderr),
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => {
            ResponseEnvelope::error(
                "adb",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run adb: {}. Is Android SDK installed?", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

pub fn adb_start() -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let output = Command::new("adb")
        .arg("start-server")
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let result = serde_json::json!({
                    "action": "start-server",
                    "status": "success",
                    "output": stdout
                });
                ResponseEnvelope::success("adb", result, start.elapsed().as_millis() as u64)
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
                ResponseEnvelope::error(
                    "adb",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("ADB start-server failed: {}", stderr),
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => {
            ResponseEnvelope::error(
                "adb",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run adb: {}. Is Android SDK installed?", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

pub fn adb_status() -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let output = Command::new("adb")
        .arg("devices")
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();

            let result = serde_json::json!({
                "status": if o.status.success() { "running" } else { "error" },
                "output": stdout,
                "error": stderr
            });
            ResponseEnvelope::success("adb", result, start.elapsed().as_millis() as u64)
        }
        Err(e) => {
            ResponseEnvelope::error(
                "adb",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run adb: {}. Is Android SDK installed?", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

pub fn adb_reverse(port: u16) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let output = Command::new("adb")
        .args(["reverse", &format!("tcp:{}", port), &format!("tcp:{}", port)])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let result = serde_json::json!({
                "action": "reverse",
                "port": port,
                "status": if o.status.success() { "success" } else { "error" },
                "output": stdout
            });
            ResponseEnvelope::success("adb", result, start.elapsed().as_millis() as u64)
        }
        Err(e) => {
            ResponseEnvelope::error(
                "adb",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run adb reverse: {}", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

pub fn adb_exec(args: Vec<String>) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let output = Command::new("adb")
        .args(&args)
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            let result = serde_json::json!({
                "args": args,
                "status": if o.status.success() { "success" } else { "error" },
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": o.status.code().unwrap_or(-1)
            });
            ResponseEnvelope::success("adb", result, start.elapsed().as_millis() as u64)
        }
        Err(e) => {
            ResponseEnvelope::error(
                "adb",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run adb: {}", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}
