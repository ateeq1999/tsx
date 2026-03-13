use minijinja::Value;
use serde::Serialize;

pub struct RenderContext {
    values: std::collections::HashMap<String, Value>,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            values: std::collections::HashMap::new(),
        }
    }

    pub fn insert<T: Serialize>(mut self, key: impl Into<String>, value: T) -> Self {
        let key = key.into();
        let minijinja_value = Value::from_serialize(value);
        self.values.insert(key, minijinja_value);
        self
    }

    pub fn build(self) -> std::collections::HashMap<String, Value> {
        self.values
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_context() {
        let ctx = RenderContext::new()
            .insert("name", "products")
            .insert("fields", vec!["id", "title"])
            .build();

        let name = ctx.get("name").unwrap();
        assert_eq!(name.as_str(), Some("products"));
    }
}
