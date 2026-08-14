use std::time::Duration;
use tsx_watcher::{WatchConfig, Watcher};

/// Watch files and re-run a generator/command on change.
pub fn watch(paths: Vec<String>, ext: Vec<String>, debounce: u64, run_cmd: Option<String>, json: bool) {
    let roots = if paths.is_empty() { vec![".".to_string()] } else { paths };
    let extensions = if ext.is_empty() {
        vec!["ts".into(), "tsx".into(), "js".into(), "rs".into(), "forge".into(), "jinja".into()]
    } else {
        ext
    };
    let config = WatchConfig {
        roots,
        extensions,
        debounce: Duration::from_millis(debounce),
        json_events: json,
    };

    let watcher = Watcher::new(config);
    eprintln!("Watching for changes... (Ctrl-C to stop)");

    let _ = watcher.start(|changed| {
        for path in &changed {
            println!("changed: {}", path.display());
        }
        if let Some(cmd) = &run_cmd {
            let status = std::process::Command::new("sh").args(["-c", cmd]).status();
            match status {
                Ok(s) if !s.success() => eprintln!("Command exited with {}", s),
                Err(e) => eprintln!("Failed to run command: {}", e),
                _ => {}
            }
        }
        true // keep watching
    });
}
