use std::time::Instant;

use crate::framework::command_registry::CommandRegistry;
use crate::json::payload::BatchPayload;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

use super::exec::execute_command;
use super::types::{BatchCommandResult, BatchError, BatchResult};

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
    // All file paths written so far, for rollback.
    let mut all_files_written: Vec<String> = Vec::new();

    for (index, cmd) in payload.commands.iter().enumerate() {
        let cmd_start = Instant::now();
        let result = execute_command(&cmd.command, &cmd.options, overwrite, dry_run);
        let cmd_duration_ms = cmd_start.elapsed().as_millis() as u64;

        let batch_cmd_result = match result {
            Ok(files_created) => {
                all_files_written.extend(files_created.clone());
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
            Err((code, message)) => {
                failed += 1;
                BatchCommandResult {
                    index: index as u32,
                    success: false,
                    result: None,
                    error: Some(BatchError {
                        code,
                        message,
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

    // Rollback: delete all files written by earlier commands if requested.
    let mut rolled_back_files: Vec<String> = Vec::new();
    if failed > 0 && payload.rollback_on_failure && !dry_run {
        for path in &all_files_written {
            if std::fs::remove_file(path).is_ok() {
                rolled_back_files.push(path.clone());
            }
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    // Sum token estimates for all commands that were resolved via the registry.
    let registry = CommandRegistry::load_all();
    let total_tokens: u32 = payload
        .commands
        .iter()
        .filter_map(|cmd| registry.resolve(&cmd.command)?.token_estimate)
        .sum();

    let batch_result = BatchResult {
        total,
        succeeded,
        failed,
        results,
        rolled_back_files,
    };

    let response = ResponseEnvelope::success(
        "batch",
        serde_json::to_value(batch_result).unwrap(),
        duration_ms,
    )
    .with_tokens_used(total_tokens);

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
