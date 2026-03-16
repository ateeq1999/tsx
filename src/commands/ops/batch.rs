use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::json::payload::BatchPayload;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub results: Vec<BatchCommandResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCommandResult {
    pub index: u32,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BatchError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

pub fn batch(
    payload: BatchPayload,
    overwrite: bool,
    dry_run: bool,
    verbose: bool,
    stream: bool,
) -> CommandResult {
    let start = Instant::now();

    let total = payload.commands.len() as u32;
    let mut succeeded: u32 = 0;
    let mut failed: u32 = 0;
    let mut results: Vec<BatchCommandResult> = Vec::new();

    for (index, cmd) in payload.commands.iter().enumerate() {
        let cmd_start = Instant::now();
        let result = execute_command(&cmd.command, &cmd.options, overwrite, dry_run);
        let cmd_duration_ms = cmd_start.elapsed().as_millis() as u64;

        let batch_cmd_result = match result {
            Ok(files_created) => {
                succeeded += 1;
                BatchCommandResult {
                    index: index as u32,
                    success: true,
                    result: Some(serde_json::json!({
                        "kind": cmd.command.clone(),
                        "files": files_created,
                        "duration_ms": cmd_duration_ms,
                    })),
                    error: None,
                }
            }
            Err(e) => {
                failed += 1;
                BatchCommandResult {
                    index: index as u32,
                    success: false,
                    result: None,
                    error: Some(BatchError {
                        code: e.0,
                        message: e.1,
                        path: None,
                    }),
                }
            }
        };

        if stream {
            // Emit each result immediately as a newline-delimited JSON event.
            if let Ok(line) = serde_json::to_string(&batch_cmd_result) {
                println!("{}", line);
            }
        }

        let should_stop = batch_cmd_result.error.is_some() && payload.stop_on_failure;
        results.push(batch_cmd_result);

        if should_stop {
            break;
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    let batch_result = BatchResult {
        total,
        succeeded,
        failed,
        results,
    };

    let response = ResponseEnvelope::success(
        "batch",
        serde_json::to_value(batch_result).unwrap(),
        duration_ms,
    );

    if !stream {
        if verbose {
            let context = crate::json::response::Context {
                project_root: std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                tsx_version: env!("CARGO_PKG_VERSION").to_string(),
            };
            response.with_context(context).print();
        } else {
            response.print();
        }
    } else {
        // In stream mode, emit a final summary line.
        if let Ok(summary) = serde_json::to_string(&serde_json::json!({
            "event": "batch_complete",
            "total": total,
            "succeeded": succeeded,
            "failed": failed,
            "duration_ms": duration_ms,
        })) {
            println!("{}", summary);
        }
    }

    CommandResult::ok("batch", vec![])
}

fn execute_command(
    command: &str,
    options: &serde_json::Value,
    overwrite: bool,
    dry_run: bool,
) -> Result<Vec<String>, (String, String)> {
    let options_str = serde_json::to_string(options)
        .map_err(|e| ("INVALID_PAYLOAD".to_string(), e.to_string()))?;

    macro_rules! dispatch {
        ($args_type:ty, $handler:expr) => {{
            let args: $args_type = serde_json::from_str(&options_str)
                .map_err(|e| ("VALIDATION_ERROR".to_string(), e.to_string()))?;
            let result = $handler(args, overwrite, dry_run);
            Ok(result.files_created)
        }};
    }

    match command {
        "add:schema" => dispatch!(crate::schemas::AddSchemaArgs, crate::commands::add_schema::add_schema),
        "add:server-fn" => dispatch!(crate::schemas::AddServerFnArgs, crate::commands::add_server_fn::add_server_fn),
        "add:query" => dispatch!(crate::schemas::AddQueryArgs, crate::commands::add_query::add_query),
        "add:form" => dispatch!(crate::schemas::AddFormArgs, crate::commands::add_form::add_form),
        "add:table" => dispatch!(crate::schemas::AddFormArgs, crate::commands::add_table::add_table),
        "add:page" => dispatch!(crate::schemas::AddPageArgs, crate::commands::add_page::add_page),
        "add:seed" => dispatch!(crate::schemas::AddSeedArgs, crate::commands::add_seed::add_seed),
        "add:feature" => dispatch!(crate::schemas::AddFeatureArgs, crate::commands::add_feature::add_feature),
        "add:auth" => dispatch!(crate::schemas::AddAuthArgs, crate::commands::add_auth::add_auth),
        "add:auth-guard" => dispatch!(crate::schemas::AddAuthGuardArgs, crate::commands::add_auth_guard::add_auth_guard),
        _ => Err((
            "UNKNOWN_COMMAND".to_string(),
            format!("Unknown command: {}", command),
        )),
    }
}
