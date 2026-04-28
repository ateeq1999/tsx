use crate::json::response::ResponseEnvelope;
use crate::json::error::{ErrorResponse, ErrorCode};
use std::process::{Command, Stdio};

pub fn flutter_run(mode: String, device: Option<String>, port: Option<u16>) -> ResponseEnvelop {
    let start = std::time::Instant::now();

    let mut cmd = Command::new("flutter");

    if device.is_some() {
        cmd.arg("-d").arg(device.unwrap());
    }

    if let Some(p) = port {
        cmd.arg("--port").arg(p.to_string());
    }

    let output = cmd
        .arg("run")
        .arg(format!("--{}", mode))
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
                ResponseEnvelop::success("flutter", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelop::error(
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
            ResponseEnvelop::error(
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

pub fn flutter_build(target: Option<String>, release: bool) -> ResponseEnvelop {
    let start = std::time::Instant::now();

    let mut cmd = Command::new("flutter");

    cmd.arg("build");

    if let Some(t) = target {
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
                ResponseEnvelop::success("flutter", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelop::error(
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
            ResponseEnvelop::error(
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

pub fn flutter_clean() -> ResponseEnvelop {
    let start = std::time::Instant::now();

    let output = Command::new("flutter")
        .arg("clean")
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let result = serde_json::json!({
                "command": "clean",
                "status": if o.status.success() { "success" } else { "error" },
                "output": stdout
            });
            if o.status.success() {
                ResponseEnvelop::success("flutter", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelop::error(
                    "flutter",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        stdout,
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => {
            ResponseEnvelop::error(
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

pub fn flutter_pub_get() -> ResponseEnvelop {
    let start = std::time::Instant::now();

    let output = Command::new("flutter")
        .args(["pub", "get"])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let result = serde_json::json!({
                "command": "pub get",
                "status": if o.status.success() { "success" } else { "error" },
                "output": stdout
            });
            if o.status.success() {
                ResponseEnvelop::success("flutter", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelop::error(
                    "flutter",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        stdout,
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => {
            ResponseEnvelop::error(
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
