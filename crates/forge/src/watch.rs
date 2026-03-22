//! Hot template reload using the `notify` file-watcher crate.
//!
//! Only available when the `watch` feature is enabled.
//!
//! # Usage
//!
//! ```rust,no_run
//! # #[cfg(feature = "watch")]
//! # {
//! use std::sync::{Arc, RwLock};
//! use tsx_forge::{Engine, watch::watch_dir};
//!
//! let engine = Arc::new(RwLock::new(Engine::new()));
//! let engine_clone = Arc::clone(&engine);
//!
//! let _watcher = watch_dir("./templates", move |path| {
//!     if let Ok(content) = std::fs::read_to_string(&path) {
//!         let name = path.file_name().unwrap().to_string_lossy();
//!         if let Ok(mut e) = engine_clone.write() {
//!             let _ = e.add_raw(&name, &content);
//!         }
//!     }
//! });
//! # }
//! ```

#[cfg(feature = "watch")]
pub use inner::watch_dir;

#[cfg(feature = "watch")]
mod inner {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

    /// Start a file watcher on `dir`.
    ///
    /// `on_change` is called with the absolute path of each `.forge` or `.jinja`
    /// file that is created or modified.  Returns the watcher handle; dropping it
    /// stops the watch.
    ///
    /// # Errors
    ///
    /// Returns an error string if the watcher cannot be initialised.
    pub fn watch_dir<F>(dir: impl AsRef<Path>, on_change: F) -> Result<RecommendedWatcher, String>
    where
        F: Fn(PathBuf) + Send + 'static,
    {
        let dir = dir.as_ref().to_path_buf();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        for path in event.paths {
                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            if ext == "forge" || ext == "jinja" {
                                on_change(path);
                            }
                        }
                    }
                    _ => {}
                }
            }
        })
        .map_err(|e| e.to_string())?;

        watcher
            .watch(&dir, RecursiveMode::Recursive)
            .map_err(|e| e.to_string())?;

        Ok(watcher)
    }
}

/// Stub exported when the `watch` feature is disabled so code that imports
/// this module still compiles without the feature enabled.
#[cfg(not(feature = "watch"))]
pub fn watch_dir_unavailable() -> &'static str {
    "Hot reload requires the 'watch' feature: tsx-forge = { features = [\"watch\"] }"
}
