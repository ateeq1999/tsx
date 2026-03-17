use super::field::FieldSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddFormArgs {
    pub name: String,
    pub fields: Vec<FieldSchema>,
    #[serde(rename = "submitFn")]
    pub submit_fn: String,
    #[serde(default)]
    pub layout: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_form_args_deserialize() {
        let json = r#"{
            "name": "product",
            "fields": [
                { "name": "title", "type": "string", "required": true }
            ],
            "submitFn": "createProduct"
        }"#;
        let args: AddFormArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.name, "product");
        assert_eq!(args.fields.len(), 1);
        assert_eq!(args.submit_fn, "createProduct");
    }
}
