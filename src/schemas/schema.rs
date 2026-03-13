use super::field::FieldSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddSchemaArgs {
    pub name: String,
    pub fields: Vec<FieldSchema>,
    #[serde(default)]
    pub timestamps: bool,
    #[serde(default)]
    pub soft_delete: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_schema_args_deserialize() {
        let json = r#"{
            "name": "categories",
            "fields": [
                { "name": "name", "type": "string", "unique": true }
            ],
            "timestamps": true,
            "softDelete": false
        }"#;
        let args: AddSchemaArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.name, "categories");
        assert_eq!(args.fields.len(), 1);
        assert!(args.timestamps);
        assert!(!args.soft_delete);
    }
}
