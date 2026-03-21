//! Rust structs/enums → Zod schemas + TypeScript type aliases.

use crate::types::{CodegenError, CodegenInput, CodegenOutput, rust_type_to_zod, rust_type_to_ts};

/// Configuration for the rust→zod pipeline.
#[derive(Debug, Clone)]
pub struct RustToZodConfig {
    /// Generate `export type X = z.infer<typeof XSchema>` (default: true)
    pub emit_type_aliases: bool,
    /// Source file path for the AUTO-GENERATED comment
    pub source_path: Option<String>,
}

impl Default for RustToZodConfig {
    fn default() -> Self {
        Self { emit_type_aliases: true, source_path: None }
    }
}

/// Convert Rust source code to Zod schemas.
pub fn convert(input: CodegenInput, config: &RustToZodConfig) -> Result<CodegenOutput, CodegenError> {
    let source = input.read()?;
    let items = parse_rust_items(&source);
    if items.is_empty() {
        return Err(CodegenError::ParseError(
            "No public structs or enums with #[derive(Serialize)] found".into(),
        ));
    }

    let mut lines: Vec<String> = Vec::new();
    let source_comment = config.source_path.as_deref().unwrap_or("Rust source");
    lines.push(format!("// AUTO-GENERATED from {}", source_comment));
    lines.push(r#"import { z } from "zod""#.to_string());
    lines.push(String::new());

    let mut exported_names: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for item in &items {
        match item {
            RustItem::Struct(s) => {
                let schema_name = format!("{}Schema", s.name);
                lines.push(format!("export const {} = z.object({{", schema_name));
                for field in &s.fields {
                    let zod = rust_type_to_zod(&field.ty);
                    let zod = if field.serde_default { format!("{}.default(null)", zod) } else { zod };
                    let zod = if field.serde_skip_none { format!("{}.optional()", zod) } else { zod };
                    lines.push(format!("  {}: {},", field.name, zod));
                }
                lines.push(String::from("})"));
                lines.push(String::new());

                if config.emit_type_aliases {
                    lines.push(format!("export type {} = z.infer<typeof {}>", s.name, schema_name));
                    lines.push(String::new());
                }

                exported_names.push(schema_name);
                if config.emit_type_aliases {
                    exported_names.push(s.name.clone());
                }
            }
            RustItem::Enum(e) => {
                if e.variants.iter().all(|v| v.fields.is_empty()) {
                    // Unit enum → z.enum([...])
                    let schema_name = format!("{}Schema", e.name);
                    let variants = e.variants.iter()
                        .map(|v| format!("\"{}\"", v.name))
                        .collect::<Vec<_>>()
                        .join(", ");
                    lines.push(format!("export const {} = z.enum([{}])", schema_name, variants));
                    lines.push(String::new());
                    if config.emit_type_aliases {
                        lines.push(format!("export type {} = z.infer<typeof {}>", e.name, schema_name));
                        lines.push(String::new());
                    }
                    exported_names.push(schema_name);
                    if config.emit_type_aliases { exported_names.push(e.name.clone()); }
                } else {
                    warnings.push(format!(
                        "Enum '{}' has tuple/struct variants — emitting z.unknown() placeholder",
                        e.name
                    ));
                    lines.push(format!("// TODO: complex enum {}", e.name));
                    lines.push(format!("export const {}Schema = z.unknown()", e.name));
                    lines.push(String::new());
                }
            }
        }
    }

    Ok(CodegenOutput {
        content: lines.join("\n"),
        filename: "generated/schemas.ts".to_string(),
        exported_names,
        warnings,
    })
}

// ---------------------------------------------------------------------------
// Minimal Rust parser (hand-written, no syn dependency)
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct StructField {
    pub name: String,
    pub ty: String,
    pub serde_default: bool,
    pub serde_skip_none: bool,
}

#[derive(Debug)]
pub struct RustStruct {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(Debug)]
pub struct RustEnum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug)]
pub enum RustItem {
    Struct(RustStruct),
    Enum(RustEnum),
}

/// Expose parse result for rust_to_ts
pub fn parse_items_pub(src: &str) -> Vec<RustItem> {
    parse_rust_items(src)
}

fn parse_rust_items(src: &str) -> Vec<RustItem> {
    let mut items = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for `pub struct Name {` (possibly with closing `}` on same line)
        if (line.starts_with("pub struct ") || line.starts_with("pub(crate) struct ")) && line.contains('{') {
            let name = extract_name(line, "struct").unwrap_or_default();
            let fields = parse_struct_fields(&lines, &mut i);
            if !name.is_empty() {
                items.push(RustItem::Struct(RustStruct { name, fields }));
            }
            continue;
        }

        // Look for `pub enum Name {`
        if (line.starts_with("pub enum ") || line.starts_with("pub(crate) enum ")) && line.contains('{') {
            let name = extract_name(line, "enum").unwrap_or_default();
            let variants = parse_enum_variants(&lines, &mut i);
            if !name.is_empty() {
                items.push(RustItem::Enum(RustEnum { name, variants }));
            }
            continue;
        }

        i += 1;
    }

    items
}

fn extract_name(line: &str, keyword: &str) -> Option<String> {
    let after = line.split(keyword).nth(1)?.trim();
    // Take until first whitespace, '<', or '{'
    let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    if name.is_empty() { None } else { Some(name) }
}

fn parse_struct_fields(lines: &[&str], i: &mut usize) -> Vec<StructField> {
    let mut fields = Vec::new();
    let mut serde_default = false;
    let mut serde_skip_none = false;

    // Check if the opening line itself contains all fields (single-line struct)
    let opening_line = lines[*i].trim();
    if opening_line.contains('{') && opening_line.contains('}') {
        // Single-line: `pub struct Foo { pub x: i32, pub y: String }`
        let body_start = opening_line.find('{').unwrap() + 1;
        let body_end = opening_line.rfind('}').unwrap();
        let body = &opening_line[body_start..body_end];
        for part in body.split(',') {
            let part = part.trim();
            if part.starts_with("pub ") && part.contains(':') {
                if let Some(f) = parse_field_line(part, false, false) {
                    fields.push(f);
                }
            }
        }
        *i += 1;
        return fields;
    }

    *i += 1;
    while *i < lines.len() {
        let line = lines[*i].trim();
        if line == "}" || line.starts_with('}') { *i += 1; break; }

        if line.contains("#[serde(default") { serde_default = true; }
        if line.contains("skip_serializing_if") { serde_skip_none = true; }
        if line.starts_with("pub ") && line.contains(':') {
            if let Some(field) = parse_field_line(line, serde_default, serde_skip_none) {
                fields.push(field);
                serde_default = false;
                serde_skip_none = false;
            }
        }
        *i += 1;
    }
    fields
}

fn parse_field_line(line: &str, serde_default: bool, serde_skip_none: bool) -> Option<StructField> {
    let without_pub = line.trim_start_matches("pub ").trim_start_matches("pub(crate) ");
    let colon = without_pub.find(':')?;
    let name = without_pub[..colon].trim().to_string();
    let ty_raw = without_pub[colon + 1..].trim().trim_end_matches(',').to_string();
    Some(StructField { name, ty: ty_raw, serde_default, serde_skip_none })
}

fn parse_enum_variants(lines: &[&str], i: &mut usize) -> Vec<EnumVariant> {
    let mut variants = Vec::new();
    *i += 1;
    while *i < lines.len() {
        let line = lines[*i].trim();
        if line == "}" || line.starts_with('}') { *i += 1; break; }
        if !line.is_empty() && !line.starts_with("//") && !line.starts_with('#') {
            let name: String = line.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                let has_fields = line.contains('(') || line.contains('{');
                variants.push(EnumVariant {
                    name,
                    fields: if has_fields { vec!["_".to_string()] } else { vec![] },
                });
            }
        }
        *i += 1;
    }
    variants
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
#[derive(Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub download_count: i64,
    #[serde(default)]
    pub star_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated_message: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub enum Status {
    Active,
    Deprecated,
    Archived,
}
"#;

    #[test]
    fn converts_struct_to_zod() {
        let out = convert(CodegenInput::source(SAMPLE), &Default::default()).unwrap();
        assert!(out.content.contains("z.object("));
        assert!(out.content.contains("z.string()"));
        assert!(out.content.contains("z.number().int()"));
        assert!(out.content.contains("z.array(z.string())"));
        assert!(out.content.contains("export type Package"));
    }

    #[test]
    fn converts_unit_enum_to_z_enum() {
        let out = convert(CodegenInput::source(SAMPLE), &Default::default()).unwrap();
        assert!(out.content.contains("z.enum(["));
        assert!(out.content.contains("\"Active\""));
    }

    #[test]
    fn exported_names_includes_schema_and_type() {
        let out = convert(CodegenInput::source(SAMPLE), &Default::default()).unwrap();
        assert!(out.exported_names.contains(&"PackageSchema".to_string()));
        assert!(out.exported_names.contains(&"Package".to_string()));
    }
}
