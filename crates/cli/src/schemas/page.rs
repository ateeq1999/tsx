use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddPageArgs {
    pub path: String,
    pub title: Option<String>,
    #[serde(default)]
    pub auth: bool,
    #[serde(default)]
    pub loader: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_page_args_deserialize() {
        let json = r#"{
            "path": "/products",
            "title": "Products",
            "auth": true
        }"#;
        let args: AddPageArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.path, "/products");
        assert_eq!(args.title, Some("Products".to_string()));
        assert!(args.auth);
    }
}
