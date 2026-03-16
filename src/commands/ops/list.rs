use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::framework::loader::FrameworkLoader;
use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult {
    pub templates: Option<Vec<TemplateInfo>>,
    pub generators: Option<Vec<GeneratorInfo>>,
    pub components: Option<Vec<ComponentInfo>>,
    pub frameworks: Option<Vec<FrameworkInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkInfo {
    pub slug: String,
    pub name: String,
    pub version: String,
    pub category: String,
    pub docs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorInfo {
    pub id: String,
    pub description: String,
    pub options: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub category: String,
    pub description: String,
    pub props: serde_json::Value,
    pub file: String,
}

pub fn list(kind: String, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let result = match kind.as_str() {
        "templates" => ListResult {
            templates: Some(vec![
                TemplateInfo {
                    id: "default".to_string(),
                    name: "Full Stack".to_string(),
                    description: "Complete TanStack Start app with auth, DB, and routing"
                        .to_string(),
                    path: "templates/default".to_string(),
                    files: vec![
                        "src/main.tsx".to_string(),
                        "routes/".to_string(),
                        "components/".to_string(),
                        "lib/".to_string(),
                    ],
                },
                TemplateInfo {
                    id: "minimal".to_string(),
                    name: "Minimal".to_string(),
                    description: "Minimal boilerplate to get started".to_string(),
                    path: "templates/minimal".to_string(),
                    files: vec!["src/main.tsx".to_string(), "routes/index.tsx".to_string()],
                },
            ]),
            generators: None,
            components: None,
            frameworks: None,
        },

        "generators" => ListResult {
            templates: None,
            generators: Some(vec![
                GeneratorInfo {
                    id: "add:feature".to_string(),
                    description: "Scaffold a complete CRUD feature module".to_string(),
                    options: serde_json::json!({
                        "name": { "type": "string", "required": true, "pattern": "^[a-z0-9-]+$" },
                        "fields": { "type": "array", "required": true },
                        "auth": { "type": "boolean", "default": false },
                        "paginated": { "type": "boolean", "default": true },
                        "operations": { "type": "array", "default": ["list", "create", "update", "delete"] }
                    }),
                },
                GeneratorInfo {
                    id: "add:schema".to_string(),
                    description: "Generate a Drizzle schema table definition".to_string(),
                    options: serde_json::json!({
                        "name": { "type": "string", "required": true },
                        "fields": { "type": "array", "required": true },
                        "timestamps": { "type": "boolean", "default": true },
                        "softDelete": { "type": "boolean", "default": false }
                    }),
                },
                GeneratorInfo {
                    id: "add:server-fn".to_string(),
                    description: "Generate a typed server function".to_string(),
                    options: serde_json::json!({
                        "name": { "type": "string", "required": true },
                        "table": { "type": "string", "required": true },
                        "operation": { "type": "string", "enum": ["list", "create", "update", "delete"], "required": true }
                    }),
                },
                GeneratorInfo {
                    id: "add:query".to_string(),
                    description: "Generate a TanStack Query hook".to_string(),
                    options: serde_json::json!({
                        "name": { "type": "string", "required": true },
                        "serverFn": { "type": "string", "required": true }
                    }),
                },
                GeneratorInfo {
                    id: "add:form".to_string(),
                    description: "Generate a TanStack Form component".to_string(),
                    options: serde_json::json!({
                        "name": { "type": "string", "required": true },
                        "fields": { "type": "array", "required": true }
                    }),
                },
                GeneratorInfo {
                    id: "add:table".to_string(),
                    description: "Generate a TanStack Table component".to_string(),
                    options: serde_json::json!({
                        "name": { "type": "string", "required": true },
                        "columns": { "type": "array", "required": true }
                    }),
                },
                GeneratorInfo {
                    id: "add:page".to_string(),
                    description: "Add a new route page".to_string(),
                    options: serde_json::json!({
                        "path": { "type": "string", "required": true },
                        "title": { "type": "string" }
                    }),
                },
                GeneratorInfo {
                    id: "add:auth".to_string(),
                    description: "Configure Better Auth".to_string(),
                    options: serde_json::json!({
                        "providers": { "type": "array", "default": ["github", "google"] }
                    }),
                },
            ]),
            components: None,
            frameworks: None,
        },

        "components" => ListResult {
            templates: None,
            generators: None,
            components: Some(vec![
                ComponentInfo {
                    name: "button".to_string(),
                    category: "inputs".to_string(),
                    description: "Interactive button with multiple variants".to_string(),
                    props: serde_json::json!({
                        "variant": { "type": "enum", "values": ["primary", "secondary", "ghost", "destructive"], "default": "primary" },
                        "size": { "type": "enum", "values": ["sm", "md", "lg"], "default": "md" },
                        "disabled": { "type": "bool", "default": false },
                        "onclick": { "type": "callback" }
                    }),
                    file: "components/ui/button.tsx".to_string(),
                },
                ComponentInfo {
                    name: "input".to_string(),
                    category: "inputs".to_string(),
                    description: "Text input field with validation support".to_string(),
                    props: serde_json::json!({
                        "type": { "type": "enum", "values": ["text", "email", "password", "number"], "default": "text" },
                        "placeholder": { "type": "string" },
                        "value": { "type": "string" },
                        "onchange": { "type": "callback" }
                    }),
                    file: "components/ui/input.tsx".to_string(),
                },
                ComponentInfo {
                    name: "card".to_string(),
                    category: "layout".to_string(),
                    description: "Container component with header, content, and footer".to_string(),
                    props: serde_json::json!({
                        "class": { "type": "string" },
                        "children": { "type": "node" }
                    }),
                    file: "components/ui/card.tsx".to_string(),
                },
            ]),
            frameworks: None,
        },

        "frameworks" => {
            let mut loader = FrameworkLoader::default();
            let frameworks = loader.load_builtin_frameworks();
            ListResult {
                templates: None,
                generators: None,
                components: None,
                frameworks: Some(
                    frameworks
                        .into_iter()
                        .map(|f| FrameworkInfo {
                            slug: f.slug,
                            name: f.name,
                            version: f.version,
                            category: format!("{:?}", f.category).to_lowercase(),
                            docs: f.docs,
                        })
                        .collect(),
                ),
            }
        }

        _ => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::unknown_kind(&kind);
            ResponseEnvelope::error("list", error, duration_ms).print();
            return CommandResult::err("list", format!("Unknown kind: {}", kind));
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    let response =
        ResponseEnvelope::success("list", serde_json::to_value(result).unwrap(), duration_ms);

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

    CommandResult::ok("list", vec![])
}
