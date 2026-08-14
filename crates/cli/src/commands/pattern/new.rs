use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

/// Scaffold a new pack directory with a starter `pack.json` and `main.forge`.
pub fn pattern_new(
    id: String,
    name: Option<String>,
    description: Option<String>,
    framework: Option<String>,
    _verbose: bool,
) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let pack_dir = forge::PackManifest::dir(&cwd, &id);

    if pack_dir.exists() {
        return ResponseEnvelope::error(
            "pattern new",
            ErrorResponse::new(ErrorCode::ValidationError, format!("Pack '{}' already exists at {}", id, pack_dir.display())),
            0,
        );
    }

    let mut commands = std::collections::HashMap::new();
    commands.insert("all".to_string(), forge::PackCommand {
        description: "Generate all outputs".to_string(),
        outputs: vec!["main".to_string()],
        default: true,
    });

    let pack = forge::PackManifest {
        id: id.clone(),
        name: name.unwrap_or_else(|| id.clone()),
        version: "1.0.0".to_string(),
        description: description.unwrap_or_else(|| format!("Pattern pack: {}", id)),
        author: String::new(),
        framework: framework.unwrap_or_default(),
        tags: Vec::new(),
        args: vec![forge::PackArg {
            name: "name".to_string(),
            arg_type: "string".to_string(),
            required: true,
            default: None,
            description: "Feature name".to_string(),
            options: Vec::new(),
        }],
        outputs: vec![forge::PackOutput {
            id: "main".to_string(),
            template: "main.forge".to_string(),
            path: "src/{{ name | snake_case }}.ts".to_string(),
        }],
        commands,
        markers: Vec::new(),
        post_hooks: std::collections::HashMap::new(),
    };

    if let Err(e) = pack.save(&cwd) {
        return ResponseEnvelope::error(
            "pattern new",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        );
    }

    let forge_content = "// {{ name | pascal_case }}\nexport const {{ name | pascal_case }} = () => {\n  // TODO: implement\n};\n";
    let forge_path = pack_dir.join("main.forge");
    if let Err(e) = std::fs::write(&forge_path, forge_content) {
        return ResponseEnvelope::error(
            "pattern new",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        );
    }

    ResponseEnvelope::success(
        "pattern new",
        serde_json::json!({
            "id": id,
            "pack_dir": pack_dir.to_string_lossy(),
            "files_created": ["pack.json", "main.forge"],
        }),
        0,
    )
    .with_next_steps(vec![
        format!("Edit the template at {}", forge_path.display()),
        format!("Run with: tsx pattern run {}", id),
    ])
}
