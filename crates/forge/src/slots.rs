//! Component slot system for forge.
//!
//! Slots are named content injection points that templates can define as overridable
//! regions. This mirrors the Vue/Svelte slot pattern and the atom-engine slot model.
//!
//! Uses thread-local storage (same pattern as `ImportCollector`) so the registered
//! `slot()` Tera function can access slot content without needing context access.
//!
//! ## Usage
//!
//! **In a template** — declare a slot with optional default content:
//!
//! ```jinja
//! {{ slot(name="header") }}
//! {{ slot(name="body") }}
//! {{ slot(name="footer", default="No footer provided.") }}
//! ```
//!
//! **In Rust** — fill slots via `ForgeContext`:
//!
//! ```rust
//! use forge::{Engine, ForgeContext};
//!
//! let mut engine = Engine::new();
//! engine.add_raw("card.jinja", "{{ slot(name='header') }}|{{ slot(name='body') }}").unwrap();
//!
//! let ctx = ForgeContext::new()
//!     .slot("header", "<h1>Title</h1>")
//!     .slot("body", "<p>Content</p>");
//!
//! let out = engine.render("card.jinja", &ctx).unwrap();
//! assert_eq!(out, "<h1>Title</h1>|<p>Content</p>");
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use tera::{Function, Value};

thread_local! {
    static SLOTS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Reset the thread-local slot map (called before each render).
pub fn reset() {
    SLOTS.with(|s| s.borrow_mut().clear());
}

/// Fill a named slot with content. Called by `ForgeContext::slot()`.
pub fn fill(name: &str, content: &str) {
    SLOTS.with(|s| {
        s.borrow_mut().insert(name.to_string(), content.to_string());
    });
}

/// Read all currently filled slots (for debugging / introspection).
#[allow(dead_code)]
pub fn snapshot() -> HashMap<String, String> {
    SLOTS.with(|s| s.borrow().clone())
}

/// Build the `slot()` Tera function.
///
/// Template usage: `{{ slot(name="header") }}` or `{{ slot(name="footer", default="—") }}`
pub fn make_slot_fn() -> impl Function {
    move |args: &HashMap<String, Value>| -> tera::Result<Value> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("slot(): 'name' argument is required"))?;

        let default_val = args
            .get("default")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let content = SLOTS.with(|s| s.borrow().get(name).cloned());

        Ok(Value::String(content.unwrap_or_else(|| default_val.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Engine, ForgeContext};

    #[test]
    fn slot_filled_from_context() {
        let mut engine = Engine::new();
        engine
            .add_raw("card.jinja", "{{ slot(name='header') }}|{{ slot(name='body') }}")
            .unwrap();
        let ctx = ForgeContext::new()
            .slot("header", "TITLE")
            .slot("body", "BODY");
        let out = engine.render("card.jinja", &ctx).unwrap();
        assert_eq!(out, "TITLE|BODY");
    }

    #[test]
    fn slot_uses_default_when_not_filled() {
        let mut engine = Engine::new();
        engine
            .add_raw("card.jinja", "{{ slot(name='footer', default='No footer') }}")
            .unwrap();
        let ctx = ForgeContext::new();
        let out = engine.render("card.jinja", &ctx).unwrap();
        assert_eq!(out, "No footer");
    }

    #[test]
    fn slot_empty_string_when_no_default() {
        let mut engine = Engine::new();
        engine.add_raw("t.jinja", "[{{ slot(name='x') }}]").unwrap();
        let ctx = ForgeContext::new();
        let out = engine.render("t.jinja", &ctx).unwrap();
        assert_eq!(out, "[]");
    }

    #[test]
    fn slot_reset_between_renders() {
        let mut engine = Engine::new();
        engine
            .add_raw("t.jinja", "{{ slot(name='a') }}")
            .unwrap();

        // First render with slot filled
        let ctx1 = ForgeContext::new().slot("a", "first");
        let out1 = engine.render("t.jinja", &ctx1).unwrap();
        assert_eq!(out1, "first");

        // Second render without slot — should be empty
        let ctx2 = ForgeContext::new();
        let out2 = engine.render("t.jinja", &ctx2).unwrap();
        assert_eq!(out2, "");
    }

    #[test]
    fn slot_multiple_slots_in_one_template() {
        let mut engine = Engine::new();
        engine
            .add_raw(
                "layout.jinja",
                "<header>{{ slot(name='nav') }}</header><main>{{ slot(name='content', default='empty') }}</main>",
            )
            .unwrap();
        let ctx = ForgeContext::new().slot("nav", "NAV").slot("content", "MAIN");
        let out = engine.render("layout.jinja", &ctx).unwrap();
        assert_eq!(out, "<header>NAV</header><main>MAIN</main>");
    }
}
