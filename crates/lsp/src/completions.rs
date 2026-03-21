//! Completion providers for .tsx/ config files and .forge templates.

use lsp_types::{CompletionItem, CompletionItemKind};

/// Top-level keys for `.tsx/stack.json` / `user-stack.json`.
const STACK_TOP_KEYS: &[(&str, &str)] = &[
    ("framework",  "Active framework slug (e.g. tanstack-start)"),
    ("packages",   "Array of installed package IDs"),
    ("paths",      "Output path overrides per generator type"),
    ("patterns",   "Code style patterns (component_style, file_naming, import_alias)"),
    ("style",      "Rendering style preferences (indent, quotes, css, forms, …)"),
    ("templates",  "Per-generator template overrides (path to .forge file)"),
    ("slots",      "Per-slot injection target overrides"),
];

/// Keys within the `style` object.
const STYLE_KEYS: &[(&str, &str)] = &[
    ("indent",     "Spaces per indent level (default: 2)"),
    ("quotes",     "Quote style: \"double\" or \"single\""),
    ("semicolons", "Whether to emit semicolons (true/false)"),
    ("css",        "CSS solution: \"tailwind\", \"cssmodules\", \"styled\""),
    ("components", "UI component library: \"shadcn\", \"radix\", \"none\""),
    ("forms",      "Form library: \"tanstack-form\", \"react-hook-form\", \"none\""),
    ("icons",      "Icon library: \"lucide-react\", \"heroicons\", \"none\""),
    ("toast",      "Toast library: \"sonner\", \"react-hot-toast\", \"none\""),
];

/// Keys within the `patterns` object.
const PATTERN_KEYS: &[(&str, &str)] = &[
    ("component_style", "\"named-export\" or \"default-export\""),
    ("file_naming",     "\"kebab-case\", \"camelCase\", or \"PascalCase\""),
    ("import_alias",    "TypeScript path alias prefix (e.g. \"@/\")"),
];

/// Template variables available in .forge / .jinja templates.
const TEMPLATE_VARS: &[(&str, &str)] = &[
    ("ctx.name",          "The primary name passed to the generator"),
    ("ctx.fields",        "Array of field definitions [{name, type, …}]"),
    ("ctx.feature",       "Feature/module name (for multi-file generators)"),
    ("ctx.auth",          "Boolean — true if better-auth is in the stack"),
    ("ctx.timestamps",    "Boolean — include createdAt/updatedAt columns"),
    ("ctx.variant",       "Active variant flag for @variant blocks"),
    ("style.indent",      "Indent size from effective style"),
    ("style.quotes",      "Quote character from effective style"),
    ("style.css",         "CSS solution from effective style"),
    ("style.forms",       "Form library from effective style"),
    ("style.components",  "UI component library from effective style"),
];

/// Forge `@`-directives for template files.
const FORGE_DIRECTIVES: &[(&str, &str)] = &[
    ("@import",   "@import(\"module\", named=[\"x\"]) — collect an import"),
    ("@if",       "@if(condition) … @end"),
    ("@unless",   "@unless(condition) … @end"),
    ("@each",     "@each(items as item) … @end"),
    ("@slot",     "@slot(\"name\") — inject a named slot"),
    ("@include",  "@include(\"path/to.forge\") — include another template"),
    ("@set",      "@set(x = expr) — assign a variable"),
    ("@variant",  "@variant(\"flag\") … @end — conditional variant block"),
    ("@inject",   "@inject(\"key\") — inject a context value"),
    ("@compose",  "@compose { … } — multi-template composition"),
];

fn make_item(label: &str, detail: &str, kind: CompletionItemKind) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(kind),
        detail: Some(detail.to_string()),
        ..Default::default()
    }
}

/// Return completions for a `.tsx/stack.json` or `user-stack.json` file.
/// `prefix` is the JSON key path up to the cursor (e.g. `"style."` or `""`).
pub fn stack_json_completions(prefix: &str) -> Vec<CompletionItem> {
    if prefix.starts_with("style.") || prefix == "style" {
        STYLE_KEYS.iter()
            .map(|(k, d)| make_item(k, d, CompletionItemKind::FIELD))
            .collect()
    } else if prefix.starts_with("patterns.") || prefix == "patterns" {
        PATTERN_KEYS.iter()
            .map(|(k, d)| make_item(k, d, CompletionItemKind::FIELD))
            .collect()
    } else {
        STACK_TOP_KEYS.iter()
            .map(|(k, d)| make_item(k, d, CompletionItemKind::PROPERTY))
            .collect()
    }
}

/// Return completions for a `.forge` or `.jinja` template file.
pub fn template_completions() -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = TEMPLATE_VARS.iter()
        .map(|(k, d)| make_item(k, d, CompletionItemKind::VARIABLE))
        .collect();
    items.extend(FORGE_DIRECTIVES.iter()
        .map(|(k, d)| make_item(k, d, CompletionItemKind::KEYWORD)));
    items
}
