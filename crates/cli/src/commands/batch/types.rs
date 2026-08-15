use serde::{Deserialize, Serialize};

use crate::json::error::ErrorCode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub results: Vec<BatchCommandResult>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rolled_back_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCommandResult {
    pub index: u32,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BatchError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::payload::BatchPayload;

    #[test]
    fn batch_result_omits_empty_rolled_back() {
        let result = BatchResult {
            total: 1,
            succeeded: 1,
            failed: 0,
            results: vec![],
            rolled_back_files: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("rolled_back_files"), "empty rollback should be omitted");
    }

    #[test]
    fn batch_result_includes_rolled_back_when_present() {
        let result = BatchResult {
            total: 2,
            succeeded: 1,
            failed: 1,
            results: vec![],
            rolled_back_files: vec!["src/foo.ts".to_string()],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("rolled_back_files"));
        assert!(json.contains("src/foo.ts"));
    }

    #[test]
    fn batch_payload_rollback_defaults_false() {
        let json = r#"{"commands":[]}"#;
        let payload: BatchPayload = serde_json::from_str(json).unwrap();
        assert!(!payload.rollback_on_failure);
        assert!(!payload.stop_on_failure);
    }

    #[test]
    fn batch_payload_rollback_explicit_true() {
        let json = r#"{"commands":[],"stop_on_failure":true,"rollback_on_failure":true}"#;
        let payload: BatchPayload = serde_json::from_str(json).unwrap();
        assert!(payload.stop_on_failure);
        assert!(payload.rollback_on_failure);
    }

    #[test]
    fn batch_error_uses_error_code_enum() {
        let err = BatchError {
            code: ErrorCode::ValidationError,
            message: "bad input".to_string(),
            path: None,
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("VALIDATIONERROR") || json.contains("ValidationError") || json.contains("VALIDATION_ERROR"));
    }
}
