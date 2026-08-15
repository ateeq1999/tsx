use crate::output::CommandResult;
use std::path::Path;

struct Check {
    label: &'static str,
    pass: bool,
    detail: String,
}

impl Check {
    fn ok(label: &'static str, detail: impl Into<String>) -> Self {
        Check { label, pass: true, detail: detail.into() }
    }
    fn fail(label: &'static str, detail: impl Into<String>) -> Self {
        Check { label, pass: false, detail: detail.into() }
    }
    fn fmt(&self) -> String {
        let icon = if self.pass { "✓" } else { "✗" };
        if self.detail.is_empty() {
            format!("{icon} {}", self.label)
        } else {
            format!("{icon} {}  — {}", self.label, self.detail)
        }
    }
}

/// `tsx doctor`
///
/// Runs a series of diagnostic checks on the current project and environment,
/// printing a pass/fail summary to help identify configuration issues.
pub fn doctor() -> CommandResult {
    let mut checks: Vec<Check> = Vec::new();
    let mut all_passed = true;

    // ── 1. Project root ───────────────────────────────────────────────────────
    let root = match crate::utils::paths::find_project_root() {
        Ok(r) => {
            checks.push(Check::ok("Project root", r.display().to_string()));
            Some(r)
        }
        Err(_) => {
            checks.push(Check::fail(
                "Project root",
                "No package.json found. Run from a project directory.",
            ));
            all_passed = false;
            None
        }
    };

    // ── 2. .env file ─────────────────────────────────────────────────────────
    if let Some(ref root) = root {
        let env_path = root.join(".env");
        if env_path.exists() {
            checks.push(Check::ok(".env file", env_path.display().to_string()));
        } else {
            checks.push(Check::fail(
                ".env file",
                "Missing — copy .env.example to .env and fill in values.",
            ));
            all_passed = false;
        }
    }

    // ── 3. DATABASE_URL in .env ───────────────────────────────────────────────
    if let Some(ref root) = root {
        let env_val = read_env_var(root, "DATABASE_URL");
        match env_val {
            Some(v) if !v.is_empty() => checks.push(Check::ok("DATABASE_URL", "set")),
            _ => {
                checks.push(Check::fail("DATABASE_URL", "Not set in .env"));
                all_passed = false;
            }
        }
    }

    // ── 4. tsconfig.json with paths ───────────────────────────────────────────
    if let Some(ref root) = root {
        let tsconfig = root.join("tsconfig.json");
        if tsconfig.exists() {
            let content = std::fs::read_to_string(&tsconfig).unwrap_or_default();
            if content.contains("\"@/*\"") || content.contains("\"paths\"") {
                checks.push(Check::ok("tsconfig.json paths", "@ alias configured"));
            } else {
                checks.push(Check::fail(
                    "tsconfig.json paths",
                    "No '@/*' alias found — add paths to compilerOptions.",
                ));
                all_passed = false;
            }
        } else {
            checks.push(Check::fail("tsconfig.json", "Not found in project root"));
            all_passed = false;
        }
    }

    // ── 5. Drizzle schema directory ───────────────────────────────────────────
    if let Some(ref root) = root {
        let schema_candidates = [
            root.join("app").join("db").join("schema.ts"),
            root.join("src").join("db").join("schema.ts"),
            root.join("db").join("schema.ts"),
        ];
        if schema_candidates.iter().any(|p| p.exists()) {
            checks.push(Check::ok("Drizzle schema", "schema.ts found"));
        } else {
            checks.push(Check::fail(
                "Drizzle schema",
                "No schema.ts found in app/db/, src/db/, or db/",
            ));
            // Non-fatal: warn only
        }
    }

    // ── 6. tsx credentials (registry login) ───────────────────────────────────
    let creds_path = dirs_next::home_dir()
        .map(|h| h.join(".tsx").join("credentials.json"));
    match creds_path {
        Some(p) if p.exists() => {
            checks.push(Check::ok("Registry credentials", "~/.tsx/credentials.json present"));
        }
        _ => {
            checks.push(Check::fail(
                "Registry credentials",
                "Not logged in — run `tsx login --token <key>`",
            ));
            // Non-fatal
        }
    }

    // ── 7. tsx CLI version vs latest ──────────────────────────────────────────
    let current = env!("CARGO_PKG_VERSION");
    match fetch_latest_version() {
        Ok(latest) if latest != current => {
            checks.push(Check::fail(
                "tsx version",
                format!("v{current} installed; v{latest} available — run `tsx upgrade cli`"),
            ));
            // Non-fatal
        }
        Ok(_) => checks.push(Check::ok("tsx version", format!("v{current} (latest)"))),
        Err(_) => checks.push(Check::ok("tsx version", format!("v{current} (could not check latest)"))),
    }

    // ── Summary ───────────────────────────────────────────────────────────────
    let lines: Vec<String> = checks.iter().map(|c| c.fmt()).collect();

    let mut result = CommandResult::ok("doctor", vec![]);
    result.next_steps = lines;
    if !all_passed {
        result.next_steps.push(String::new());
        result.next_steps.push("Some checks failed. Fix the issues above and re-run `tsx doctor`.".to_string());
    }
    result
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Read a variable from the project `.env` file (not the process env).
fn read_env_var(root: &Path, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(root.join(".env")).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() { continue; }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().trim_matches('"').trim_matches('\'').to_string());
            }
        }
    }
    None
}

/// Fetch the latest tsx CLI version from GitHub Releases.
fn fetch_latest_version() -> Result<String, ()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("tsx-cli/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|_| ())?;

    let resp = client
        .get("https://api.github.com/repos/ateeq1999/tsx/releases/latest")
        .send()
        .map_err(|_| ())?;

    if !resp.status().is_success() { return Err(()); }

    let body: serde_json::Value = resp.json().map_err(|_| ())?;
    let tag = body.get("tag_name").and_then(|v| v.as_str()).ok_or(())?;
    Ok(tag.trim_start_matches('v').to_string())
}
