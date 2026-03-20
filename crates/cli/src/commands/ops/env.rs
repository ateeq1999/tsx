//! `tsx env` — validate and diff `.env` files (A: tsx env check / tsx env diff).
//!
//! `tsx env check` — validate `.env` against `.env.schema`:
//!   Each line in `.env.schema` is either:
//!     DATABASE_URL=string         # required string
//!     PORT=number?                # optional number
//!     LOG_LEVEL=string?           # optional
//!     API_KEY=string # required   # inline comment
//!
//! `tsx env diff` — show vars in `.env.example` that are missing from `.env`

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub type_hint: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvIssue {
    pub key: String,
    pub severity: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// tsx env check
// ---------------------------------------------------------------------------

pub fn env_check(schema_path: Option<String>, env_path: Option<String>) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let cwd = project_root();

    let schema_file = resolve_path(&cwd, schema_path, &[".env.schema", ".env.example"]);
    let env_file = resolve_path(&cwd, env_path, &[".env", ".env.local"]);

    let schema_file = match schema_file {
        Some(p) => p,
        None => {
            return ResponseEnvelope::error(
                "env check",
                ErrorResponse::new(
                    ErrorCode::TemplateNotFound,
                    "No .env.schema found. Create one or pass --schema <file>.",
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    };

    let env_file = match env_file {
        Some(p) => p,
        None => {
            return ResponseEnvelope::error(
                "env check",
                ErrorResponse::new(
                    ErrorCode::TemplateNotFound,
                    "No .env file found. Create one or pass --env <file>.",
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    };

    let schema_vars = parse_schema(&schema_file);
    let env_vars = parse_env(&env_file);

    let mut issues: Vec<EnvIssue> = Vec::new();

    for var in &schema_vars {
        match env_vars.get(&var.key) {
            None => {
                if var.required {
                    issues.push(EnvIssue {
                        key: var.key.clone(),
                        severity: "error".to_string(),
                        message: format!("Required variable '{}' is missing from .env", var.key),
                    });
                }
            }
            Some(val) => {
                if let Err(e) = validate_type(val, &var.type_hint) {
                    issues.push(EnvIssue {
                        key: var.key.clone(),
                        severity: "warning".to_string(),
                        message: format!("'{}' {}", var.key, e),
                    });
                }
            }
        }
    }

    let errors = issues.iter().filter(|i| i.severity == "error").count();
    let result = serde_json::json!({
        "schema_file": schema_file.to_string_lossy(),
        "env_file": env_file.to_string_lossy(),
        "schema_vars": schema_vars.len(),
        "errors": errors,
        "warnings": issues.iter().filter(|i| i.severity == "warning").count(),
        "issues": issues,
    });

    let mut env = ResponseEnvelope::success("env check", result, start.elapsed().as_millis() as u64);
    if errors > 0 {
        env.next_steps = vec![format!("{} required variable(s) missing from .env", errors)];
    }
    env
}

// ---------------------------------------------------------------------------
// tsx env diff
// ---------------------------------------------------------------------------

pub fn env_diff(example_path: Option<String>, env_path: Option<String>) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let cwd = project_root();

    let example_file = resolve_path(&cwd, example_path, &[".env.example", ".env.schema"]);
    let env_file = resolve_path(&cwd, env_path, &[".env", ".env.local"]);

    let example_file = match example_file {
        Some(p) => p,
        None => {
            return ResponseEnvelope::error(
                "env diff",
                ErrorResponse::new(ErrorCode::TemplateNotFound, "No .env.example found."),
                start.elapsed().as_millis() as u64,
            )
        }
    };

    let example_vars = parse_env(&example_file);
    let env_vars = match &env_file {
        Some(p) => parse_env(p),
        None => HashMap::new(),
    };

    let missing: Vec<String> = example_vars
        .keys()
        .filter(|k| !env_vars.contains_key(*k))
        .cloned()
        .collect();

    let extra: Vec<String> = env_vars
        .keys()
        .filter(|k| !example_vars.contains_key(*k))
        .cloned()
        .collect();

    let result = serde_json::json!({
        "example_file": example_file.to_string_lossy(),
        "env_file": env_file.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "(not found)".to_string()),
        "missing_from_env": missing,
        "extra_in_env": extra,
    });

    let mut env = ResponseEnvelope::success("env diff", result, start.elapsed().as_millis() as u64);
    if !missing.is_empty() {
        env.next_steps = vec![
            format!("{} variable(s) in .env.example are missing from .env", missing.len()),
        ];
    }
    env
}

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

/// Parse a `.env.schema` file into typed var declarations.
/// Format: KEY=type[?]  or  KEY=type  (trailing `?` = optional)
fn parse_schema(path: &Path) -> Vec<EnvVar> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .filter_map(|line| {
            // Strip inline comment
            let line = line.splitn(2, " #").next().unwrap_or(line).trim();
            let (key, type_raw) = split_eq(line)?;
            let type_raw = type_raw.trim();
            let required = !type_raw.ends_with('?');
            let type_hint = type_raw.trim_end_matches('?').to_string();
            Some(EnvVar { key: key.to_string(), type_hint, required })
        })
        .collect()
}

/// Parse a `.env` or `.env.example` file into a key→value map.
fn parse_env(path: &Path) -> HashMap<String, String> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .filter_map(|line| {
            let (k, v) = split_eq(line)?;
            Some((k.trim().to_string(), v.trim().trim_matches('"').trim_matches('\'').to_string()))
        })
        .collect()
}

fn split_eq(s: &str) -> Option<(&str, &str)> {
    let pos = s.find('=')?;
    Some((&s[..pos], &s[pos + 1..]))
}

fn validate_type(val: &str, type_hint: &str) -> Result<(), String> {
    match type_hint {
        "number" | "int" => {
            if val.parse::<f64>().is_err() {
                return Err(format!("expected number, got '{}'", val));
            }
        }
        "boolean" | "bool" => {
            if !matches!(val, "true" | "false" | "1" | "0") {
                return Err(format!("expected boolean, got '{}'", val));
            }
        }
        "url" => {
            if !val.starts_with("http://") && !val.starts_with("https://") {
                return Err(format!("expected URL, got '{}'", val));
            }
        }
        _ => {} // "string" or unknown type — any value is valid
    }
    Ok(())
}

fn project_root() -> PathBuf {
    crate::utils::paths::find_project_root()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn resolve_path(cwd: &Path, given: Option<String>, defaults: &[&str]) -> Option<PathBuf> {
    if let Some(p) = given {
        let path = PathBuf::from(&p);
        let abs = if path.is_absolute() { path } else { cwd.join(path) };
        if abs.exists() { Some(abs) } else { None }
    } else {
        defaults
            .iter()
            .map(|name| cwd.join(name))
            .find(|p| p.exists())
    }
}
