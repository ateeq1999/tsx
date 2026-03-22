//! Snapshot testing helpers and example tests for forge templates.
//!
//! These tests render templates against fixed inputs and compare against
//! expected output strings stored inline.  They serve as both regression
//! tests and living documentation of the template system.
//!
//! # Pattern
//!
//! ```rust,ignore
//! assert_forge_snapshot!(engine, template_name, ctx, "expected output");
//! ```

use tsx_forge::{Engine, ForgeContext, lint_template, validate_input};

// ---------------------------------------------------------------------------
// Snapshot assertion macro
// ---------------------------------------------------------------------------

/// Assert that rendering `$template` with `$ctx` in `$engine` produces `$expected`.
///
/// On mismatch the panic message shows a line-by-line diff so failures are
/// easy to diagnose.
#[macro_export]
macro_rules! assert_forge_snapshot {
    ($engine:expr, $template:expr, $ctx:expr, $expected:expr) => {{
        let actual = $engine
            .render($template, &$ctx)
            .unwrap_or_else(|e| panic!("render failed for '{}': {}", $template, e));
        if actual.trim() != $expected.trim() {
            let diff = simple_diff(actual.trim(), $expected.trim());
            panic!(
                "Snapshot mismatch for '{}':\n{}\n\nActual:\n{}\n\nExpected:\n{}",
                $template, diff, actual.trim(), $expected.trim()
            );
        }
    }};
}

/// Minimal line-diff for readable panic messages.
fn simple_diff(actual: &str, expected: &str) -> String {
    let mut out = String::new();
    let act_lines: Vec<&str> = actual.lines().collect();
    let exp_lines: Vec<&str> = expected.lines().collect();
    let max = act_lines.len().max(exp_lines.len());
    for i in 0..max {
        let a = act_lines.get(i).copied().unwrap_or("<missing>");
        let e = exp_lines.get(i).copied().unwrap_or("<missing>");
        if a != e {
            out.push_str(&format!("  line {}: got     {:?}\n", i + 1, a));
            out.push_str(&format!("  line {}: expected {:?}\n", i + 1, e));
        }
    }
    if out.is_empty() { "(no line differences — whitespace?)".to_string() } else { out }
}

// ---------------------------------------------------------------------------
// Helper: build a forge Engine with inline templates
// ---------------------------------------------------------------------------

fn engine_with(templates: &[(&str, &str)]) -> Engine {
    let mut engine = Engine::new();
    for (name, src) in templates {
        engine.add_raw(name, src).unwrap();
    }
    engine
}

// ---------------------------------------------------------------------------
// Snapshot tests: case-conversion filters
// ---------------------------------------------------------------------------

#[test]
fn snapshot_pascal_case_filter() {
    let engine = engine_with(&[("t.jinja", "{{ name | pascal_case }}")]);
    let ctx = ForgeContext::new().insert("name", "hello_world");
    assert_forge_snapshot!(engine, "t.jinja", ctx, "HelloWorld");
}

#[test]
fn snapshot_snake_case_filter() {
    let engine = engine_with(&[("t.jinja", "{{ name | snake_case }}")]);
    let ctx = ForgeContext::new().insert("name", "HelloWorld");
    assert_forge_snapshot!(engine, "t.jinja", ctx, "hello_world");
}

#[test]
fn snapshot_kebab_case_filter() {
    let engine = engine_with(&[("t.jinja", "{{ name | kebab_case }}")]);
    let ctx = ForgeContext::new().insert("name", "HelloWorld");
    assert_forge_snapshot!(engine, "t.jinja", ctx, "hello-world");
}

// ---------------------------------------------------------------------------
// Snapshot tests: slugify
// ---------------------------------------------------------------------------

#[test]
fn snapshot_slugify() {
    let engine = engine_with(&[("t.jinja", "{{ title | slugify }}")]);
    let ctx = ForgeContext::new().insert("title", "Hello World! 123");
    assert_forge_snapshot!(engine, "t.jinja", ctx, "hello-world-123");
}

// ---------------------------------------------------------------------------
// Snapshot tests: plural / singular
// ---------------------------------------------------------------------------

#[test]
fn snapshot_plural_regular() {
    let engine = engine_with(&[("t.jinja", "{{ word | plural }}")]);
    let ctx = ForgeContext::new().insert("word", "product");
    assert_forge_snapshot!(engine, "t.jinja", ctx, "products");
}

#[test]
fn snapshot_singular_regular() {
    let engine = engine_with(&[("t.jinja", "{{ word | singular }}")]);
    let ctx = ForgeContext::new().insert("word", "products");
    assert_forge_snapshot!(engine, "t.jinja", ctx, "product");
}

// ---------------------------------------------------------------------------
// Snapshot tests: truncate_str
// ---------------------------------------------------------------------------

#[test]
fn snapshot_truncate_short_string_unchanged() {
    let engine = engine_with(&[("t.jinja", "{{ s | truncate_str(length=20) }}")]);
    let ctx = ForgeContext::new().insert("s", "hello");
    assert_forge_snapshot!(engine, "t.jinja", ctx, "hello");
}

#[test]
fn snapshot_truncate_long_string() {
    let engine = engine_with(&[("t.jinja", "{{ s | truncate_str(length=5) }}")]);
    let ctx = ForgeContext::new().insert("s", "hello world");
    assert_forge_snapshot!(engine, "t.jinja", ctx, "hello...");
}

// ---------------------------------------------------------------------------
// Snapshot tests: indent
// ---------------------------------------------------------------------------

#[test]
fn snapshot_indent_filter() {
    let engine = engine_with(&[("t.jinja", "{{ code | indent(width=4) }}")]);
    let ctx = ForgeContext::new().insert("code", "line1\nline2");
    assert_forge_snapshot!(engine, "t.jinja", ctx, "    line1\n    line2");
}

// ---------------------------------------------------------------------------
// Snapshot tests: json_encode / json_decode
// ---------------------------------------------------------------------------

#[test]
fn snapshot_json_encode() {
    let engine = engine_with(&[("t.jinja", "{{ val | json_encode }}")]);
    let ctx = ForgeContext::new().insert("val", &serde_json::json!({"key": "value"}));
    let out = engine.render("t.jinja", &ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(parsed["key"], "value");
}

// ---------------------------------------------------------------------------
// Snapshot tests: forge directives via preprocessor
// ---------------------------------------------------------------------------

#[test]
fn snapshot_forge_if_each() {
    let src = "@if(show)\n@each(items as item)\n- {{ item }}\n@end\n@end";
    let engine = engine_with(&[("t.forge", src)]);
    let ctx = ForgeContext::new()
        .insert("show", &true)
        .insert("items", &["alpha", "beta"]);
    // Tera block tags leave blank lines; verify content rather than exact whitespace
    let out = engine.render("t.forge", &ctx).unwrap();
    assert!(out.contains("- alpha"), "missing alpha: {out:?}");
    assert!(out.contains("- beta"), "missing beta: {out:?}");
}

#[test]
fn snapshot_forge_import_hoisting() {
    let src = "@import(\"zod\", named=[\"z\"])\n{{ render_imports() }}";
    let engine = engine_with(&[("t.forge", src)]);
    let ctx = ForgeContext::new();
    assert_forge_snapshot!(engine, "t.forge", ctx, "import { z } from 'zod'");
}

#[test]
fn snapshot_schema_directive_stripped() {
    let src = "@schema({ \"name\": { \"type\": \"string\", \"required\": true } })\nhello {{ name }}";
    let engine = engine_with(&[("t.forge", src)]);
    let ctx = ForgeContext::new().insert("name", "world");
    assert_forge_snapshot!(engine, "t.forge", ctx, "hello world");
}

#[test]
fn snapshot_feature_enabled() {
    let src = "@feature(\"auth\")\nauthenticated\n@end";
    let engine = engine_with(&[("t.forge", src)]);
    let features = vec!["auth", "analytics"];
    let ctx = ForgeContext::new().insert("features", &features);
    assert_forge_snapshot!(engine, "t.forge", ctx, "authenticated");
}

#[test]
fn snapshot_feature_disabled() {
    let src = "@feature(\"auth\")\nauthenticated\n@end";
    let engine = engine_with(&[("t.forge", src)]);
    let features: Vec<&str> = vec![];
    let ctx = ForgeContext::new().insert("features", &features);
    assert_forge_snapshot!(engine, "t.forge", ctx, "");
}

// ---------------------------------------------------------------------------
// Snapshot tests: template composition via @extends
// ---------------------------------------------------------------------------

#[test]
fn snapshot_extends_block_override() {
    // Base template with a block that can be overridden by child templates.
    // Using a base-only block (no surrounding static content) to avoid Tera
    // whitespace quirks with content-outside-blocks rendering.
    let base = "{% block content %}default content{% endblock content %}\nFOOTER";
    let child = "@extends(\"base.forge\")\n@slot(\"content\")\noverridden\n@end";
    let engine = engine_with(&[("base.forge", base), ("child.forge", child)]);
    let ctx = ForgeContext::new();
    let out = engine.render("child.forge", &ctx).unwrap();
    assert!(out.contains("overridden"), "block override missing: {out:?}");
    assert!(!out.contains("default content"), "default should be replaced: {out:?}");
    assert!(out.contains("FOOTER"), "base content missing: {out:?}");
}

// ---------------------------------------------------------------------------
// Validation error snapshot tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_validation_required_missing() {
    let schema = serde_json::json!({
        "name": { "type": "string", "required": true }
    });
    let input = serde_json::json!({});
    let result = validate_input(&input, &schema);
    assert!(!result.is_ok());
    let msg = result.errors[0].clone();
    assert!(msg.contains("'name' is required"), "got: {msg}");
}

#[test]
fn snapshot_validation_type_mismatch() {
    let schema = serde_json::json!({
        "count": { "type": "number" }
    });
    let input = serde_json::json!({ "count": "not-a-number" });
    let result = validate_input(&input, &schema);
    assert!(!result.is_ok());
    assert!(result.errors[0].contains("number"), "{}", result.errors[0]);
}

// ---------------------------------------------------------------------------
// Lint snapshot tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_lint_unclosed_block() {
    let src = "@if(ctx.auth)\nhello";
    let result = lint_template(src);
    assert!(!result.is_ok());
    assert_eq!(result.errors[0].code, "F003");
    assert!(result.errors[0].message.contains("@if"));
}

#[test]
fn snapshot_lint_clean_template() {
    let src = "@if(ctx.auth)\nhello\n@end";
    let result = lint_template(src);
    assert!(result.is_ok());
    assert!(result.warnings.is_empty());
}

#[test]
fn snapshot_lint_suggests_import() {
    let src = "import React from 'react'";
    let result = lint_template(src);
    assert!(!result.suggestions.is_empty());
    assert!(result.suggestions[0].message.contains("@import"));
}
