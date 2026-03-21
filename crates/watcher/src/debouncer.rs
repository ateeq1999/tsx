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

    #[test]
    fn flush_clears_pending() {
        let mut d = Debouncer::new(Duration::from_millis(1));
        d.add(PathBuf::from("a.ts"));
        std::thread::sleep(Duration::from_millis(5));
        d.flush().unwrap();
        // second flush should return None — nothing pending
        assert!(d.flush().is_none(), "pending should be empty after flush");
    }

    #[test]
    fn empty_debouncer_flush_returns_none() {
        let mut d = Debouncer::new(Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(5));
        assert!(d.flush().is_none(), "flush with nothing added should return None");
    }

    #[test]
    fn many_rapid_events_produce_one_batch() {
        let mut d = Debouncer::new(Duration::from_millis(50));
        for i in 0..20 {
            d.add(PathBuf::from(format!("file{}.ts", i)));
        }
        // Window has not elapsed — should still be pending
        assert!(d.flush().is_none(), "should not flush before window");
        std::thread::sleep(Duration::from_millis(60));
        let batch = d.flush().unwrap();
        assert_eq!(batch.len(), 20, "all 20 unique files should flush as one batch");
    }
}
