use std::path::PathBuf;

use crate::json::response::ResponseEnvelope;

use super::runner::run_snapshots;

pub fn snapshot_diff(generator: Option<String>, verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    run_snapshots(&cwd, generator.as_deref(), true, verbose)
}
