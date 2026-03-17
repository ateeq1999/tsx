use super::field::FieldSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddTableArgs {
    /// Resource name (e.g. "product", "order")
    pub name: String,
    /// Columns to display in the table
    pub fields: Vec<FieldSchema>,
    /// Name of the server function / query key used to fetch rows
    #[serde(rename = "queryFn", default)]
    pub query_fn: String,
    /// Enable server-side pagination (default: true)
    #[serde(default = "default_true")]
    pub paginated: bool,
    /// Enable column sorting (default: true)
    #[serde(default = "default_true")]
    pub sortable: bool,
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::field::FieldType;

    #[test]
    fn deserialise_minimal() {
        let json = r#"{ "name": "product", "fields": [{ "name": "title", "type": "string" }] }"#;
        let args: AddTableArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.name, "product");
        assert_eq!(args.fields.len(), 1);
        assert_eq!(args.fields[0].field_type, FieldType::String);
        assert!(args.paginated);
        assert!(args.sortable);
    }

    #[test]
    fn deserialise_full() {
        let json = r#"{
            "name": "order",
            "fields": [{ "name": "total", "type": "decimal" }],
            "queryFn": "getOrders",
            "paginated": false,
            "sortable": true
        }"#;
        let args: AddTableArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.query_fn, "getOrders");
        assert!(!args.paginated);
    }
}
