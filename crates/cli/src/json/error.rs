use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ErrorCode {
    InvalidPayload,
    ValidationError,
    UnknownCommand,
    UnknownKind,
    FileExists,
    DirectoryNotFound,
    PermissionDenied,
    TemplateNotFound,
    ProjectNotFound,
    InternalError,
    Unauthorized,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::InvalidPayload => "INVALID_PAYLOAD",
            ErrorCode::ValidationError => "VALIDATION_ERROR",
            ErrorCode::UnknownCommand => "UNKNOWN_COMMAND",
            ErrorCode::UnknownKind => "UNKNOWN_KIND",
            ErrorCode::FileExists => "FILE_EXISTS",
            ErrorCode::DirectoryNotFound => "DIRECTORY_NOT_FOUND",
            ErrorCode::PermissionDenied => "PERMISSION_DENIED",
            ErrorCode::TemplateNotFound => "TEMPLATE_NOT_FOUND",
            ErrorCode::ProjectNotFound => "PROJECT_NOT_FOUND",
            ErrorCode::InternalError => "INTERNAL_ERROR",
            ErrorCode::Unauthorized => "UNAUTHORIZED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorDetail {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<ValidationErrorDetail>,
}

impl ErrorResponse {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub fn with_detail(mut self, field: impl Into<String>, message: impl Into<String>) -> Self {
        self.details.push(ValidationErrorDetail {
            field: field.into(),
            message: message.into(),
        });
        self
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationError, message)
    }

    pub fn invalid_payload(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidPayload, message)
    }

    pub fn file_exists(path: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::FileExists,
            format!("File already exists: {}", path.into()),
        )
    }

    pub fn project_not_found() -> Self {
        Self::new(
            ErrorCode::ProjectNotFound,
            "Not running inside a TanStack Start project (no package.json found)",
        )
    }

    pub fn unknown_command(cmd: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::UnknownCommand,
            format!("Unknown command: {}", cmd.into()),
        )
    }

    pub fn unknown_kind(kind: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::UnknownKind,
            format!("Unknown kind: {}", kind.into()),
        )
    }
}
