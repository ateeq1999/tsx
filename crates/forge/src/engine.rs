//! The forge Engine — a Tera wrapper with the 4-tier system, import hoisting,
//! and framework package loading built in.

use std::path::Path;
use tera::Tera;
use walkdir::WalkDir;

use crate::{collector, context::ForgeContext, error::ForgeError, filters, slots, tier::Tier};

/// The forge rendering engine.
///
/// Wrap Tera with:
/// - Custom filters: `snake_case`, `pascal_case`, `camel_case`, `kebab_case`
/// - Import hoisting filters: `collect_import`, `collect_import_priority`
/// - Import drain function: `render_imports()`
/// - Tier-aware template registry
pub struct Engine {
    tera: Tera,
}

impl Engine {
    /// Create an engine with all forge extensions registered. No templates loaded yet.
    pub fn new() -> Self {
        let mut tera = Tera::default();
        register_extensions(&mut tera);
        Engine { tera }
    }

    /// Load all `.jinja` and `.forge` template files from `dir` recursively.
    /// Template names are relative paths from `dir` with forward slashes.
    pub fn load_dir(&mut self, dir: &Path) -> Result<(), ForgeError> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "jinja" || ext == "forge" {
                    let name = path
                        .strip_prefix(dir)
                        .map_err(|e| ForgeError::LoadError(e.to_string()))?
                        .to_string_lossy()
                        .replace('\\', "/");
                    let content = std::fs::read_to_string(path)
                        .map_err(|e| ForgeError::LoadError(e.to_string()))?;
                    self.tera
                        .add_raw_template(&name, &content)
                        .map_err(|e| ForgeError::LoadError(e.to_string()))?;
                }
            }
        }
        Ok(())
    }

    /// Load templates from embedded `(name, content)` pairs (for binary embedding).
    pub fn load_embedded(&mut self, templates: &[(&str, &str)]) -> Result<(), ForgeError> {
        for (name, content) in templates {
            self.tera
                .add_raw_template(name, content)
                .map_err(|e| ForgeError::LoadError(e.to_string()))?;
        }
        Ok(())
    }

    /// Add a single raw template by name and content string.
    pub fn add_raw(&mut self, name: &str, content: &str) -> Result<(), ForgeError> {
        self.tera
            .add_raw_template(name, content)
            .map_err(|e| ForgeError::LoadError(e.to_string()))?;
        Ok(())
    }

    /// Render a template by name with the given context.
    /// Resets the ImportCollector and populates slots before rendering.
    pub fn render(&self, name: &str, ctx: &ForgeContext) -> Result<String, ForgeError> {
        collector::reset();
        slots::reset();
        // Populate thread-local slots from the context
        if let Some(slot_map) = ctx.slots() {
            for (k, v) in &slot_map {
                if let Some(content) = v.as_str() {
                    slots::fill(k, content);
                }
            }
        }
        self.tera
            .render(name, ctx.as_tera())
            .map_err(|e| ForgeError::RenderError(format!("{name}: {e}")))
    }

    /// Render without resetting the ImportCollector or slots.
    /// Use when rendering multiple templates in sequence and collecting all their imports.
    pub fn render_continue(&self, name: &str, ctx: &ForgeContext) -> Result<String, ForgeError> {
        self.tera
            .render(name, ctx.as_tera())
            .map_err(|e| ForgeError::RenderError(format!("{name}: {e}")))
    }

    /// Return the tier of a template based on its path.
    pub fn tier_of(&self, name: &str) -> Tier {
        Tier::from_path(name)
    }

    /// Check whether a template with this name is loaded.
    pub fn has_template(&self, name: &str) -> bool {
        self.tera.get_template_names().any(|n| n == name)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

fn register_extensions(tera: &mut Tera) {
    tera.register_filter("snake_case", filters::snake_case);
    tera.register_filter("pascal_case", filters::pascal_case);
    tera.register_filter("camel_case", filters::camel_case);
    tera.register_filter("kebab_case", filters::kebab_case);
    tera.register_filter("collect_import", filters::collect_import);
    tera.register_filter("collect_import_priority", filters::collect_import_priority);
    tera.register_function("render_imports", filters::render_imports_fn);
    tera.register_function("slot", slots::make_slot_fn());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_renders_simple_template() {
        let mut engine = Engine::new();
        engine.add_raw("test.jinja", "Hello {{ name | pascal_case }}!").unwrap();
        let ctx = ForgeContext::new().insert("name", "world");
        let out = engine.render("test.jinja", &ctx).unwrap();
        assert_eq!(out, "Hello World!");
    }

    #[test]
    fn engine_collect_and_drain_imports() {
        let mut engine = Engine::new();
        engine
            .add_raw(
                "test.jinja",
                "{{ 'import React from \"react\"' | collect_import_priority }}{{ 'import { z } from \"zod\"' | collect_import }}{{ render_imports() }}",
            )
            .unwrap();
        let ctx = ForgeContext::new();
        let out = engine.render("test.jinja", &ctx).unwrap();
        let lines: Vec<&str> = out.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines[0], "import React from \"react\"");
        assert!(lines.iter().any(|l| l.contains("zod")));
    }

    #[test]
    fn tier_infers_from_name() {
        let engine = Engine::new();
        assert_eq!(engine.tier_of("atoms/drizzle/column.jinja"), Tier::Atom);
        assert_eq!(engine.tier_of("features/schema.jinja"), Tier::Feature);
    }
}
