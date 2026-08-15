use std::time::Instant;

use crate::framework::command_registry::CommandRegistry;
use crate::json::payload::BatchPayload;
use crate::json::response::ResponseEnvelope;

/// Plan a batch without executing it: resolve each command against the registry and
/// return what files would be created along with token estimates.
///
/// `tsx batch --json '<payload>' --plan`
pub fn batch_plan(payload: BatchPayload, _verbose: bool) -> ResponseEnvelope {
    let start = Instant::now();
    let registry = CommandRegistry::load_all();

    // Load path config from stack profile for output path expansion.
    let cwd = std::env::current_dir().unwrap_or_default();
    let stack = crate::stack::StackProfile::load(&cwd);
    let path_config = stack.as_ref().map(|p| &p.paths);

    let mut total_tokens: u32 = 0;
    let mut all_would_create: Vec<String> = vec![];

    let steps: Vec<serde_json::Value> = payload
        .commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            match registry.resolve(&cmd.command) {
                Some(spec) => {
                    let tokens = spec.token_estimate.unwrap_or(0);
                    total_tokens += tokens;

                    let would_create: Vec<String> = spec
                        .output_paths
                        .iter()
                        .map(|p| expand_plan_path(p, &cmd.options, path_config))
                        .collect();
                    all_would_create.extend(would_create.clone());

                    serde_json::json!({
                        "step": i + 1,
                        "command": cmd.command,
                        "package": spec.framework,
                        "token_estimate": tokens,
                        "would_create": would_create,
                        "next_steps": spec.next_steps,
                    })
                }
                None => serde_json::json!({
                    "step": i + 1,
                    "command": cmd.command,
                    "error": format!("Unknown command '{}' — run `tsx list` to see available commands", cmd.command),
                }),
            }
        })
        .collect();

    let duration_ms = start.elapsed().as_millis() as u64;
    ResponseEnvelope::success(
        "batch:plan",
        serde_json::json!({
            "steps": steps,
            "total_commands": payload.commands.len(),
            "total_token_estimate": total_tokens,
            "total_files": all_would_create.len(),
            "all_would_create": all_would_create,
        }),
        duration_ms,
    )
    .with_tokens_used(total_tokens)
}

/// Simple path template expander used for plan output (mirrors run.rs logic without
/// pulling in the full run machinery).
fn expand_plan_path(
    template: &str,
    options: &serde_json::Value,
    paths: Option<&crate::stack::PathConfig>,
) -> String {
    // Apply path prefix overrides first
    let expanded = if let Some(cfg) = paths {
        let overrides: &[(&str, Option<&str>)] = &[
            ("components/", Some(cfg.components.as_str())),
            ("routes/", Some(cfg.routes.as_str())),
            ("db/", Some(cfg.db.as_str())),
            ("server-functions/", Some(cfg.server_fns.as_str())),
            ("hooks/", Some(cfg.hooks.as_str())),
        ];
        let mut t = template.to_string();
        for (prefix, override_dir) in overrides {
            if let Some(dir) = override_dir {
                if t.starts_with(prefix) {
                    t = format!("{}/{}", dir.trim_end_matches('/'), &t[prefix.len()..]);
                    break;
                }
            }
        }
        t
    } else {
        template.to_string()
    };

    // Then expand {{field}} placeholders from options
    let Some(obj) = options.as_object() else {
        return expanded;
    };
    let mut result = expanded;
    for (key, value) in obj {
        if key.starts_with("__") {
            continue;
        }
        let placeholder = format!("{{{{{}}}}}", key);
        if let Some(s) = value.as_str() {
            result = result.replace(&placeholder, s);
        }
    }
    result
}
