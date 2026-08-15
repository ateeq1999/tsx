use serde::Serialize;

#[derive(Serialize)]
pub(super) struct RunResult {
    pub id: String,
    pub command: String,
    pub framework: String,
    pub files_created: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub next_steps: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run_paths: Option<Vec<String>>,
    /// Approximate tokens an LLM would have spent writing this code manually.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_saved: Option<u32>,
}

#[derive(Serialize)]
pub(super) struct RunListEntry {
    pub id: String,
    pub command: String,
    pub framework: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_estimate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}
