//! OpenAPI 3.x JSON → Zod schemas.

use std::collections::HashMap;
use crate::types::{CodegenError, CodegenInput, CodegenOutput};

pub fn convert(input: CodegenInput, out_filename: Option<&str>) -> Result<CodegenOutput, CodegenError> {
    let src = input.read()?;
    let spec: serde_json::Value = serde_json::from_str(&src)
        .map_err(|e| CodegenError::ParseError(format!("Invalid OpenAPI JSON: {}", e)))?;

    let schemas = spec
        .pointer("/components/schemas")
        .and_then(|v| v.as_object())
        .ok_or_else(|| CodegenError::ParseError("No /components/schemas found".into()))?;

    let mut lines = vec![
        "// AUTO-GENERATED from OpenAPI spec".to_string(),
        r#"import { z } from "zod""#.to_string(),
        String::new(),
    ];
    let mut exported_names = Vec::new();
    let mut warnings = Vec::new();

    for (name, schema) in schemas {
        let schema_name = format!("{}Schema", pascal_case(name));
        match openapi_schema_to_zod(schema, schemas, &mut warnings) {
            Ok(zod_expr) => {
                lines.push(format!("export const {} = {}", schema_name, zod_expr));
                lines.push(String::new());
                lines.push(format!("export type {} = z.infer<typeof {}>", pascal_case(name), schema_name));
                lines.push(String::new());
                exported_names.push(schema_name);
                exported_names.push(pascal_case(name));
            }
            Err(e) => {
                warnings.push(format!("Skipped {}: {}", name, e));
                lines.push(format!("// TODO: {} — skipped ({})", name, e));
                lines.push(String::new());
            }
        }
    }

    Ok(CodegenOutput {
        content: lines.join("\n"),
        filename: out_filename.unwrap_or("generated/api-schemas.ts").to_string(),
        exported_names,
        warnings,
    })
}

fn openapi_schema_to_zod(
    schema: &serde_json::Value,
    all_schemas: &serde_json::Map<String, serde_json::Value>,
    warnings: &mut Vec<String>,
) -> Result<String, String> {
    // $ref → forward reference
    if let Some(r) = schema.get("$ref").and_then(|v| v.as_str()) {
        let ref_name = r.split('/').last().unwrap_or(r);
        return Ok(format!("{}Schema", pascal_case(ref_name)));
    }

    let ty = schema.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match ty {
        "string" => {
            let mut s = "z.string()".to_string();
            if let Some(fmt) = schema.get("format").and_then(|v| v.as_str()) {
                match fmt {
                    "date-time" | "datetime" => s.push_str(".datetime()"),
                    "email" => s.push_str(".email()"),
                    "uri" | "url" => s.push_str(".url()"),
                    "uuid" => s.push_str(".uuid()"),
                    _ => {}
                }
            }
            if let Some(min) = schema.get("minLength").and_then(|v| v.as_u64()) {
                s.push_str(&format!(".min({})", min));
            }
            if let Some(max) = schema.get("maxLength").and_then(|v| v.as_u64()) {
                s.push_str(&format!(".max({})", max));
            }
            Ok(s)
        }
        "integer" | "number" => {
            let mut s = "z.number()".to_string();
            if ty == "integer" { s.push_str(".int()"); }
            Ok(s)
        }
        "boolean" => Ok("z.boolean()".into()),
        "array" => {
            if let Some(items) = schema.get("items") {
                let inner = openapi_schema_to_zod(items, all_schemas, warnings)?;
                Ok(format!("z.array({})", inner))
            } else {
                Ok("z.array(z.unknown())".into())
            }
        }
        "object" | "" => {
            let props = schema.get("properties").and_then(|v| v.as_object());
            let required_fields: Vec<&str> = schema
                .get("required")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();

            if let Some(props) = props {
                let mut field_lines: Vec<String> = Vec::new();
                for (field, field_schema) in props {
                    match openapi_schema_to_zod(field_schema, all_schemas, warnings) {
                        Ok(mut zod) => {
                            if !required_fields.contains(&field.as_str()) {
                                zod.push_str(".optional()");
                            }
                            field_lines.push(format!("  {}: {}", field, zod));
                        }
                        Err(e) => {
                            warnings.push(format!("Field {}: {}", field, e));
                            field_lines.push(format!("  {}: z.unknown() /* {} */", field, e));
                        }
                    }
                }
                Ok(format!("z.object({{\n{}\n}})", field_lines.join(",\n")))
            } else {
                Ok("z.record(z.string(), z.unknown())".into())
            }
        }
        other => Err(format!("unsupported type '{}'", other)),
    }
}

fn pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_simple_openapi_schema() {
        let spec = serde_json::json!({
            "components": {
                "schemas": {
                    "User": {
                        "type": "object",
                        "required": ["id", "name"],
                        "properties": {
                            "id": { "type": "string", "format": "uuid" },
                            "name": { "type": "string" },
                            "age": { "type": "integer" }
                        }
                    }
                }
            }
        });
        let out = convert(CodegenInput::source(spec.to_string()), None).unwrap();
        assert!(out.content.contains("z.string().uuid()"));
        assert!(out.content.contains("z.number().int().optional()"));
        assert!(out.content.contains("UserSchema"));
    }
}
