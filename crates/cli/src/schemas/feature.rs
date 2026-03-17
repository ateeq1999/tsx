use super::field::{FieldSchema, Operation};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddFeatureArgs {
    pub name: String,
    pub fields: Vec<FieldSchema>,
    #[serde(default)]
    pub auth: bool,
    #[serde(default)]
    pub paginated: bool,
    #[serde(default = "default_operations")]
    pub operations: Vec<Operation>,
}

fn default_operations() -> Vec<Operation> {
    vec![
        Operation::List,
        Operation::Create,
        Operation::Update,
        Operation::Delete,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_feature_args_deserialize() {
        let json = r#"{
            "name": "products",
            "fields": [
                { "name": "title", "type": "string", "required": true },
                { "name": "price", "type": "number", "required": true }
            ],
            "auth": true,
            "paginated": true,
            "operations": ["list", "create"]
        }"#;
        let args: AddFeatureArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.name, "products");
        assert_eq!(args.fields.len(), 2);
        assert!(args.auth);
        assert!(args.paginated);
    }
}
