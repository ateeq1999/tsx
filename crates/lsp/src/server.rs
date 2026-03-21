//! Language Server event loop — JSON-RPC over stdio.

use std::collections::HashMap;
use std::io::{self, BufReader, Write};

use lsp_types::{
    CompletionOptions, CompletionParams, CompletionResponse, HoverContents, HoverParams,
    InitializeResult, MarkupContent, MarkupKind, PublishDiagnosticsParams, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use serde_json::json;

use crate::completions::{stack_json_completions, template_completions};
use crate::diagnostics::check_template;
use crate::hover::hover_for_key;
use crate::transport::{read_message, write_message};

/// In-memory document store: URI → text content.
type DocStore = HashMap<String, String>;

/// Start the LSP server loop, reading from stdin and writing to stdout.
pub fn run_lsp_server() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    let mut docs: DocStore = HashMap::new();
    let mut shutdown = false;

    loop {
        let msg = match read_message(&mut reader) {
            Some(m) => m,
            None => break,
        };

        let method = msg.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let id = msg.get("id").cloned();
        let params = msg.get("params").cloned().unwrap_or(json!({}));

        match method {
            "initialize" => {
                let result = initialize_result();
                respond(&mut writer, &id, result)?;
            }
            "initialized" => {} // notification — no response needed
            "shutdown" => {
                shutdown = true;
                respond(&mut writer, &id, json!(null))?;
            }
            "exit" => {
                std::process::exit(if shutdown { 0 } else { 1 });
            }

            // ── Document sync ────────────────────────────────────────────────
            "textDocument/didOpen" => {
                if let Some((uri, text)) = extract_doc_open(&params) {
                    let diags = maybe_diagnostics(&uri, &text);
                    docs.insert(uri.clone(), text);
                    if let Some(diags) = diags {
                        publish_diagnostics(&mut writer, &uri, diags)?;
                    }
                }
            }
            "textDocument/didChange" => {
                if let Some((uri, text)) = extract_doc_change(&params) {
                    let diags = maybe_diagnostics(&uri, &text);
                    docs.insert(uri.clone(), text);
                    if let Some(diags) = diags {
                        publish_diagnostics(&mut writer, &uri, diags)?;
                    }
                }
            }
            "textDocument/didClose" => {
                if let Some(uri) = params
                    .pointer("/textDocument/uri")
                    .and_then(|v| v.as_str())
                {
                    docs.remove(uri);
                    // Clear diagnostics on close
                    publish_diagnostics(&mut writer, uri, vec![])?;
                }
            }

            // ── Completion ───────────────────────────────────────────────────
            "textDocument/completion" => {
                if let Ok(p) = serde_json::from_value::<CompletionParams>(params.clone()) {
                    let uri = p.text_document_position.text_document.uri.as_str().to_string();
                    let text = docs.get(&uri).map(|s| s.as_str()).unwrap_or("");
                    let prefix = cursor_prefix(text, &p.text_document_position.position);
                    let items = completion_items_for_uri(&uri, &prefix);
                    let resp = CompletionResponse::Array(items);
                    respond(&mut writer, &id, serde_json::to_value(resp).unwrap())?;
                } else {
                    respond(&mut writer, &id, json!([]))?;
                }
            }

            // ── Hover ────────────────────────────────────────────────────────
            "textDocument/hover" => {
                if let Ok(p) = serde_json::from_value::<HoverParams>(params.clone()) {
                    let uri = p.text_document_position_params.text_document.uri.as_str().to_string();
                    let text = docs.get(&uri).map(|s| s.as_str()).unwrap_or("");
                    let word = word_at(text, &p.text_document_position_params.position);
                    let doc = hover_for_key(&word);
                    if let Some(contents) = doc {
                        let hover = lsp_types::Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: contents.to_string(),
                            }),
                            range: None,
                        };
                        respond(&mut writer, &id, serde_json::to_value(hover).unwrap())?;
                    } else {
                        respond(&mut writer, &id, json!(null))?;
                    }
                } else {
                    respond(&mut writer, &id, json!(null))?;
                }
            }

            _ => {
                // Unknown request — send MethodNotFound if it has an id
                if id.is_some() {
                    error_response(&mut writer, &id, -32601, "MethodNotFound")?;
                }
            }
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn initialize_result() -> serde_json::Value {
    let result = InitializeResult {
        capabilities: ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::FULL,
            )),
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec![
                    "\"".to_string(), ".".to_string(), "@".to_string(),
                ]),
                resolve_provider: Some(false),
                ..Default::default()
            }),
            hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
            ..Default::default()
        },
        server_info: Some(ServerInfo {
            name: "tsx-lsp".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
    };
    serde_json::to_value(result).unwrap()
}

fn respond(writer: &mut impl Write, id: &Option<serde_json::Value>, result: serde_json::Value) -> io::Result<()> {
    let msg = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    });
    write_message(writer, &msg)
}

fn error_response(writer: &mut impl Write, id: &Option<serde_json::Value>, code: i32, message: &str) -> io::Result<()> {
    let msg = json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    });
    write_message(writer, &msg)
}

fn publish_diagnostics(
    writer: &mut impl Write,
    uri: &str,
    diagnostics: Vec<lsp_types::Diagnostic>,
) -> io::Result<()> {
    let uri_val: serde_json::Value = json!(uri);
    let params = PublishDiagnosticsParams {
        uri: serde_json::from_value(uri_val).unwrap_or_else(|_| {
            serde_json::from_value(json!("file:///unknown")).unwrap()
        }),
        diagnostics,
        version: None,
    };
    let msg = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": serde_json::to_value(params).unwrap(),
    });
    write_message(writer, &msg)
}

fn maybe_diagnostics(uri: &str, text: &str) -> Option<Vec<lsp_types::Diagnostic>> {
    let lower = uri.to_lowercase();
    if lower.ends_with(".forge") || lower.ends_with(".jinja") || lower.ends_with(".jinja2") {
        Some(check_template(text))
    } else {
        None
    }
}

fn completion_items_for_uri(uri: &str, prefix: &str) -> Vec<lsp_types::CompletionItem> {
    let lower = uri.to_lowercase();
    if lower.ends_with(".forge") || lower.ends_with(".jinja") || lower.ends_with(".jinja2") {
        template_completions()
    } else if lower.ends_with("stack.json") || lower.ends_with("user-stack.json") {
        stack_json_completions(prefix)
    } else {
        vec![]
    }
}

fn extract_doc_open(params: &serde_json::Value) -> Option<(String, String)> {
    let uri = params.pointer("/textDocument/uri")?.as_str()?.to_string();
    let text = params.pointer("/textDocument/text")?.as_str()?.to_string();
    Some((uri, text))
}

fn extract_doc_change(params: &serde_json::Value) -> Option<(String, String)> {
    let uri = params.pointer("/textDocument/uri")?.as_str()?.to_string();
    // Full sync: take the last content change
    let text = params
        .pointer("/contentChanges/0/text")?
        .as_str()?
        .to_string();
    Some((uri, text))
}

/// Extract the "current JSON key prefix" at the cursor position.
/// E.g. in `"style": { "qu`, returns `"style.qu"`.
/// Very approximate — good enough for field key completions.
fn cursor_prefix(text: &str, pos: &lsp_types::Position) -> String {
    let line = text.lines().nth(pos.line as usize).unwrap_or("");
    let ch = pos.character as usize;
    let before = &line[..ch.min(line.len())];
    // Find last `"` before cursor
    if let Some(start) = before.rfind('"') {
        before[start + 1..].to_string()
    } else {
        String::new()
    }
}

/// Extract the JSON key word at the cursor position.
fn word_at(text: &str, pos: &lsp_types::Position) -> String {
    let line = text.lines().nth(pos.line as usize).unwrap_or("");
    let ch = pos.character as usize;
    let before = &line[..ch.min(line.len())];
    let after = if ch < line.len() { &line[ch..] } else { "" };
    let start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
        .map(|i| i + 1)
        .unwrap_or(0);
    let end = after.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
        .unwrap_or(after.len());
    format!("{}{}", &before[start..], &after[..end])
}
