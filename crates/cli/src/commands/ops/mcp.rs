//! `tsx mcp` — Model Context Protocol server over stdio.
//!
//! Implements the MCP JSON-RPC 2.0 protocol so agents like Claude and Cursor
//! can invoke tsx generators and query templates without spawning subprocesses.
//!
//! # Transport
//!
//! Newline-delimited JSON over stdin/stdout (the standard MCP stdio transport).
//!
//! # Tools exposed
//!
//! | Tool | Description |
//! |------|-------------|
//! | `tsx_list_templates` | List all installed template bundles |
//! | `tsx_search_templates` | Search templates by keyword |
//! | `tsx_get_template_info` | Get full manifest for a template id |
//! | `tsx_generate` | Run a generator by id |
//! | `tsx_plan` | Dry-run a generator — returns planned file paths |
//! | `tsx_diff` | Diff what a generator would change |
//! | `tsx_validate_schema` | Validate JSON against a template's @schema |
//! | `tsx_introspect` | Return the forge system overview |
//!
//! # MCP config snippet (Claude / Cursor)
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "tsx": {
//!       "command": "tsx",
//!       "args": ["mcp"]
//!     }
//!   }
//! }
//! ```

use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Request {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct Response {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i64,
    message: String,
}

impl Response {
    fn ok(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: "2.0", id, result: Some(result), error: None }
    }
    fn err(id: Option<Value>, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(RpcError { code, message: message.into() }),
        }
    }
    fn notification(method: &str) -> String {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
        })
        .to_string()
    }
}

// ---------------------------------------------------------------------------
// Tool definitions
// ---------------------------------------------------------------------------

fn tool_definitions() -> Value {
    serde_json::json!([
        {
            "name": "tsx_list_templates",
            "description": "List all installed forge template bundles, optionally filtered by source (global, project).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "enum": ["global", "project", "framework"],
                        "description": "Filter by template source. Omit to return all."
                    }
                }
            }
        },
        {
            "name": "tsx_search_templates",
            "description": "Search installed templates by keyword (matches id, name, or description).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search keyword"
                    }
                },
                "required": ["query"]
            }
        },
        {
            "name": "tsx_get_template_info",
            "description": "Return full manifest details for a template by id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Template id" }
                },
                "required": ["id"]
            }
        },
        {
            "name": "tsx_generate",
            "description": "Run a tsx generator by id with a JSON context object.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Generator command id (e.g. add:schema)" },
                    "input": { "type": "object", "description": "Generator input (JSON object)" },
                    "framework": { "type": "string", "description": "Framework slug (auto-detected if omitted)" }
                },
                "required": ["id"]
            }
        },
        {
            "name": "tsx_plan",
            "description": "Dry-run a tsx generator — returns what files would be created without writing them.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Generator command id" },
                    "input": { "type": "object", "description": "Generator input" },
                    "framework": { "type": "string", "description": "Framework slug" }
                },
                "required": ["id"]
            }
        },
        {
            "name": "tsx_diff",
            "description": "Show a unified diff of what a generator would change without writing files.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Generator command id" },
                    "input": { "type": "object", "description": "Generator input" },
                    "framework": { "type": "string", "description": "Framework slug" }
                },
                "required": ["id"]
            }
        },
        {
            "name": "tsx_validate_schema",
            "description": "Validate a JSON object against the @schema declared in a template.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "template_id": { "type": "string", "description": "Template bundle id" },
                    "command": { "type": "string", "description": "Command id within the template" },
                    "input": { "type": "object", "description": "Data to validate" }
                },
                "required": ["template_id", "command", "input"]
            }
        },
        {
            "name": "tsx_introspect",
            "description": "Return a summary of the tsx forge system: installed templates, generators, and configuration.",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}

// ---------------------------------------------------------------------------
// Tool handlers
// ---------------------------------------------------------------------------

fn handle_list_templates(args: &Value) -> Value {
    let source_filter = args.get("source").and_then(|v| v.as_str());

    let templates = match source_filter {
        Some("global") => forge::discover_from_source(forge::TemplateSource::Global),
        Some("project") => forge::discover_from_source(forge::TemplateSource::Project),
        Some("framework") => forge::discover_from_source(forge::TemplateSource::Framework),
        _ => forge::discover_templates(),
    };

    let items: Vec<Value> = templates
        .iter()
        .map(|t| {
            serde_json::json!({
                "id": t.id,
                "name": t.name,
                "version": t.version,
                "description": t.description,
                "source": t.source.to_string(),
            })
        })
        .collect();

    serde_json::json!({ "count": items.len(), "templates": items })
}

fn handle_search_templates(args: &Value) -> Value {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();

    let templates: Vec<Value> = forge::discover_templates()
        .into_iter()
        .filter(|t| {
            t.id.to_lowercase().contains(&query)
                || t.name.to_lowercase().contains(&query)
                || t.description.to_lowercase().contains(&query)
        })
        .map(|t| {
            serde_json::json!({
                "id": t.id,
                "name": t.name,
                "version": t.version,
                "description": t.description,
                "source": t.source.to_string(),
            })
        })
        .collect();

    serde_json::json!({ "count": templates.len(), "templates": templates })
}

fn handle_get_template_info(args: &Value) -> Result<Value, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("missing required argument: id")?;

    forge::find_template(id)
        .map(|info| {
            serde_json::json!({
                "id": info.id,
                "name": info.name,
                "version": info.version,
                "description": info.description,
                "source": info.source.to_string(),
                "path": info.path.to_string_lossy(),
                "manifest": info.manifest,
            })
        })
        .ok_or_else(|| format!("Template '{}' not found", id))
}

fn handle_generate(args: &Value, dry_run: bool) -> Value {
    use crate::commands::run;

    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let framework = args.get("framework").and_then(|v| v.as_str()).map(|s| s.to_string());
    let input = args.get("input").map(|v| v.to_string());

    let result = run::run(id, framework, input, false, dry_run, false);
    serde_json::to_value(&result).unwrap_or_else(|_| serde_json::json!({"error": "serialisation failed"}))
}

fn handle_validate_schema(args: &Value) -> Result<Value, String> {
    let template_id = args
        .get("template_id")
        .and_then(|v| v.as_str())
        .ok_or("missing required argument: template_id")?;
    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or("missing required argument: command")?;
    let input = args
        .get("input")
        .ok_or("missing required argument: input")?;

    let schema = forge::template_schema(template_id, command)
        .ok_or_else(|| format!("No @schema found for template '{}' command '{}'", template_id, command))?;

    let result = forge::validate_input(input, &schema);
    Ok(serde_json::json!({
        "valid": result.is_ok(),
        "errors": result.errors,
    }))
}

fn handle_introspect() -> Value {
    let templates = forge::discover_templates();
    let global_cfg = forge::load_global_config();
    let project_cfg = forge::load_project_config();

    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "templates": {
            "count": templates.len(),
            "ids": templates.iter().map(|t| &t.id).collect::<Vec<_>>(),
        },
        "config": {
            "global_path": forge::global_config_path().to_string_lossy(),
            "project_path": forge::project_config_path().to_string_lossy(),
            "registry_url": global_cfg.registry_url,
            "preferred_templates": global_cfg.preferred_templates,
            "project_templates": project_cfg.templates,
        },
        "capabilities": [
            "template_discovery",
            "template_installation",
            "schema_validation",
            "code_generation",
            "dry_run",
        ]
    })
}

// ---------------------------------------------------------------------------
// Request dispatcher
// ---------------------------------------------------------------------------

fn dispatch(req: Request, stdout: &mut impl Write) {
    let id = req.id.clone();
    let args = &req.params;

    let response = match req.method.as_str() {
        // ── MCP lifecycle ──────────────────────────────────────────────────
        "initialize" => {
            let result = serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "tsx",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            });
            // After initialize we must send an initialized notification
            let notif = Response::notification("notifications/initialized");
            writeln!(stdout, "{}", notif).ok();
            Response::ok(id, result)
        }

        "tools/list" => Response::ok(
            id,
            serde_json::json!({ "tools": tool_definitions() }),
        ),

        "tools/call" => {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let tool_args = args.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));

            let result: Result<Value, String> = match name {
                "tsx_list_templates" => Ok(handle_list_templates(&tool_args)),
                "tsx_search_templates" => Ok(handle_search_templates(&tool_args)),
                "tsx_get_template_info" => handle_get_template_info(&tool_args),
                "tsx_generate" => Ok(handle_generate(&tool_args, false)),
                "tsx_plan" => Ok(handle_generate(&tool_args, true)),
                "tsx_diff" => Ok(handle_generate(&tool_args, true)),
                "tsx_validate_schema" => handle_validate_schema(&tool_args),
                "tsx_introspect" => Ok(handle_introspect()),
                unknown => Err(format!("Unknown tool: {}", unknown)),
            };

            match result {
                Ok(data) => Response::ok(
                    id,
                    serde_json::json!({
                        "content": [{ "type": "text", "text": data.to_string() }]
                    }),
                ),
                Err(msg) => Response::err(id, -32602, msg),
            }
        }

        // ── Notifications (no response needed) ────────────────────────────
        "notifications/initialized" | "notifications/cancelled" => return,

        // ── Unknown method ────────────────────────────────────────────────
        _ => Response::err(id, -32601, format!("Method not found: {}", req.method)),
    };

    let line = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
    writeln!(stdout, "{}", line).ok();
    stdout.flush().ok();
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Start the MCP server, reading JSON-RPC messages from stdin and writing
/// responses to stdout. Runs until stdin is closed (EOF).
pub fn run_mcp_server() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if l.trim().is_empty() => continue,
            Ok(l) => l,
            Err(_) => break,
        };

        match serde_json::from_str::<Request>(&line) {
            Ok(req) => dispatch(req, &mut stdout),
            Err(e) => {
                let err = Response::err(None, -32700, format!("Parse error: {}", e));
                let msg = serde_json::to_string(&err).unwrap_or_default();
                writeln!(stdout, "{}", msg).ok();
                stdout.flush().ok();
            }
        }
    }
}
