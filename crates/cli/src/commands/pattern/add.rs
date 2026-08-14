use std::path::PathBuf;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::types::{PatternArg, PatternDefinition, PatternOutput};

pub fn pattern_add(
    name: String,
    description: Option<String>,
    template: Option<String>,
    args_spec: Option<String>,
    _verbose: bool,
) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Parse args spec: "name:string, entity:string, methods:string[]"
    let args = parse_args_spec(args_spec.as_deref().unwrap_or(""));

    // Determine output template name
    let template_file = template.as_deref().unwrap_or("template.forge");
    let template_base = PathBuf::from(template_file)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("template.forge")
        .to_string();

    let pattern = PatternDefinition {
        id: name.clone(),
        description: description.unwrap_or_else(|| format!("User-defined pattern: {}", name)),
        args,
        outputs: vec![PatternOutput {
            path: format!("{{{{paths.{}}}}}/{{{{kebab(name)}}}}.ts", name.replace('-', "_")),
            template: template_base.clone(),
        }],
        slots: Vec::new(),
        post_hooks: Vec::new(),
        version: "1.0.0".to_string(),
    };

    match pattern.save(&cwd) {
        Ok(_) => {
            let pattern_dir = PatternDefinition::dir(&cwd, &name);

            // Copy the template file into the pattern directory if it exists and is external
            if let Some(tmpl) = &template {
                let src = PathBuf::from(tmpl);
                if src.exists() && src != pattern_dir.join(&template_base) {
                    let _ = std::fs::copy(&src, pattern_dir.join(&template_base));
                }
            }

            ResponseEnvelope::success(
                "pattern add",
                serde_json::json!({
                    "id": name,
                    "manifest": PatternDefinition::manifest_path(&cwd, &name).to_string_lossy(),
                    "template_dir": pattern_dir.to_string_lossy(),
                    "pattern": serde_json::to_value(&pattern).unwrap_or_default(),
                }),
                0,
            )
            .with_next_steps(vec![
                format!("Edit the template at {}", pattern_dir.join(&template_base).display()),
                format!("Run the pattern with: tsx run {}", name),
                format!("Share it: tsx pattern share --name {}", name),
            ])
        }
        Err(e) => ResponseEnvelope::error(
            "pattern add",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

fn parse_args_spec(spec: &str) -> Vec<PatternArg> {
    if spec.trim().is_empty() {
        return Vec::new();
    }
    spec.split(',')
        .filter_map(|part| {
            let part = part.trim();
            if let Some(colon) = part.find(':') {
                let name = part[..colon].trim().to_string();
                let arg_type = part[colon + 1..].trim().to_string();
                if !name.is_empty() {
                    return Some(PatternArg { name, arg_type, description: None });
                }
            } else if !part.is_empty() {
                return Some(PatternArg {
                    name: part.to_string(),
                    arg_type: "string".to_string(),
                    description: None,
                });
            }
            None
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_spec_basic() {
        let args = parse_args_spec("name:string, entity:string, methods:string[]");
        assert_eq!(args.len(), 3);
        assert_eq!(args[0].name, "name");
        assert_eq!(args[1].arg_type, "string");
        assert_eq!(args[2].name, "methods");
    }
}
