use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddQueryArgs {
    pub name: String,
    #[serde(rename = "serverFn")]
    pub server_fn: String,
    #[serde(default)]
    pub suspense: bool,
    #[serde(default)]
    pub mutation: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_query_args_deserialize() {
        let json = r#"{
            "name": "products",
            "serverFn": "getProducts",
            "suspense": true,
            "mutation": false
        }"#;
        let args: AddQueryArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.name, "products");
        assert_eq!(args.server_fn, "getProducts");
        assert!(args.suspense);
        assert!(!args.mutation);
    }
}
