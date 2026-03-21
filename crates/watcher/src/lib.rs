//! **tsx-watcher** — file watcher + live regeneration (Section B).
//!
//! Replaces the ad-hoc `--watch` flag implementations scattered across individual
//! codegen commands with a unified, debounced watcher that:
//!
//! 1. Watches a set of source glob patterns
//! 2. Debounces rapid changes (default 300ms)
//! 3. Calls a user-supplied callback with the list of changed paths
//! 4. Emits structured JSON events on stdout for agent consumption
//!
//! ## Usage
//!
//! ```rust
//! use tsx_watcher::{Watcher, WatchConfig};
//! use std::time::Duration;
//!
//! let config = WatchConfig {
//!     roots: vec!["crates/shared/src".into()],
//!     extensions: vec!["rs".into()],
//!     debounce: Duration::from_millis(300),
//!     json_events: false,
//! };
//!
//! let mut watcher = Watcher::new(config);
//! watcher.start(|changed| {
//!     println!("Changed: {:?}", changed);
//! }).unwrap();
//! ```

pub mod config;
pub mod debouncer;
pub mod event;

pub use config::WatchConfig;
pub use event::{WatchEvent, EventKind};

use std::sync::mpsc;
use std::time::Duration;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Watcher
// ---------------------------------------------------------------------------

/// A debounced file watcher.
pub struct Watcher {
    pub config: WatchConfig,
}

impl Watcher {
    pub fn new(config: WatchConfig) -> Self {
        Self { config }
    }

    /// Start watching. Calls `callback` with a list of changed absolute paths
    /// whenever a debounced batch arrives. Blocks until Ctrl-C or the callback
    /// returns `false`.
    pub fn start<F>(&self, mut callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(Vec<PathBuf>) -> bool,
    {
        use notify::{Watcher as NotifyWatcher, RecursiveMode, recommended_watcher};

        let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

        let mut watcher = recommended_watcher(move |evt| {
            let _ = tx.send(evt);
        })?;

        for root in &self.config.roots {
            let path = PathBuf::from(root);
            if path.exists() {
                watcher.watch(&path, RecursiveMode::Recursive)?;
            }
        }

        if self.config.json_events {
            let evt = WatchEvent::started(&self.config.roots);
            println!("{}", serde_json::to_string(&evt).unwrap_or_default());
        }

        let mut debouncer = debouncer::Debouncer::new(self.config.debounce);

        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok(event)) => {
                    let paths: Vec<PathBuf> = event
                        .paths
                        .into_iter()
                        .filter(|p| self.matches_extension(p))
                        .collect();
                    for path in paths {
                        debouncer.add(path);
                    }
                }
                Ok(Err(e)) => {
                    if self.config.json_events {
                        let evt = WatchEvent::error(&e.to_string());
                        eprintln!("{}", serde_json::to_string(&evt).unwrap_or_default());
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }

            if let Some(batch) = debouncer.flush() {
                if !batch.is_empty() {
                    if self.config.json_events {
                        let evt = WatchEvent::changed(&batch);
                        println!("{}", serde_json::to_string(&evt).unwrap_or_default());
                    }
                    if !callback(batch) {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn matches_extension(&self, path: &PathBuf) -> bool {
        if self.config.extensions.is_empty() {
            return true;
        }
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.config.extensions.iter().any(|e| e == ext))
            .unwrap_or(false)
    }
}
