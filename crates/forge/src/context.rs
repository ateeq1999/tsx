//! ForgeContext — a context builder for forge template rendering.

use serde::Serialize;

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
