//! `tsx test` — run the project's test suite (vitest / jest / playwright).
//!
//! Auto-detects the test runner from package.json devDependencies.
//! Passes through `--filter`, `--watch`, and `--json` flags.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

#[derive(Debug, Clone)]
enum TestRunner {
    Vitest,
    Jest,
    Playwright,
    Script(String), // `npm run test` fallback
}

pub fn test_run(
    filter: Option<String>,
    watch: bool,
    json_output: bool,
    _verbose: bool,
) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let root = match crate::utils::paths::find_project_root() {
        Ok(p) => p,
        Err(_) => {
            return ResponseEnvelope::error(
                "test",
                ErrorResponse::new(ErrorCode::ProjectNotFound, "No project root found (missing package.json)."),
                0,
            )
        }
    };

    let runner = detect_runner(&root);
    let (cmd, args) = build_command(&runner, filter.as_deref(), watch, json_output);

    let output = Command::new(&cmd)
        .args(&args)
        .current_dir(&root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(o) => {
            let exit_code = o.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let passed = exit_code == 0;

            // Try to parse vitest/jest JSON output for structured results
            let test_results = if json_output {
                parse_test_output(&stdout)
            } else {
                serde_json::Value::Null
            };

            let result = serde_json::json!({
                "runner": runner_name(&runner),
                "exit_code": exit_code,
                "passed": passed,
                "stdout": stdout,
                "stderr": stderr,
                "test_results": test_results,
            });

            if passed {
                ResponseEnvelope::success("test", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelope::error(
                    "test",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("Tests failed (exit {})", exit_code),
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => ResponseEnvelope::error(
            "test",
            ErrorResponse::new(ErrorCode::InternalError, format!("Failed to run tests: {}", e)),
            start.elapsed().as_millis() as u64,
        ),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn detect_runner(root: &PathBuf) -> TestRunner {
    let pkg_path = root.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&pkg_path) {
        if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
            let all_deps: Vec<String> = ["dependencies", "devDependencies"]
                .iter()
                .filter_map(|k| pkg.get(k)?.as_object())
                .flat_map(|m| m.keys().cloned())
                .collect();

            if all_deps.iter().any(|d| d == "vitest") {
                return TestRunner::Vitest;
            }
            if all_deps.iter().any(|d| d == "@playwright/test" || d == "playwright") {
                return TestRunner::Playwright;
            }
            if all_deps.iter().any(|d| d == "jest" || d == "@jest/core") {
                return TestRunner::Jest;
            }
            // Check scripts.test
            if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
                if let Some(script) = scripts.get("test").and_then(|v| v.as_str()) {
                    return TestRunner::Script(script.to_string());
                }
            }
        }
    }
    TestRunner::Script("test".to_string())
}

fn build_command(
    runner: &TestRunner,
    filter: Option<&str>,
    watch: bool,
    json_output: bool,
) -> (String, Vec<String>) {
    match runner {
        TestRunner::Vitest => {
            let mut args = vec!["run".to_string()];
            if watch { args = vec!["watch".to_string()]; }
            if json_output { args.push("--reporter=json".to_string()); }
            if let Some(f) = filter { args.push(f.to_string()); }
            ("npx".to_string(), {
                let mut full = vec!["vitest".to_string()];
                full.extend(args);
                full
            })
        }
        TestRunner::Jest => {
            let mut args = vec![];
            if watch { args.push("--watch".to_string()); }
            if json_output { args.push("--json".to_string()); }
            if let Some(f) = filter { args.push("--testNamePattern".to_string()); args.push(f.to_string()); }
            ("npx".to_string(), {
                let mut full = vec!["jest".to_string()];
                full.extend(args);
                full
            })
        }
        TestRunner::Playwright => {
            let mut args = vec![];
            if let Some(f) = filter { args.push("-g".to_string()); args.push(f.to_string()); }
            if json_output { args.push("--reporter=json".to_string()); }
            ("npx".to_string(), {
                let mut full = vec!["playwright".to_string(), "test".to_string()];
                full.extend(args);
                full
            })
        }
        TestRunner::Script(script) => {
            ("npm".to_string(), vec!["run".to_string(), script.clone()])
        }
    }
}

fn runner_name(runner: &TestRunner) -> &str {
    match runner {
        TestRunner::Vitest => "vitest",
        TestRunner::Jest => "jest",
        TestRunner::Playwright => "playwright",
        TestRunner::Script(_) => "npm run test",
    }
}

fn parse_test_output(stdout: &str) -> serde_json::Value {
    // Try to find a JSON block in stdout
    for line in stdout.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            return v;
        }
    }
    serde_json::Value::Null
}
