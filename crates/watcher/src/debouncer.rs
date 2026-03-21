//! Simple debouncer: accumulates paths and flushes after `window` has elapsed
//! since the last addition.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct Debouncer {
    window: Duration,
    pending: HashSet<PathBuf>,
    last_event: Option<Instant>,
}

impl Debouncer {
    pub fn new(window: Duration) -> Self {
        Self { window, pending: HashSet::new(), last_event: None }
    }

    pub fn add(&mut self, path: PathBuf) {
        self.pending.insert(path);
        self.last_event = Some(Instant::now());
    }

    /// Returns the batch if the debounce window has elapsed, else `None`.
    pub fn flush(&mut self) -> Option<Vec<PathBuf>> {
        let elapsed = self.last_event?.elapsed();
        if elapsed >= self.window && !self.pending.is_empty() {
            let batch: Vec<PathBuf> = self.pending.drain().collect();
            self.last_event = None;
            Some(batch)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_before_window() {
        let mut d = Debouncer::new(Duration::from_secs(10));
        d.add(PathBuf::from("foo.ts"));
        assert!(d.flush().is_none(), "should not flush before window");
    }

    #[test]
    fn flushes_after_window() {
        let mut d = Debouncer::new(Duration::from_millis(1));
        d.add(PathBuf::from("foo.ts"));
        std::thread::sleep(Duration::from_millis(5));
        let batch = d.flush();
        assert!(batch.is_some());
        assert_eq!(batch.unwrap().len(), 1);
    }

    #[test]
    fn deduplicates_paths() {
        let mut d = Debouncer::new(Duration::from_millis(1));
        d.add(PathBuf::from("foo.ts"));
        d.add(PathBuf::from("foo.ts"));
        d.add(PathBuf::from("bar.ts"));
        std::thread::sleep(Duration::from_millis(5));
        let batch = d.flush().unwrap();
        assert_eq!(batch.len(), 2);
    }
}
