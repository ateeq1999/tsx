//! Forge `@`-directive preprocessor.
//!
//! Transforms a `.forge` template with `@`-directives into plain Tera/Jinja2 syntax
//! **before** the template is handed to the Tera engine.  Pure Jinja2 files pass through
//! unchanged (the preprocessor is a no-op when no `@` directives are present).
//!
//! # Translation table
//!
//! | Forge directive | Tera output |
//! |---|---|
//! | `@import("pkg")` | `{{ "import pkg from 'pkg'" \| collect_import_priority }}` |
//! | `@import("pkg", named=["a","b"])` | `{{ "import { a, b } from 'pkg'" \| collect_import }}` |
//! | `@if(cond)` | `{% if cond %}` |
//! | `@unless(cond)` | `{% if not cond %}` |
//! | `@each(items as item)` | `{% for item in items %}` |
//! | `@end` | closes the nearest open block (`{% endif %}` / `{% endfor %}`) |
//! | `@set(x = expr)` | `{% set x = expr %}` |
//! | `@slot("x")` | `{{ slot(name='x') }}` |
//! | `@slot("x", default="y")` | `{{ slot(name='x', default='y') }}` |
//! | `@inject("x")` | `{{ inject(key='x') }}` |
//! | `@include("path")` | `{% include "path" %}` |
//! | `@variant("name")` | `{% if ctx.variant == 'name' %}` |
//! | `@ctx.name.pascal()` | `{{ ctx.name \| pascal_case }}` |
//! | `@ctx.name.kebab().upper()` | `{{ ctx.name \| kebab_case \| upper }}` |
//! | `@extends("base.forge")` | `{% extends "base.forge" %}` |
//! | `@slot("x") … @end` (in extends child) | `{% block x %} … {% endblock x %}` |
//! | `@schema({...})` | stripped (metadata only, used by `validate.rs`) |
//! | `@feature("name") … @end` | `{% if features and 'name' in features %} … {% endif %}` |
//! | `@macro("name") … @end` | `{% macro name() %} … {% endmacro name %}` |
//! | `@call("name")` | `{{ self::name() }}` |
//! | `@call("name", {key: "val"})` | `{{ self::name(key="val") }}` |
//! | `@hook("before-render") … @end` | stripped (engine-level hook, not a render-time construct) |

/// Preprocess a `.forge` template source, translating `@`-directives to Tera syntax.
///
/// Returns the transformed source string.  If the input contains no `@` directives
/// the output is identical to the input (zero-copy is not guaranteed but the content
/// is identical).
pub fn preprocess(src: &str) -> String {
    // Quick scan: determine if this template uses @extends (child template).
    // In extends-mode, @slot("name") ... @end becomes {% block name %} ... {% endblock name %}.
    let extends_mode = src
        .lines()
        .any(|l| l.trim_start().starts_with("@extends("));

    let mut out = String::with_capacity(src.len() + 64);
    // Stack tracking open block types so @end knows which closing tag to emit.
    let mut block_stack: Vec<BlockKind> = Vec::new();

    for line in src.lines() {
        let trimmed = line.trim_start();

        // Determine if we are currently inside an @hook block (stripped).
        let in_hook = block_stack.last() == Some(&BlockKind::Hook);

        if let Some(rest) = trimmed.strip_prefix('@') {
            let indent = &line[..line.len() - trimmed.len()];
            if let Some(transformed) =
                transform_directive(rest, &mut block_stack, extends_mode)
            {
                // Directives that produce "__STRIP__" are silently removed.
                // Also skip all content while inside a hook block.
                if transformed == "__STRIP__" || in_hook {
                    continue;
                }
                out.push_str(indent);
                out.push_str(&transformed);
                out.push('\n');
                continue;
            }
            // Not a recognised directive — check if it's a method-chain variable reference
            if !in_hook {
                if let Some(var_expr) = try_method_chain(rest) {
                    out.push_str(indent);
                    out.push_str(&var_expr);
                    out.push('\n');
                    continue;
                }
            }
        }

        // Pass through unchanged (skip if inside a hook block)
        if in_hook {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }

    // Trim trailing newline added above if the source didn't end with one.
    if !src.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }

    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockKind {
    If,
    For,
    /// An `@slot("name") ... @end` block inside an `@extends` child template,
    /// which maps to Tera's `{% block name %} ... {% endblock name %}`.
    ExtendSlot(String),
    /// An `@feature("name") ... @end` conditional block.
    Feature,
    /// An `@macro("name") ... @end` block mapping to Tera's `{% macro name() %}`.
    Macro(String),
    /// An `@hook("...") ... @end` block that is stripped from output.
    Hook,
}

/// Try to transform a single `@`-directive (the `@` has already been stripped).
/// Returns `Some(tera_string)` on success, `None` if not a known directive.
/// Returns `Some("__STRIP__")` for directives that should be silently removed (e.g. `@schema`).
fn transform_directive(
    rest: &str,
    stack: &mut Vec<BlockKind>,
    extends_mode: bool,
) -> Option<String> {
    // Strip optional trailing whitespace / inline comments
    let rest = rest.trim_end();

    // ── @end ──────────────────────────────────────────────────────────────────
    if rest == "end" {
        let tag = match stack.pop() {
            Some(BlockKind::If)                  => "{% endif %}".to_string(),
            Some(BlockKind::For)                 => "{% endfor %}".to_string(),
            Some(BlockKind::Feature)             => "{% endif %}".to_string(),
            Some(BlockKind::ExtendSlot(name))    => format!("{{% endblock {name} %}}"),
            Some(BlockKind::Macro(name))         => format!("{{% endmacro {name} %}}"),
            Some(BlockKind::Hook)                => "__STRIP__".to_string(),
            None                                 => "{% endif %}".to_string(), // fallback
        };
        return Some(tag);
    }

    // ── @extends("path") ──────────────────────────────────────────────────────
    if let Some(args) = strip_call(rest, "extends") {
        let path = args.trim().trim_matches(|c| c == '"' || c == '\'');
        return Some(format!("{{% extends \"{path}\" %}}"));
    }

    // ── @schema({...}) ─ metadata only, stripped from output ─────────────────
    if rest.starts_with("schema(") {
        return Some("__STRIP__".to_string());
    }

    // ── @import(...) ──────────────────────────────────────────────────────────
    if let Some(args) = strip_call(rest, "import") {
        return Some(transform_import(&args));
    }

    // ── @if(cond) ─────────────────────────────────────────────────────────────
    if let Some(cond) = strip_call(rest, "if") {
        stack.push(BlockKind::If);
        return Some(format!("{{% if {cond} %}}"));
    }

    // ── @unless(cond) ─────────────────────────────────────────────────────────
    if let Some(cond) = strip_call(rest, "unless") {
        stack.push(BlockKind::If);
        return Some(format!("{{% if not {cond} %}}"));
    }

    // ── @each(items as item) ──────────────────────────────────────────────────
    if let Some(expr) = strip_call(rest, "each") {
        stack.push(BlockKind::For);
        if let Some((collection, var)) = split_as(&expr) {
            return Some(format!("{{% for {} in {} %}}", var.trim(), collection.trim()));
        }
        return Some(format!("{{% for item in {} %}}", expr.trim()));
    }

    // ── @set(x = expr) ────────────────────────────────────────────────────────
    if let Some(assignment) = strip_call(rest, "set") {
        return Some(format!("{{% set {assignment} %}}"));
    }

    // ── @slot("name") / @slot("name", default="fallback") ────────────────────
    // In extends-mode this opens a Tera block override; otherwise it's a slot call.
    if let Some(args) = strip_call(rest, "slot") {
        if extends_mode {
            let (name, _rest) = parse_quoted_string(args.trim());
            stack.push(BlockKind::ExtendSlot(name.clone()));
            return Some(format!("{{% block {name} %}}"));
        }
        return Some(transform_slot(&args));
    }

    // ── @inject("key") ────────────────────────────────────────────────────────
    if let Some(args) = strip_call(rest, "inject") {
        let key = args.trim().trim_matches(|c| c == '"' || c == '\'');
        return Some(format!("{{{{ inject(key='{key}') }}}}"));
    }

    // ── @include("path") ──────────────────────────────────────────────────────
    if let Some(args) = strip_call(rest, "include") {
        let path = args.trim().trim_matches(|c| c == '"' || c == '\'');
        return Some(format!("{{% include \"{path}\" %}}"));
    }

    // ── @variant("name") ──────────────────────────────────────────────────────
    if let Some(args) = strip_call(rest, "variant") {
        stack.push(BlockKind::If);
        let name = args.trim().trim_matches(|c| c == '"' || c == '\'');
        return Some(format!("{{% if ctx.variant == '{name}' %}}"));
    }

    // ── @feature("name") ──────────────────────────────────────────────────────
    if let Some(args) = strip_call(rest, "feature") {
        stack.push(BlockKind::Feature);
        let name = args.trim().trim_matches(|c| c == '"' || c == '\'');
        return Some(format!(
            "{{% if features is defined and '{name}' in features %}}"
        ));
    }

    // ── @macro("name") ────────────────────────────────────────────────────────
    // Translates to Tera's native `{% macro name() %} ... {% endmacro name %}`.
    if let Some(args) = strip_call(rest, "macro") {
        let (name, _rest) = parse_quoted_string(args.trim());
        stack.push(BlockKind::Macro(name.clone()));
        return Some(format!("{{% macro {name}() %}}"));
    }

    // ── @call("name") / @call("name", {key: "val"}) ───────────────────────────
    if let Some(args) = strip_call(rest, "call") {
        return Some(transform_call(&args));
    }

    // ── @hook("event") ─────────────────────────────────────────────────────────
    // Hooks are engine-level constructs, not Tera constructs.
    // The block opener and its body are stripped from the rendered output.
    if rest.starts_with("hook(") {
        stack.push(BlockKind::Hook);
        return Some("__STRIP__".to_string());
    }

    None
}

/// Handle method-chain variable expressions like `ctx.name.pascal()`.
/// Returns `Some("{{ expr | filter | filter }}")` if it looks like a chain, else `None`.
fn try_method_chain(rest: &str) -> Option<String> {
    // Must start with a word character and contain at least one `.method()`
    if !rest.chars().next().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) {
        return None;
    }
    if !rest.contains('.') {
        return None;
    }
    // Split on '.' and interpret trailing `method()` calls as Tera filters
    let parts: Vec<&str> = rest.splitn(2, '.').collect();
    if parts.len() < 2 {
        return None;
    }

    let var_name = parts[0];
    let chain = parts[1];

    // Build the variable path (may contain more dots without parentheses)
    let mut var_path = var_name.to_string();
    let mut filters: Vec<String> = Vec::new();
    let mut remaining = chain;

    loop {
        // Try to parse `method_name()` or `method_name("arg")`
        if let Some(dot_pos) = remaining.find('.') {
            let segment = &remaining[..dot_pos];
            remaining = &remaining[dot_pos + 1..];
            if let Some(filter) = parse_method_call(segment) {
                filters.push(filter);
            } else {
                // It's a plain field access, extend var_path
                var_path.push('.');
                var_path.push_str(segment);
            }
        } else {
            // Last segment
            if let Some(filter) = parse_method_call(remaining) {
                filters.push(filter);
            } else {
                // Not a method call — not a method chain expression
                if filters.is_empty() {
                    return None;
                }
                var_path.push('.');
                var_path.push_str(remaining);
            }
            break;
        }
    }

    if filters.is_empty() {
        return None;
    }

    let filter_chain = filters.join(" | ");
    Some(format!("{{{{ {} | {} }}}}", var_path, filter_chain))
}

/// Parse a method call like `pascal()`, `kebab()`, `upper()`, `replace("-","_")`.
/// Returns the Tera filter string, or `None` if not a recognised method.
fn parse_method_call(segment: &str) -> Option<String> {
    let (name, args_raw) = if let Some(paren_pos) = segment.find('(') {
        let name = &segment[..paren_pos];
        let args = segment[paren_pos + 1..].trim_end_matches(')');
        (name, args)
    } else {
        return None; // not a method call (no parentheses)
    };

    let filter = match name {
        "pascal" => "pascal_case".to_string(),
        "kebab" => "kebab_case".to_string(),
        "snake" => "snake_case".to_string(),
        "camel" => "camel_case".to_string(),
        "upper" => "upper".to_string(),
        "lower" => "lower".to_string(),
        "trim" => "trim".to_string(),
        "replace" => {
            // replace("from","to") → replace(from="from", to="to")
            let args = args_raw.trim();
            if args.is_empty() {
                "replace".to_string()
            } else {
                let parts: Vec<&str> = args.splitn(2, ',').collect();
                if parts.len() == 2 {
                    let from = parts[0].trim().trim_matches(|c| c == '"' || c == '\'');
                    let to = parts[1].trim().trim_matches(|c| c == '"' || c == '\'');
                    format!("replace(from=\"{}\", to=\"{}\")", from, to)
                } else {
                    "replace".to_string()
                }
            }
        }
        _ => return None,
    };

    Some(filter)
}

// ---------------------------------------------------------------------------
// Helper parsers
// ---------------------------------------------------------------------------

/// If `src` starts with `name(` and ends with `)`, return the inner argument string.
fn strip_call<'a>(src: &'a str, name: &str) -> Option<String> {
    let prefix = format!("{}(", name);
    if let Some(after) = src.strip_prefix(prefix.as_str()) {
        // Find the matching closing paren (handle nested parens)
        let mut depth = 1usize;
        let mut end = 0usize;
        for (i, ch) in after.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }
        return Some(after[..end].to_string());
    }
    None
}

/// Split `"collection as var"` into `(collection, var)`.
fn split_as(expr: &str) -> Option<(&str, &str)> {
    let mut iter = expr.splitn(2, " as ");
    let collection = iter.next()?;
    let var = iter.next()?;
    Some((collection, var))
}

/// Transform `@call("name")` or `@call("name", {key: "val"})` into a Tera macro call.
///
/// `@call("greet")` → `{{ self::greet() }}`
/// `@call("greet", {name: "World"})` → `{{ self::greet(name="World") }}`
fn transform_call(args: &str) -> String {
    let args = args.trim();
    let (macro_name, rest) = parse_quoted_string(args);
    let rest = rest.trim();

    // If there's a second argument (the args object), try to parse key=value pairs
    let call_args = if let Some(obj_str) = rest.strip_prefix(',') {
        let obj_str = obj_str.trim().trim_start_matches('{').trim_end_matches('}');
        let mut kv_pairs: Vec<String> = Vec::new();
        for part in obj_str.split(',') {
            let part = part.trim();
            if let Some(colon) = part.find(':') {
                let key = part[..colon].trim().trim_matches(|c| c == '"' || c == '\'');
                let val = part[colon + 1..].trim().trim_matches(|c| c == '"' || c == '\'');
                kv_pairs.push(format!("{}=\"{}\"", key, val));
            }
        }
        kv_pairs.join(", ")
    } else {
        String::new()
    };

    format!("{{{{ self::{}({}) }}}}", macro_name, call_args)
}

/// Transform `@import` arguments into a Tera import-collector call.
///
/// `"pkg"` → `{{ "import pkg from 'pkg'" | collect_import_priority }}`
/// `"pkg", named=["a","b"]` → `{{ "import { a, b } from 'pkg'" | collect_import }}`
fn transform_import(args: &str) -> String {
    // Parse the package name (first quoted string)
    let args = args.trim();
    let (pkg, rest) = parse_quoted_string(args);

    let rest = rest.trim_start_matches(',').trim();

    // Check for `named=[...]`
    if let Some(named_raw) = rest.strip_prefix("named=") {
        let named_raw = named_raw.trim();
        if named_raw.starts_with('[') {
            let inner = named_raw
                .trim_start_matches('[')
                .trim_end_matches(']');
            let names: Vec<&str> = inner
                .split(',')
                .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\''))
                .filter(|s| !s.is_empty())
                .collect();
            let named_str = names.join(", ");
            let import_str = format!("import {{ {} }} from '{}'", named_str, pkg);
            return format!("{{{{ \"{}\" | collect_import }}}}", import_str);
        }
    }

    // Default: side-effect / default import (priority so it lands at top)
    let import_str = format!("import {} from '{}'", pkg, pkg);
    format!("{{{{ \"{}\" | collect_import_priority }}}}", import_str)
}

/// Transform `@slot` arguments.
fn transform_slot(args: &str) -> String {
    let args = args.trim();
    let (name, rest) = parse_quoted_string(args);
    let rest = rest.trim_start_matches(',').trim();

    if let Some(default_raw) = rest.strip_prefix("default=") {
        let default_val = default_raw.trim().trim_matches(|c| c == '"' || c == '\'');
        return format!("{{{{ slot(name='{}', default='{}') }}}}", name, default_val);
    }

    format!("{{{{ slot(name='{}') }}}}", name)
}

/// Parse the leading quoted string from `src`, return `(value, remainder)`.
fn parse_quoted_string(src: &str) -> (String, &str) {
    let src = src.trim();
    let quote_char = if src.starts_with('"') { '"' } else { '\'' };
    if src.starts_with(quote_char) {
        let inner = &src[1..];
        if let Some(end) = inner.find(quote_char) {
            return (inner[..end].to_string(), &inner[end + 1..]);
        }
    }
    // Fallback: treat everything up to ',' or end as the value
    if let Some(comma) = src.find(',') {
        (src[..comma].trim().to_string(), &src[comma..])
    } else {
        (src.to_string(), "")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_plain_jinja() {
        let src = "{% if x %}hello{% endif %}";
        assert_eq!(preprocess(src), src);
    }

    #[test]
    fn if_end() {
        let src = "@if(ctx.auth)\nhello\n@end";
        let out = preprocess(src);
        assert!(out.contains("{% if ctx.auth %}"), "got: {}", out);
        assert!(out.contains("{% endif %}"), "got: {}", out);
    }

    #[test]
    fn unless() {
        let out = preprocess("@unless(ctx.slim)");
        assert!(out.contains("{% if not ctx.slim %}"), "got: {}", out);
    }

    #[test]
    fn each_as() {
        let out = preprocess("@each(ctx.fields as field)");
        assert!(out.contains("{% for field in ctx.fields %}"), "got: {}", out);
    }

    #[test]
    fn import_default() {
        let out = preprocess("@import(\"react\")");
        assert!(out.contains("collect_import_priority"), "got: {}", out);
        assert!(out.contains("react"), "got: {}", out);
    }

    #[test]
    fn import_named() {
        let out = preprocess("@import(\"zod\", named=[\"z\"])");
        assert!(out.contains("import { z } from 'zod'"), "got: {}", out);
        assert!(out.contains("collect_import"), "got: {}", out);
    }

    #[test]
    fn slot_simple() {
        let out = preprocess("@slot(\"extra-columns\")");
        assert!(out.contains("slot(name='extra-columns')"), "got: {}", out);
    }

    #[test]
    fn slot_with_default() {
        let out = preprocess("@slot(\"header\", default=\"// none\")");
        assert!(out.contains("default='// none'"), "got: {}", out);
    }

    #[test]
    fn inject() {
        let out = preprocess("@inject(\"db-import\")");
        assert!(out.contains("inject(key='db-import')"), "got: {}", out);
    }

    #[test]
    fn include_directive() {
        let out = preprocess("@include(\"atoms/col.forge\")");
        assert!(out.contains("{% include \"atoms/col.forge\" %}"), "got: {}", out);
    }

    #[test]
    fn variant() {
        let out = preprocess("@variant(\"with-relations\")\nhello\n@end");
        assert!(out.contains("ctx.variant == 'with-relations'"), "got: {}", out);
        assert!(out.contains("{% endif %}"), "got: {}", out);
    }

    #[test]
    fn method_chain_pascal() {
        let out = preprocess("@ctx.name.pascal()");
        assert!(out.contains("{{ ctx.name | pascal_case }}"), "got: {}", out);
    }

    #[test]
    fn method_chain_kebab_upper() {
        let out = preprocess("@ctx.name.kebab().upper()");
        assert!(out.contains("kebab_case | upper"), "got: {}", out);
    }

    #[test]
    fn set_directive() {
        let out = preprocess("@set(x = 42)");
        assert!(out.contains("{% set x = 42 %}"), "got: {}", out);
    }

    #[test]
    fn nested_if_each() {
        let src = "@if(ctx.auth)\n@each(ctx.fields as f)\nhello\n@end\n@end";
        let out = preprocess(src);
        assert!(out.contains("{% if ctx.auth %}"), "got: {}", out);
        assert!(out.contains("{% for f in ctx.fields %}"), "got: {}", out);
        assert!(out.contains("{% endfor %}"), "got: {}", out);
        assert!(out.contains("{% endif %}"), "got: {}", out);
    }

    #[test]
    fn extends_translates() {
        let src = "@extends(\"layouts/base.forge\")\n@slot(\"content\")\nhello\n@end";
        let out = preprocess(src);
        assert!(out.contains("{% extends \"layouts/base.forge\" %}"), "got: {}", out);
        assert!(out.contains("{% block content %}"), "got: {}", out);
        assert!(out.contains("{% endblock content %}"), "got: {}", out);
    }

    #[test]
    fn slot_without_extends_stays_as_slot_call() {
        let out = preprocess("@slot(\"sidebar\")");
        assert!(out.contains("slot(name='sidebar')"), "got: {}", out);
        assert!(!out.contains("block"), "got: {}", out);
    }

    #[test]
    fn schema_directive_is_stripped() {
        let src = "@schema({ \"name\": { \"type\": \"string\" } })\nhello";
        let out = preprocess(src);
        assert!(!out.contains("@schema"), "got: {}", out);
        assert!(!out.contains("__STRIP__"), "got: {}", out);
        assert!(out.contains("hello"), "got: {}", out);
    }

    #[test]
    fn feature_directive() {
        let src = "@feature(\"auth\")\nhello\n@end";
        let out = preprocess(src);
        assert!(out.contains("features is defined"), "got: {}", out);
        assert!(out.contains("'auth' in features"), "got: {}", out);
        assert!(out.contains("{% endif %}"), "got: {}", out);
    }

    #[test]
    fn macro_directive() {
        let src = "@macro(\"greet\")\nhello\n@end";
        let out = preprocess(src);
        assert!(out.contains("{% macro greet() %}"), "got: {}", out);
        assert!(out.contains("hello"), "got: {}", out);
        assert!(out.contains("{% endmacro greet %}"), "got: {}", out);
    }

    #[test]
    fn call_directive_no_args() {
        let out = preprocess("@call(\"greet\")");
        assert!(out.contains("{{ self::greet() }}"), "got: {}", out);
    }

    #[test]
    fn call_directive_with_args() {
        let out = preprocess("@call(\"greet\", {name: \"World\"})");
        assert!(out.contains("self::greet("), "got: {}", out);
        assert!(out.contains("name="), "got: {}", out);
    }

    #[test]
    fn hook_block_is_stripped() {
        let src = "@hook(\"before-render\")\nsome setup code\n@end\nhello";
        let out = preprocess(src);
        assert!(!out.contains("before-render"), "hook opener should be stripped: {}", out);
        assert!(!out.contains("some setup code"), "hook body should be stripped: {}", out);
        assert!(out.contains("hello"), "content after hook should remain: {}", out);
    }
}
