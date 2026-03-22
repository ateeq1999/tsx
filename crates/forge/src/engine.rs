//! The forge Engine — a Tera wrapper with the 4-tier system, import hoisting,
//! and framework package loading built in.

use std::path::Path;
use tera::Tera;
use walkdir::WalkDir;

use crate::{
    cache::TemplateCache,
    collector,
    compose::{self, ExtendsGraph},
    context::ForgeContext,
    error::ForgeError,
    filters,
    preprocessor,
    provide,
    slots,
    tier::Tier,
};

/// The forge rendering engine.
///
/// Wraps Tera with:
/// - Custom filters: `snake_case`, `pascal_case`, `camel_case`, `kebab_case`
/// - Import hoisting filters: `collect_import`, `collect_import_priority`
/// - Import drain function: `render_imports()`
/// - Tier-aware template registry
/// - Circular `@extends` dependency detection (fails fast on load)
/// - Optional LRU source cache to skip redundant preprocessing on repeated `add_raw` calls
pub struct Engine {
    tera: Tera,
    /// Tracks `child → parent` `@extends` relationships for cycle detection.
    extends_graph: ExtendsGraph,
    /// Optional source-level cache. When set, `add_raw` skips reprocessing
    /// templates whose preprocessed source is already cached.
    cache: Option<TemplateCache>,
}

impl Engine {
    /// Create an engine with all forge extensions registered. No templates loaded yet.
    pub fn new() -> Self {
        let mut tera = Tera::default();
        register_extensions(&mut tera);
        Engine { tera, extends_graph: ExtendsGraph::new(), cache: None }
    }

    /// Attach a [`TemplateCache`] to this engine.
    ///
    /// When a cache is present, `add_raw` stores the preprocessed source so
    /// identical templates can be re-registered without redundant preprocessing.
    pub fn with_cache(mut self, cache: TemplateCache) -> Self {
        self.cache = Some(cache);
        self
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
                    let raw = std::fs::read_to_string(path)
                        .map_err(|e| ForgeError::LoadError(e.to_string()))?;
                    // Run @-directive preprocessor for .forge files
                    let content = if ext == "forge" {
                        preprocessor::preprocess(&raw)
                    } else {
                        raw
                    };
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
    /// If `name` ends in `.forge`, the `@`-directive preprocessor is applied first.
    ///
    /// When a [`TemplateCache`] is attached (via [`Engine::with_cache`]), the
    /// preprocessed source is stored and re-used on subsequent calls with the
    /// same `name`, avoiding redundant preprocessing.
    pub fn add_raw(&mut self, name: &str, content: &str) -> Result<(), ForgeError> {
        // Check cache first
        if let Some(cached) = self.cache.as_ref().and_then(|c| c.get(name)) {
            return self.tera
                .add_raw_template(name, &cached)
                .map_err(|e| ForgeError::LoadError(e.to_string()));
        }

        let processed;
        let final_content = if name.ends_with(".forge") {
            processed = preprocessor::preprocess(content);
            // Detect circular @extends before registering this template.
            if let Some(parent) = compose::extract_extends_path(&processed) {
                if compose::would_cycle(&self.extends_graph, name, &parent) {
                    return Err(ForgeError::CircularDependency(format!(
                        "{name} → {parent} (would create a cycle)"
                    )));
                }
                self.extends_graph.add(name, parent);
            }
            // Store in cache if one is attached
            if let Some(cache) = &self.cache {
                cache.put(name, &processed);
            }
            &processed
        } else {
            content
        };
        self.tera
            .add_raw_template(name, final_content)
            .map_err(|e| ForgeError::LoadError(e.to_string()))?;
        Ok(())
    }

    /// Render a template by name with the given context.
    /// Resets the ImportCollector and populates slots before rendering.
    pub fn render(&self, name: &str, ctx: &ForgeContext) -> Result<String, ForgeError> {
        collector::reset();
        slots::reset();
        provide::reset();
        // Populate thread-local slots from the context
        if let Some(slot_map) = ctx.slots() {
            for (k, v) in &slot_map {
                if let Some(content) = v.as_str() {
                    slots::fill(k, content);
                }
            }
        }
        // Populate thread-local provides from the context
        if let Some(provides_map) = ctx.provides() {
            for (k, v) in &provides_map {
                if let Some(content) = v.as_str() {
                    provide::provide(k, content);
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

    /// Return the current `@extends` dependency graph (useful for tooling).
    pub fn extends_graph(&self) -> &ExtendsGraph {
        &self.extends_graph
    }

    /// Start watching `dir` for template changes and call `on_change(path)` on each.
    ///
    /// Requires the `watch` feature: `tsx-forge = { features = ["watch"] }`.
    /// Returns the watcher handle; dropping it stops the watch.
    #[cfg(feature = "watch")]
    pub fn watch_dir<F>(
        &self,
        dir: &std::path::Path,
        on_change: F,
    ) -> Result<notify::RecommendedWatcher, String>
    where
        F: Fn(std::path::PathBuf) + Send + 'static,
    {
        crate::watch::watch_dir(dir, on_change)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}


fn register_extensions(tera: &mut Tera) {
    // Case conversion
    tera.register_filter("snake_case", filters::snake_case);
    tera.register_filter("pascal_case", filters::pascal_case);
    tera.register_filter("camel_case", filters::camel_case);
    tera.register_filter("kebab_case", filters::kebab_case);
    // Import hoisting
    tera.register_filter("collect_import", filters::collect_import);
    tera.register_filter("collect_import_priority", filters::collect_import_priority);
    tera.register_function("render_imports", filters::render_imports_fn);
    // String utilities
    tera.register_filter("slugify", filters::slugify);
    tera.register_filter("truncate_str", filters::truncate);
    tera.register_filter("indent", filters::indent);
    // JSON utilities
    tera.register_filter("json_encode", filters::json_encode);
    tera.register_filter("json_decode", filters::json_decode);
    tera.register_filter("debug", filters::debug_filter);
    // Inflection
    tera.register_filter("plural", filters::plural);
    tera.register_filter("singular", filters::singular);
    // System
    tera.register_filter("env", filters::env_filter);
    // Generation
    tera.register_function("random_id", filters::random_id_fn);
    // Slots and provides
    tera.register_function("slot", slots::make_slot_fn());
    tera.register_function("inject", provide::make_inject_fn());
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
