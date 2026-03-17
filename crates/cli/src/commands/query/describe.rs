//! `tsx describe <framework>` — agent entry point for any registered framework.
//!
//! Returns a cost map of what knowledge is available and how many tokens each section costs,
//! so an agent can decide what to load before committing to loading it.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

use crate::framework::knowledge::load_knowledge;
use crate::framework::loader::FrameworkLoader;
use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::get_frameworks_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribeResult {
    pub framework: String,
    pub version: String,
    pub category: String,
    pub docs: String,
    /// Map from section name → token cost + retrieval command.
    pub knowledge_available: HashMap<String, KnowledgeMeta>,
    pub generators: Vec<String>,
    pub starters: Vec<String>,
    pub integrations: Vec<String>,
    /// Sum of all knowledge section token estimates.
    pub total_knowledge_tokens: u32,
    /// Suggested first command for an agent starting fresh.
    pub quick_start: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeMeta {
    pub token_estimate: u32,
    pub command: String,
}

pub fn describe(target: Option<String>, section: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();

    // No target — return a high-level summary of what's available
    let framework = match target {
        Some(t) => t,
        None => {
            let dur = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::validation(
                "Provide a framework slug or generator id. Examples:\n  tsx describe tanstack-start\n  tsx describe add:schema",
            );
            ResponseEnvelope::error("describe", error, dur).print();
            return CommandResult::err("describe", "Missing target");
        }
    };

    // Check if it looks like a generator id (contains ':' or matches a known generator)
    // Try generator lookup first when the target contains ':' or doesn't match framework slugs
    if framework.contains(':') || !framework_looks_like_slug(&framework) {
        if let Some(result) = describe_generator(&framework, start) {
            return result;
        }
        // Fall through to framework lookup (e.g. user mistyped)
    }

    let mut loader = FrameworkLoader::default();
    loader.load_builtin_frameworks();

    let reg = match loader.get_registry(&framework) {
        Some(r) => r.clone(),
        None => {
            // One more attempt: try as generator id even without ':'
            if let Some(result) = describe_generator(&framework, start) {
                return result;
            }
            let dur = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::validation(&format!(
                "Not found: '{framework}'. Pass a framework slug (e.g. tanstack-start) or a generator command-id (e.g. add:schema)."
            ));
            ResponseEnvelope::error("describe", error, dur).print();
            return CommandResult::err("describe", format!("Unknown target: {framework}"));
        }
    };

    let frameworks_dir = get_frameworks_dir();
    let fw_dir = frameworks_dir.join(&framework);
    let knowledge_dir = fw_dir.join("knowledge");
    let starters_dir = fw_dir.join("starters");
    let integrations_dir = fw_dir.join("integrations");

    // If a specific section was requested, return just that content.
    if let Some(sec) = section {
        return serve_section(&framework, &knowledge_dir, &sec, start);
    }

    // Build knowledge cost map.
    let entries = load_knowledge(&knowledge_dir);
    let mut knowledge_available: HashMap<String, KnowledgeMeta> = HashMap::new();
    let mut total_tokens = 0u32;
    for entry in &entries {
        total_tokens += entry.token_estimate;
        knowledge_available.insert(
            entry.section.clone(),
            KnowledgeMeta {
                token_estimate: entry.token_estimate,
                command: format!("tsx describe {framework} --section {}", entry.section),
            },
        );
    }

    let starters = list_dir_stems(&starters_dir);
    let integrations = if integrations_dir.exists() {
        list_dir_stems(&integrations_dir)
    } else {
        reg.integrations.iter().map(|i| i.package.clone()).collect()
    };
    let generators: Vec<String> = reg.generators.iter().map(|g| g.id.clone()).collect();

    let result = DescribeResult {
        framework: reg.framework.clone(),
        version: reg.version.clone(),
        category: format!("{:?}", reg.category).to_lowercase(),
        docs: reg.docs.clone(),
        knowledge_available,
        generators,
        starters,
        integrations,
        total_knowledge_tokens: total_tokens,
        quick_start: format!("tsx create --from {framework}"),
    };

    let dur = start.elapsed().as_millis() as u64;
    let response = ResponseEnvelope::success("describe", serde_json::to_value(result).unwrap(), dur);
    if verbose {
        let ctx = crate::json::response::Context {
            project_root: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        response.with_context(ctx).print();
    } else {
        response.print();
    }

    CommandResult::ok("describe", vec![])
}

fn serve_section(
    framework: &str,
    knowledge_dir: &std::path::Path,
    section: &str,
    start: Instant,
) -> CommandResult {
    use crate::framework::knowledge::load_section;

    match load_section(knowledge_dir, section) {
        Some(entry) => {
            let mut map = serde_json::Map::new();
            map.insert("framework".into(), serde_json::Value::String(framework.to_string()));
            map.insert("section".into(), serde_json::Value::String(section.to_string()));
            map.insert(
                "token_estimate".into(),
                serde_json::Value::Number(entry.token_estimate.into()),
            );
            map.insert("content".into(), serde_json::Value::String(entry.content));
            let dur = start.elapsed().as_millis() as u64;
            ResponseEnvelope::success("describe", serde_json::Value::Object(map), dur).print();
            CommandResult::ok("describe", vec![])
        }
        None => {
            let dur = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::validation(&format!(
                "Section '{section}' not found for framework '{framework}'."
            ));
            ResponseEnvelope::error("describe", error, dur).print();
            CommandResult::err("describe", format!("Section not found: {section}"))
        }
    }
}

/// Returns true when `s` looks like a framework slug (kebab-case, no ':')
fn framework_looks_like_slug(s: &str) -> bool {
    !s.contains(':') && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Try to resolve `id_or_cmd` as a generator id / command name in the `CommandRegistry`.
/// Returns `Some(CommandResult)` if found, `None` if not found (caller should fall through).
fn describe_generator(id_or_cmd: &str, start: std::time::Instant) -> Option<CommandResult> {
    use crate::framework::command_registry::CommandRegistry;

    let registry = CommandRegistry::load_all();
    let spec = registry.resolve(id_or_cmd)?;

    let dur = start.elapsed().as_millis() as u64;
    let payload = serde_json::json!({
        "id": spec.id,
        "command": spec.command,
        "framework": spec.framework,
        "description": spec.description,
        "token_estimate": spec.token_estimate,
        "output_paths": spec.output_paths,
        "next_steps": spec.next_steps,
        "schema": spec.schema,
        "usage": format!("tsx run {} --json '{{...}}'", spec.command),
    });
    ResponseEnvelope::success("describe", payload, dur).print();
    Some(CommandResult::ok("describe", vec![]))
}

fn list_dir_stems(dir: &std::path::Path) -> Vec<String> {
    if !dir.exists() {
        return vec![];
    }
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    e.path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default()
}
