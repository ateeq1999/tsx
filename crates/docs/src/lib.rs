//! **tsx-docs** — offline documentation viewer (Priority 13).
//!
//! Renders `.md` files from `.tsx/knowledge/` and embedded framework docs in
//! a navigable terminal UI. Uses ratatui for the initial implementation
//! (iocraft migration planned for streaming markdown rendering).
//!
//! ## Views
//! - **Topic list** — searchable list of documentation topics
//! - **Doc reader** — scrollable Markdown renderer (inline formatting)
//! - **Search** — full-text search across all topics

pub mod markdown;
pub mod reader;
pub mod topic;

pub use reader::run_docs_viewer;
pub use topic::{DocTopic, collect_topics, default_roots};
