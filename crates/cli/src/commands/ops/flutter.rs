use crate::json::response::ResponseEnvelope;
use crate::json::error::{ErrorResponse, ErrorCode};
use std::process::{Command, Stdio};

pub fn flutter_run(mode: String, device: Option<String>, port: Option<u16>) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let mut cmd = Command::new("flutter");

    // -d is a global Flutter option so it can precede the subcommand
    if let Some(ref d) = device {
        cmd.arg("-d").arg(d);
    }

    // "run" must come before run-specific options like --port
    cmd.arg("run");
    cmd.arg(format!("--{}", mode));

    if let Some(p) = port {
        cmd.arg("--port").arg(p.to_string());
    }

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let result = serde_json::json!({
                "command": "run",
                "mode": mode,
                "device": device,
                "port": port,
                "status": if o.status.success() { "success" } else { "error" },
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": o.status.code().unwrap_or(-1)
            });
            if o.status.success() {
                ResponseEnvelope::success("flutter", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelope::error(
                    "flutter",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("Flutter run failed: {}", stderr),
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => {
            ResponseEnvelope::error(
                "flutter",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run flutter: {}. Is Flutter SDK installed?", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

pub fn flutter_build(target: Option<String>, release: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let mut cmd = Command::new("flutter");

    cmd.arg("build");

    if let Some(ref t) = target {
        cmd.arg(t);
    }

    if release {
        cmd.arg("--release");
    }

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let result = serde_json::json!({
                "command": "build",
                "target": target,
                "release": release,
                "status": if o.status.success() { "success" } else { "error" },
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": o.status.code().unwrap_or(-1)
            });
            if o.status.success() {
                ResponseEnvelope::success("flutter", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelope::error(
                    "flutter",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("Flutter build failed: {}", stderr),
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => {
            ResponseEnvelope::error(
                "flutter",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run flutter: {}. Is Flutter SDK installed?", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

pub fn flutter_clean() -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let output = Command::new("flutter")
        .arg("clean")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let result = serde_json::json!({
                "command": "clean",
                "status": if o.status.success() { "success" } else { "error" },
                "output": stdout
            });
            if o.status.success() {
                ResponseEnvelope::success("flutter", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelope::error(
                    "flutter",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("Flutter clean failed: {}", stderr),
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => {
            ResponseEnvelope::error(
                "flutter",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run flutter: {}. Is Flutter SDK installed?", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

pub fn flutter_pub_get() -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let output = Command::new("flutter")
        .args(["pub", "get"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let result = serde_json::json!({
                "command": "pub get",
                "status": if o.status.success() { "success" } else { "error" },
                "output": stdout
            });
            if o.status.success() {
                ResponseEnvelope::success("flutter", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelope::error(
                    "flutter",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("Flutter pub get failed: {}", stderr),
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => {
            ResponseEnvelope::error(
                "flutter",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run flutter: {}. Is Flutter SDK installed?", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}
