use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub command: String,
    pub files_created: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub next_steps: Vec<String>,
}

impl CommandResult {
    pub fn ok(command: impl Into<String>, files_created: Vec<String>) -> Self {
        Self {
            success: true,
            command: command.into(),
            files_created,
            error: None,
            warnings: Vec::new(),
            next_steps: Vec::new(),
        }
    }

    pub fn err(command: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            command: command.into(),
            files_created: Vec::new(),
            error: Some(message.into()),
            warnings: Vec::new(),
            next_steps: Vec::new(),
        }
    }

    pub fn print(&self) {
        let json = serde_json::to_string_pretty(self).unwrap();
        println!("{}", json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_round_trip() {
        let result = CommandResult::ok("add:feature", vec!["file1.ts".to_string()]);
        let json = serde_json::to_string(&result).unwrap();
        let parsed: CommandResult = serde_json::from_str(&json).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.command, "add:feature");
        assert_eq!(parsed.files_created.len(), 1);
    }

    #[test]
    fn test_command_result_error() {
        let result = CommandResult::err("add:feature", "Something went wrong");
        let json = serde_json::to_string(&result).unwrap();
        let parsed: CommandResult = serde_json::from_str(&json).unwrap();
        assert!(!parsed.success);
        assert_eq!(parsed.error.as_deref(), Some("Something went wrong"));
        assert!(parsed.warnings.is_empty());
    }
}
