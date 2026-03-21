//! ForgeContext — a context builder for forge template rendering.

use serde::Serialize;
use serde_json;

/// A typed context passed to `Engine::render()`.
/// Wraps `tera::Context` and provides a builder-style API.
pub struct ForgeContext {
    inner: tera::Context,
}

impl ForgeContext {
    pub fn new() -> Self {
        Self {
            inner: tera::Context::new(),
        }
    }

    /// Insert a serializable value (builder style, consumes self).
    pub fn insert<T: Serialize + ?Sized>(mut self, key: &str, value: &T) -> Self {
        self.inner.insert(key, value);
        self
    }

    /// Insert a serializable value (mutable style, for use in loops).
    pub fn insert_mut<T: Serialize + ?Sized>(&mut self, key: &str, value: &T) {
        self.inner.insert(key, value);
    }

    /// Register a named slot with content to be injected into the template.
    ///
    /// Slots use thread-local storage and are populated just before the render
    /// call inside `Engine::render()`. This method stages them on the context
    /// so that `Engine` can apply them before delegating to Tera.
    pub fn slot(mut self, name: &str, content: &str) -> Self {
        // Store slots as a plain JSON map under "__slots__" key so we can
        // iterate them in engine.rs at render time without leaking internals.
        let mut current = self
            .inner
            .get("__slots__")
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        current.insert(
            name.to_string(),
            serde_json::Value::String(content.to_string()),
        );
        self.inner.insert("__slots__", &current);
        self
    }

    /// Register a provided value to be made available via `inject(key=...)` in templates.
    /// Values are stored under `__provides__` and applied to the thread-local store
    /// at render time by `Engine::render()`.
    pub fn provide(mut self, key: &str, value: &str) -> Self {
        let mut current = self
            .inner
            .get("__provides__")
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        current.insert(
            key.to_string(),
            serde_json::Value::String(value.to_string()),
        );
        self.inner.insert("__provides__", &current);
        self
    }

    /// Return the provides map from this context (for engine use at render time).
    pub(crate) fn provides(&self) -> Option<serde_json::Map<String, serde_json::Value>> {
        self.inner
            .get("__provides__")
            .and_then(|v| v.as_object().cloned())
    }

    /// Return the slot map from this context (for engine use at render time).
    pub(crate) fn slots(&self) -> Option<serde_json::Map<String, serde_json::Value>> {
        self.inner
            .get("__slots__")
            .and_then(|v| v.as_object().cloned())
    }

    /// Inject a `style` object so templates can branch on `style.forms`, etc. (C2).
    ///
    /// Call this after constructing the context:
    /// ```rust
    /// let ctx = ForgeContext::new()
    ///     .insert("name", "users")
    ///     .with_style(style_map);
    /// ```
    /// Inside templates:
    /// ```jinja
    /// {% if style.forms == "tanstack-form" %}
    ///   import { useForm } from "@tanstack/react-form"
    /// {% elif style.forms == "react-hook-form" %}
    ///   import { useForm } from "react-hook-form"
    /// {% endif %}
    /// ```
    pub fn with_style<T: Serialize>(mut self, style: &T) -> Self {
        self.inner.insert("style", style);
        self
    }

    /// Build a context from a serializable struct.
    pub fn from_serialize<T: Serialize>(data: &T) -> anyhow::Result<Self> {
        let inner = tera::Context::from_serialize(data)
            .map_err(|e| anyhow::anyhow!("Context serialization failed: {e}"))?;
        Ok(Self { inner })
    }

    pub(crate) fn as_tera(&self) -> &tera::Context {
        &self.inner
    }
}

impl Default for ForgeContext {
    fn default() -> Self {
        Self::new()
    }
}
