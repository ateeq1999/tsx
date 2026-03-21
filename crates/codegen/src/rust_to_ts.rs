//! Rust structs → TypeScript interfaces (no Zod runtime dependency).

use crate::types::{CodegenError, CodegenInput, CodegenOutput, rust_type_to_ts};
use crate::rust_to_zod::{RustItem};

/// Convert Rust source to TypeScript interface declarations.
pub fn convert(input: CodegenInput, source_path: Option<&str>) -> Result<CodegenOutput, CodegenError> {
    let source = input.read()?;
    let items = crate::rust_to_zod::parse_items_pub(&source);

    let mut lines: Vec<String> = Vec::new();
    let comment = source_path.unwrap_or("Rust source");
    lines.push(format!("// AUTO-GENERATED from {}", comment));
    lines.push(String::new());

    let mut exported_names = Vec::new();
    let mut warnings = Vec::new();

    for item in &items {
        match item {
            RustItem::Struct(s) => {
                lines.push(format!("export interface {} {{", s.name));
                for field in &s.fields {
                    let ts = rust_type_to_ts(&field.ty);
                    let optional = if field.serde_skip_none { "?" } else { "" };
                    lines.push(format!("  {}{}: {}", field.name, optional, ts));
                }
                lines.push(String::from("}"));
                lines.push(String::new());
                exported_names.push(s.name.clone());
            }
            RustItem::Enum(e) => {
                if e.variants.iter().all(|v| v.fields.is_empty()) {
                    let union = e.variants.iter()
                        .map(|v| format!("\"{}\"", v.name))
                        .collect::<Vec<_>>()
                        .join(" | ");
                    lines.push(format!("export type {} = {}", e.name, union));
                    lines.push(String::new());
                    exported_names.push(e.name.clone());
                } else {
                    warnings.push(format!("Complex enum '{}' emitted as unknown", e.name));
                    lines.push(format!("// TODO: complex enum {}", e.name));
                    lines.push(format!("export type {} = unknown", e.name));
                    lines.push(String::new());
                }
            }
        }
    }

    Ok(CodegenOutput {
        content: lines.join("\n"),
        filename: "generated/types.ts".to_string(),
        exported_names,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_interface() {
        let src = "pub struct User { pub id: String, pub age: i32, }";
        let out = convert(CodegenInput::source(src), None).unwrap();
        assert!(out.content.contains("export interface User"));
        assert!(out.content.contains("id: string"));
        assert!(out.content.contains("age: number"));
    }
}
