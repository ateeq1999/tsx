//! tsx-lsp — Language Server for .tsx/ config files and .forge templates.
//!
//! Provides completions, hover documentation, and diagnostics for:
//! - `.tsx/stack.json` / `user-stack.json` — config key completions and hover docs
//! - `.forge` / `.jinja` template files — template variable completions + syntax diagnostics

pub mod completions;
pub mod diagnostics;
pub mod hover;
pub mod server;
mod transport;

pub use server::run_lsp_server;
