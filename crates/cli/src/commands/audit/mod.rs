//! `tsx audit` — run `npm audit` and parse/format the output.
//!
//! Flags:
//! - `--severity <level>` — filter by severity: critical, high, moderate, low (default: all)
//! - `--fix`              — run `npm audit fix` instead of just auditing

use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditVulnerability {
    pub name: String,
    pub severity: String,
    pub via: Vec<String>,
    pub range: String,
    pub nodes: Vec<String>,
    pub fix_available: bool,
}

pub fn audit(severity: Option<String>, fix: bool, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let root = match crate::utils::paths::find_project_root() {
        Ok(p) => p,
        Err(_) => {
            return ResponseEnvelope::error(
                "audit",
                ErrorResponse::new(ErrorCode::ProjectNotFound, "No project root found (missing package.json)."),
                0,
            )
        }
    };

    if fix {
        return run_audit_fix(&root, start);
    }

    let output = Command::new("npm")
        .args(["audit", "--json"])
        .current_dir(&root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Err(e) => ResponseEnvelope::error(
            "audit",
            ErrorResponse::new(ErrorCode::InternalError, format!("Failed to run npm audit: {}", e)),
            start.elapsed().as_millis() as u64,
        ),
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let parsed = parse_audit_json(&stdout);
            let severity_filter = severity.as_deref().unwrap_or("all");

            let vulns: Vec<&AuditVulnerability> = parsed
                .iter()
                .filter(|v| {
                    severity_filter == "all"
                        || v.severity == severity_filter
                        || matches_min_severity(&v.severity, severity_filter)
                })
                .collect();

            let critical = vulns.iter().filter(|v| v.severity == "critical").count();
            let high = vulns.iter().filter(|v| v.severity == "high").count();
            let moderate = vulns.iter().filter(|v| v.severity == "moderate").count();
            let low = vulns.iter().filter(|v| v.severity == "low").count();

            let result = serde_json::json!({
                "total": vulns.len(),
                "critical": critical,
                "high": high,
                "moderate": moderate,
                "low": low,
                "vulnerabilities": vulns,
            });

            let mut env = ResponseEnvelope::success("audit", result, start.elapsed().as_millis() as u64);
            if critical > 0 || high > 0 {
                env.next_steps = vec![
                    format!("{} critical / {} high vulnerabilities — run `tsx audit --fix`", critical, high),
                ];
            }
            env
        }
    }
}

fn run_audit_fix(root: &PathBuf, start: std::time::Instant) -> ResponseEnvelope {
    let output = Command::new("npm")
        .args(["audit", "fix"])
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Err(e) => ResponseEnvelope::error(
            "audit --fix",
            ErrorResponse::new(ErrorCode::InternalError, format!("Failed to run npm audit fix: {}", e)),
            start.elapsed().as_millis() as u64,
        ),
        Ok(o) => {
            let exit_code = o.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let result = serde_json::json!({
                "fix": true,
                "exit_code": exit_code,
                "output": stdout,
            });
            ResponseEnvelope::success("audit --fix", result, start.elapsed().as_millis() as u64)
        }
    }
}

fn parse_audit_json(json_str: &str) -> Vec<AuditVulnerability> {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) else {
        return Vec::new();
    };

    let Some(vulns) = val.get("vulnerabilities").and_then(|v| v.as_object()) else {
        return Vec::new();
    };

    vulns
        .values()
        .map(|v| {
            let name = v.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let severity = v.get("severity").and_then(|s| s.as_str()).unwrap_or("info").to_string();
            let via: Vec<String> = v
                .get("via")
                .and_then(|a| a.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|x| {
                            x.as_str()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| x.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string())
                        })
                        .collect()
                })
                .unwrap_or_default();
            let range = v.get("range").and_then(|r| r.as_str()).unwrap_or("").to_string();
            let nodes: Vec<String> = v
                .get("nodes")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str()).map(|s| s.to_string()).collect())
                .unwrap_or_default();
            let fix_available = v.get("fixAvailable").and_then(|f| f.as_bool()).unwrap_or(false);

            AuditVulnerability { name, severity, via, range, nodes, fix_available }
        })
        .collect()
}

fn matches_min_severity(vuln_sev: &str, min_sev: &str) -> bool {
    let order = ["low", "moderate", "high", "critical"];
    let vuln_idx = order.iter().position(|&s| s == vuln_sev).unwrap_or(0);
    let min_idx = order.iter().position(|&s| s == min_sev).unwrap_or(0);
    vuln_idx >= min_idx
}
