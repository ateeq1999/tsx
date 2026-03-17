use std::path::PathBuf;

use crate::framework::command_registry::{CommandRegistry, GeneratorSpec};
use crate::json::response::ResponseEnvelope;
use crate::stack::StackProfile;

pub fn context(_verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let profile = StackProfile::load(&cwd);
    let registry = CommandRegistry::load_all();

    // If a stack profile exists, filter commands to active packages.
    // Otherwise, return everything.
    let all = registry.all();
    let commands: Vec<&GeneratorSpec> = if let Some(ref p) = profile {
        let names = p.package_names();
        all.iter()
            .copied()
            .filter(|s| {
                names.contains(&s.framework.as_str())
                    || names.iter().any(|n| s.framework.starts_with(n))
            })
            .collect()
    } else {
        all.iter().copied().collect()
    };

    let command_list: Vec<serde_json::Value> = commands
        .iter()
        .map(|s| {
            serde_json::json!({
                "command": s.command,
                "id": s.id,
                "package": s.framework,
                "description": s.description,
                "token_estimate": s.token_estimate,
                "required_inputs": required_fields(s),
            })
        })
        .collect();

    let stack_json = profile
        .as_ref()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .unwrap_or_else(|| {
            serde_json::json!({
                "hint": "No .tsx/stack.json found. Run `tsx stack init` to configure your stack."
            })
        });

    let summary = build_summary(&profile, &command_list);

    ResponseEnvelope::success(
        "context",
        serde_json::json!({
            "stack": stack_json,
            "commands": command_list,
            "command_count": command_list.len(),
            "summary": summary,
        }),
        0,
    )
}

fn required_fields(spec: &GeneratorSpec) -> Vec<String> {
    spec.schema
        .as_ref()
        .and_then(|s| s.get("required"))
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn build_summary(profile: &Option<StackProfile>, commands: &[serde_json::Value]) -> String {
    let lang = profile.as_ref().map(|p| p.lang.as_str()).unwrap_or("unknown");
    let runtime = profile
        .as_ref()
        .and_then(|p| p.runtime.as_deref())
        .unwrap_or("");
    let packages = profile
        .as_ref()
        .map(|p| p.package_names().join(", "))
        .unwrap_or_else(|| "none configured — run `tsx stack init`".to_string());
    let count = commands.len();

    let runtime_str = if runtime.is_empty() {
        String::new()
    } else {
        format!(" / {runtime}")
    };

    format!(
        "This project uses tsx for code generation. \
         Language: {lang}{runtime_str}. \
         Active packages: {packages}. \
         {count} commands available. \
         Use `tsx run <command> --json '<payload>'` to generate code. \
         Use `tsx list --json` for full schemas. \
         Use `tsx describe <id> --json` for usage details. \
         Prefer tsx commands over writing boilerplate manually."
    )
}
