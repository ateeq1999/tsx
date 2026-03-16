//! Provide/Inject context propagation for forge.
//!
//! Provides a mechanism for templates to share data across nested renders without
//! threading it through every context. The pattern mirrors Vue's `provide/inject` API.
//!
//! Values are stored in a thread-local registry, populated before each render and
//! accessible through the `inject()` Tera function.
//!
//! ## Usage
//!
//! **In templates:**
//! ```jinja
//! {# Read a provided value #}
//! {% set theme = inject(key="theme") %}
//! {% set user = inject(key="currentUser", default="guest") %}
//! ```
//!
//! **In Rust:**
//! ```rust
//! use forge::{Engine, ForgeContext};
//!
//! let mut engine = Engine::new();
//! engine.add_raw("t.jinja", "Hello {{ inject(key='user') }}!").unwrap();
//!
//! let ctx = ForgeContext::new().provide("user", "Alice");
//! let out = engine.render("t.jinja", &ctx).unwrap();
//! assert_eq!(out, "Hello Alice!");
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use tera::{Function, Value};

thread_local! {
    static PROVIDES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Reset the thread-local provide store (called before each render).
pub fn reset() {
    PROVIDES.with(|p| p.borrow_mut().clear());
}

/// Provide a named value. Called by `ForgeContext::provide()` via `Engine::render()`.
pub fn provide(key: &str, value: &str) {
    PROVIDES.with(|p| {
        p.borrow_mut().insert(key.to_string(), value.to_string());
    });
}

/// Build the `inject()` Tera function.
///
/// Template usage: `inject(key="theme")` or `inject(key="user", default="guest")`
pub fn make_inject_fn() -> impl Function {
    move |args: &HashMap<String, Value>| -> tera::Result<Value> {
        let key = args
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("inject(): 'key' argument is required"))?;

        let default_val = args
            .get("default")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let value = PROVIDES.with(|p| p.borrow().get(key).cloned());

        Ok(Value::String(value.unwrap_or_else(|| default_val.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Engine, ForgeContext};

    #[test]
    fn inject_reads_provided_value() {
        let mut engine = Engine::new();
        engine
            .add_raw("t.jinja", "{{ inject(key='theme') }}")
            .unwrap();
        let ctx = ForgeContext::new().provide("theme", "dark");
        let out = engine.render("t.jinja", &ctx).unwrap();
        assert_eq!(out, "dark");
    }

    #[test]
    fn inject_uses_default_when_not_provided() {
        let mut engine = Engine::new();
        engine
            .add_raw("t.jinja", "{{ inject(key='locale', default='en') }}")
            .unwrap();
        let ctx = ForgeContext::new();
        let out = engine.render("t.jinja", &ctx).unwrap();
        assert_eq!(out, "en");
    }

    #[test]
    fn inject_empty_string_when_no_default() {
        let mut engine = Engine::new();
        engine.add_raw("t.jinja", "[{{ inject(key='x') }}]").unwrap();
        let ctx = ForgeContext::new();
        let out = engine.render("t.jinja", &ctx).unwrap();
        assert_eq!(out, "[]");
    }

    #[test]
    fn inject_reset_between_renders() {
        let mut engine = Engine::new();
        engine
            .add_raw("t.jinja", "{{ inject(key='user') }}")
            .unwrap();

        let ctx1 = ForgeContext::new().provide("user", "Alice");
        let out1 = engine.render("t.jinja", &ctx1).unwrap();
        assert_eq!(out1, "Alice");

        // Second render — no provide; should be empty
        let ctx2 = ForgeContext::new();
        let out2 = engine.render("t.jinja", &ctx2).unwrap();
        assert_eq!(out2, "");
    }

    #[test]
    fn multiple_provides_in_one_context() {
        let mut engine = Engine::new();
        engine
            .add_raw("t.jinja", "{{ inject(key='a') }}-{{ inject(key='b') }}")
            .unwrap();
        let ctx = ForgeContext::new().provide("a", "X").provide("b", "Y");
        let out = engine.render("t.jinja", &ctx).unwrap();
        assert_eq!(out, "X-Y");
    }
}
