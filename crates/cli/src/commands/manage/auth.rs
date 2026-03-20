use crate::output::CommandResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Credentials ───────────────────────────────────────────────────────────────

const DEFAULT_REGISTRY_URL: &str = "https://tsx-tsnv.onrender.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub registry_url: String,
    pub api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

fn credentials_path() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".tsx")
        .join("credentials.json")
}

pub fn load_credentials() -> Option<Credentials> {
    let path = credentials_path();
    let raw = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn save_credentials(creds: &Credentials) -> anyhow::Result<()> {
    let path = credentials_path();
    std::fs::create_dir_all(path.parent().unwrap())?;
    let json = serde_json::to_string_pretty(creds)?;
    std::fs::write(&path, json)?;
    Ok(())
}

// ── tsx login ─────────────────────────────────────────────────────────────────

/// `tsx login --token <key> [--registry <url>]`
///
/// Stores the API key and registry URL in `~/.tsx/credentials.json`.
/// Validates connectivity by pinging the registry's `/health` endpoint.
pub fn login(token: String, registry: Option<String>) -> CommandResult {
    let registry_url = registry
        .as_deref()
        .unwrap_or(DEFAULT_REGISTRY_URL)
        .trim_end_matches('/')
        .to_string();

    // Validate the key is non-empty
    if token.trim().is_empty() {
        return CommandResult::err("login", "API key cannot be empty");
    }

    // Ping the registry to verify it's reachable
    let client = match reqwest::blocking::Client::builder()
        .user_agent(format!("tsx-cli/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CommandResult::err("login", format!("Failed to build HTTP client: {e}")),
    };

    let health_url = format!("{registry_url}/health");
    match client.get(&health_url).send() {
        Err(e) => {
            return CommandResult::err(
                "login",
                format!("Cannot reach registry at {registry_url}: {e}"),
            )
        }
        Ok(resp) if !resp.status().is_success() => {
            return CommandResult::err(
                "login",
                format!(
                    "Registry returned non-OK status: {}",
                    resp.status()
                ),
            )
        }
        Ok(_) => {}
    }

    // Validate the API key by making an authenticated request
    let validate_url = format!("{registry_url}/v1/auth/whoami");
    let user_info: Option<(Option<String>, Option<String>)> = client
        .get(&validate_url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .ok()
        .and_then(|r| {
            if r.status().is_success() {
                r.json::<serde_json::Value>().ok().map(|v| {
                    let username = v.get("username").and_then(|u| u.as_str()).map(String::from);
                    let email = v.get("email").and_then(|e| e.as_str()).map(String::from);
                    (username, email)
                })
            } else {
                None
            }
        });

    let (username, email) = user_info.unwrap_or((None, None));

    let creds = Credentials {
        registry_url: registry_url.clone(),
        api_key: token,
        username: username.clone(),
        email: email.clone(),
    };

    if let Err(e) = save_credentials(&creds) {
        return CommandResult::err("login", format!("Failed to save credentials: {e}"));
    }

    let display = match username.as_deref() {
        Some(u) => format!("Logged in as {u} @ {registry_url}"),
        None => format!("API key saved for {registry_url}"),
    };

    let mut result = CommandResult::ok("login", vec![]);
    result.next_steps = vec![
        display,
        "Run `tsx whoami` to confirm your credentials.".to_string(),
        "Run `tsx pkg install <name>` to install packages.".to_string(),
    ];
    result
}

// ── tsx logout ────────────────────────────────────────────────────────────────

/// `tsx logout`
///
/// Removes `~/.tsx/credentials.json`.
pub fn logout() -> CommandResult {
    let path = credentials_path();
    if !path.exists() {
        let mut result = CommandResult::ok("logout", vec![]);
        result.next_steps = vec!["No credentials found — already logged out.".to_string()];
        return result;
    }
    if let Err(e) = std::fs::remove_file(&path) {
        return CommandResult::err("logout", format!("Failed to remove credentials: {e}"));
    }
    let mut result = CommandResult::ok("logout", vec![]);
    result.next_steps = vec!["Credentials removed. Run `tsx login --token <key>` to log in again.".to_string()];
    result
}

// ── tsx whoami ────────────────────────────────────────────────────────────────

/// `tsx whoami`
///
/// Prints the stored credentials and validates them against the registry.
pub fn whoami() -> CommandResult {
    let creds = match load_credentials() {
        Some(c) => c,
        None => {
            return CommandResult::err(
                "whoami",
                "Not logged in. Run `tsx login --token <key>` first.",
            )
        }
    };

    let masked_key = {
        let key = &creds.api_key;
        if key.len() > 8 {
            format!("{}…{}", &key[..4], &key[key.len() - 4..])
        } else {
            "****".to_string()
        }
    };

    let mut info = vec![
        format!("Registry: {}", creds.registry_url),
        format!("API key:  {masked_key}"),
    ];
    if let Some(u) = &creds.username {
        info.push(format!("Username: {u}"));
    }
    if let Some(e) = &creds.email {
        info.push(format!("Email:    {e}"));
    }

    let mut result = CommandResult::ok("whoami", vec![]);
    result.next_steps = info;
    result
}
