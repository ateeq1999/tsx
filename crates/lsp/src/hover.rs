//! Hover documentation for `.tsx/` config fields.

/// Look up hover documentation for a JSON key (possibly dot-separated path).
pub fn hover_for_key(key: &str) -> Option<&'static str> {
    match key {
        "framework" => Some(
            "**framework** — Active framework slug.\n\nExamples: `tanstack-start`, `nextjs`, `remix`.\nUsed to resolve which registry packages and templates are active."
        ),
        "packages" => Some(
            "**packages** — Installed tsx package IDs.\n\nArray of strings, e.g. `[\"drizzle-pg\", \"better-auth\", \"shadcn\"]`.\nPackages add generators, templates, and atoms to the active project."
        ),
        "paths" => Some(
            "**paths** — Output path overrides per generator type.\n\nKeys match generator output slots: `schema`, `route`, `server-fn`, `component`, `query-hook`.\nValues are Jinja2 path templates, e.g. `\"src/db/schema/{{name}}.ts\"`."
        ),
        "patterns" => Some(
            "**patterns** — Code style patterns.\n\n- `component_style`: `named-export` | `default-export`\n- `file_naming`: `kebab-case` | `camelCase` | `PascalCase`\n- `import_alias`: TypeScript path alias (e.g. `@/`)"
        ),
        "style" => Some(
            "**style** — Rendering style preferences.\n\nInjected into every template as `style.*`. Fields include:\n`indent`, `quotes`, `semicolons`, `css`, `components`, `forms`, `icons`, `toast`."
        ),
        "style.indent" => Some(
            "**style.indent** — Spaces per indent level.\n\nDefault: `2`. Injected into templates as `{{ style.indent }}`."
        ),
        "style.quotes" => Some(
            "**style.quotes** — Quote character for string literals.\n\nValues: `\"double\"` | `\"single\"`. Default: `\"double\"`."
        ),
        "style.semicolons" => Some(
            "**style.semicolons** — Whether to emit trailing semicolons.\n\nValues: `true` | `false`. Default: `false`."
        ),
        "style.css" => Some(
            "**style.css** — CSS solution.\n\nValues: `\"tailwind\"` | `\"cssmodules\"` | `\"styled\"`. Templates branch on this to choose className utilities."
        ),
        "style.components" => Some(
            "**style.components** — UI component library.\n\nValues: `\"shadcn\"` | `\"radix\"` | `\"none\"`. Used to auto-import the right primitives."
        ),
        "style.forms" => Some(
            "**style.forms** — Form library.\n\nValues: `\"tanstack-form\"` | `\"react-hook-form\"` | `\"none\"`.\n\nTemplates check `{% if style.forms == \"tanstack-form\" %}` to pick the right import."
        ),
        "style.icons" => Some(
            "**style.icons** — Icon library.\n\nValues: `\"lucide-react\"` | `\"heroicons\"` | `\"none\"`."
        ),
        "style.toast" => Some(
            "**style.toast** — Toast / notification library.\n\nValues: `\"sonner\"` | `\"react-hot-toast\"` | `\"none\"`."
        ),
        "templates" => Some(
            "**templates** — Per-generator template overrides.\n\nKeys are generator IDs (e.g. `\"schema\"`), values are paths to `.forge` files.\nExample: `{ \"schema\": \"./my-templates/schema.forge\" }`"
        ),
        "slots" => Some(
            "**slots** — Per-slot injection target overrides.\n\nKeys are slot names, values are file paths.\nExample: `{ \"schema-imports\": \"app/db/schema.ts\" }`"
        ),
        _ => None,
    }
}
