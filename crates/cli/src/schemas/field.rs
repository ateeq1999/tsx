use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Date,
    Id,
    Enum,
    Json,
    Decimal,
    Email,
    Url,
    Password,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FieldSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default = "default_true")]
    pub required: bool,
    pub unique: Option<bool>,
    pub references: Option<String>,
    pub values: Option<Vec<String>>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    List,
    Create,
    Update,
    Delete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_schema_deserialize() {
        let json = r#"{
            "name": "title",
            "type": "string",
            "required": true,
            "unique": false
        }"#;
        let field: FieldSchema = serde_json::from_str(json).unwrap();
        assert_eq!(field.name, "title");
        assert_eq!(field.field_type, FieldType::String);
        assert!(field.required);
        assert!(!field.unique.unwrap());
    }

    #[test]
    fn test_operation_deserialize() {
        let json = r#""list""#;
        let op: Operation = serde_json::from_str(json).unwrap();
        assert_eq!(op, Operation::List);
    }
}
