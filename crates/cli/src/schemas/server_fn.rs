use super::field::Operation;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddServerFnArgs {
    pub name: String,
    pub table: String,
    pub operation: Operation,
    #[serde(default)]
    pub auth: bool,
    pub input: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_server_fn_args_deserialize() {
        let json = r#"{
            "name": "createProduct",
            "table": "products",
            "operation": "create",
            "auth": true
        }"#;
        let args: AddServerFnArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.name, "createProduct");
        assert_eq!(args.table, "products");
        assert_eq!(args.operation, Operation::Create);
        assert!(args.auth);
    }
}
