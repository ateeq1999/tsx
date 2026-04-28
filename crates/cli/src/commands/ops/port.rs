use crate::json::response::ResponseEnvelope;
use crate::json::error::{ErrorResponse, ErrorCode};
use std::process::{Command, Stdio};

#[cfg(windows)]
pub fn find_process_by_port(port: u16) -> ResponseEnvelop {
    let start = std::time::Instant::now();

    // Use netstat to find process using the port
    let output = Command::new("netstat")
        .args(["-ano", "|", "findstr", &format!(":{}", port)])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let lines: Vec<&str> = stdout.lines().collect();

            let mut processes = Vec::new();
            for line in lines {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let pid = parts[parts.len() - 1];
                    // Get process name from PID
                    let proc_output = Command::new("tasklist")
                        .args(["/FI", &format!("PID eq {}", pid), "/NH", "/FO", "CSV"])
                        .output();

                    if let Ok(po) = proc_output {
                        let proc_info = String::from_utf8_lossy(&po.stdout).to_string();
                        if !proc_info.is_empty() && proc_info != "\r\n" {
                            processes.push(serde_json::json!({
                                "pid": pid,
                                "info": proc_info.trim()
                            }));
                        }
                    }
                }
            }

            let result = serde_json::json!({
                "port": port,
                "found": !processes.is_empty(),
                "processes": processes
            });

            ResponseEnvelop::success("port", result, start.elapsed().as_millis() as u64)
        }
        Err(e) => {
            ResponseEnvelop::error(
                "port",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run netstat: {}", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

#[cfg(windows)]
pub fn kill_process_by_port(port: u16) -> ResponseEnvelop {
    let start = std::time::Instant::now();

    // First find the PID
    let output = Command::new("netstat")
        .args(["-ano", "|", "findstr", &format!(":{}", port)])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let lines: Vec<&str> = stdout.lines().collect();

            let mut killed = Vec::new();
            let mut errors = Vec::new();

            for line in lines {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let pid = parts[parts.len() - 1];
                    let kill_output = Command::new("taskkill")
                        .args(["/F", "/PID", pid])
                        .output();

                    match kill_output {
                        Ok(ko) => {
                            if ko.status.success() {
                                killed.push(pid.to_string());
                            } else {
                                let err = String::from_utf8_lossy(&ko.stderr).to_string();
                                errors.push(serde_json::json!({
                                    "pid": pid,
                                    "error": err
                                }));
                            }
                        }
                        Err(e) => {
                            errors.push(serde_json::json!({
                                "pid": pid,
                                "error": format!("{}", e)
                            }));
                        }
                    }
                }
            }

            let result = serde_json::json!({
                "port": port,
                "killed": killed,
                "errors": errors
            });

            if errors.is_empty() {
                ResponseEnvelop::success("port", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelop::error(
                    "port",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("Failed to kill {} processes", errors.len()),
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => {
            ResponseEnvelop::error(
                "port",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run netstat: {}", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

#[cfg(unix)]
pub fn find_process_by_port(port: u16) -> ResponseEnvelop {
    let start = std::time::Instant::now();

    let output = Command::new("lsof")
        .args(["-ti", &format!(":{}", port)])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let pids: Vec<&str> = stdout.lines().collect();

            let mut processes = Vec::new();
            for pid in pids {
                if !pid.is_empty() {
                    let proc_output = Command::new("ps")
                        .args(["-p", pid, "-o", "comm="])
                        .output();

                    if let Ok(po) = proc_output {
                        let name = String::from_utf8_lossy(&po.stdout).trim().to_string();
                        processes.push(serde_json::json!({
                            "pid": pid,
                            "name": name
                        }));
                    }
                }
            }

            let result = serde_json::json!({
                "port": port,
                "found": !processes.is_empty(),
                "processes": processes
            });

            ResponseEnvelop::success("port", result, start.elapsed().as_millis() as u64)
        }
        Err(e) => {
            ResponseEnvelop::error(
                "port",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run lsof: {}", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

#[cfg(unix)]
pub fn kill_process_by_port(port: u16) -> ResponseEnvelop {
    let start = std::time::Instant::now();

    let output = Command::new("lsof")
        .args(["-ti", &format!(":{}", port)])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let pids: Vec<&str> = stdout.lines().collect();

            let mut killed = Vec::new();
            let mut errors = Vec::new();

            for pid in pids {
                if pid.is_empty() {
                    continue;
                }
                let kill_output = Command::new("kill")
                    .args(["-9", pid])
                    .output();

                match kill_output {
                    Ok(ko) => {
                        if ko.status.success() {
                            killed.push(pid.to_string());
                        } else {
                            let err = String::from_utf8_lossy(&ko.stderr).to_string();
                            errors.push(serde_json::json!({
                                "pid": pid,
                                "error": err
                            }));
                        }
                    }
                    Err(e) => {
                        errors.push(serde_json::json!({
                            "pid": pid,
                            "error": format!("{}", e)
                        }));
                    }
                }
            }

            let result = serde_json::json!({
                "port": port,
                "killed": killed,
                "errors": errors
            });

            if errors.is_empty() {
                ResponseEnvelop::success("port", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelop::error(
                    "port",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("Failed to kill {} processes", errors.len()),
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => {
            ResponseEnvelop::error(
                "port",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Failed to run lsof: {}", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    }
}

pub fn kill_all_port(port: u16) -> ResponseEnvelop {
    kill_process_by_port(port)
}
