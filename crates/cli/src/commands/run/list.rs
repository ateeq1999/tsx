use std::time::Instant;

use crate::framework::command_registry::CommandRegistry;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

use super::types::RunListEntry;

/// List all available generators, optionally filtered to a single framework.
pub fn run_list(framework: Option<String>, verbose: bool) -> CommandResult {
    let _ = verbose;
    let start = Instant::now();
    let registry = CommandRegistry::load_all();

    let specs = match &framework {
        Some(fw) => registry.for_framework(fw),
        None => registry.all(),
    };

    let entries: Vec<RunListEntry> = specs
        .iter()
        .map(|s| RunListEntry {
            id: s.id.clone(),
            command: s.command.clone(),
            framework: s.framework.clone(),
            description: s.description.clone(),
            token_estimate: s.token_estimate,
            schema: s.schema.clone(),
        })
        .collect();

    let count = entries.len();
    let duration_ms = start.elapsed().as_millis() as u64;
    let payload = serde_json::json!({
        "generators": entries,
        "total": count,
    });

    ResponseEnvelope::success("run:list", payload, duration_ms).print();
    CommandResult::ok("run:list", vec![])
}
