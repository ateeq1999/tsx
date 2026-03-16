use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::output::CommandResult;
use crate::utils::paths::find_project_root;

/// All event types emitted by `tsx dev --json-events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Started,
    FileChanged,
    FileAdded,
    FileDeleted,
    BuildStarted,
    BuildCompleted,
    BuildFailed,
    HotReload,
    Error,
    Stopped,
}

/// A single JSON event line written to stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevEvent {
    pub event: EventType,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl DevEvent {
    pub fn new(event: EventType, data: Option<serde_json::Value>) -> Self {
        DevEvent {
            event,
            timestamp: iso_timestamp(),
            data,
        }
    }

    /// Emit this event as a single JSON line to stdout.
    pub fn emit(&self) {
        if let Ok(line) = serde_json::to_string(self) {
            println!("{}", line);
        }
    }
}

fn iso_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Minimal ISO-8601 UTC timestamp without chrono dependency.
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let year = 1970 + days / 365;
    let day_of_year = days % 365 + 1;
    let month = (day_of_year / 30).min(11) + 1;
    let day = day_of_year % 30 + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, h, m, s
    )
}

/// A broadcast channel for WebSocket clients.
type EventBroadcast = Arc<Mutex<Vec<std::sync::mpsc::Sender<String>>>>;

/// Start a WebSocket server on the given port that broadcasts DevEvents to connected clients.
///
/// Returns a handle, a broadcast channel, and a shutdown flag.
/// Set `shutdown.store(true, Ordering::Relaxed)` then connect once to unblock the accept loop.
fn start_ws_server(
    port: u16,
) -> (
    std::thread::JoinHandle<()>,
    EventBroadcast,
    Arc<AtomicBool>,
) {
    let clients: EventBroadcast = Arc::new(Mutex::new(Vec::new()));
    let clients_clone = Arc::clone(&clients);
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    let handle = std::thread::spawn(move || {
        use std::net::TcpListener;
        use tungstenite::accept;

        let addr = format!("127.0.0.1:{}", port);
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[tsx dev] WebSocket server failed to bind {}: {}", addr, e);
                return;
            }
        };

        eprintln!("[tsx dev] WebSocket server listening on ws://{}", addr);

        for stream in listener.incoming().flatten() {
            if shutdown_clone.load(Ordering::Relaxed) {
                break;
            }
            let clients_ref = Arc::clone(&clients_clone);
            std::thread::spawn(move || {
                let mut ws = match accept(stream) {
                    Ok(ws) => ws,
                    Err(_) => return,
                };

                let (tx, rx) = std::sync::mpsc::channel::<String>();
                {
                    let mut lock = clients_ref.lock().unwrap();
                    lock.push(tx);
                }

                // Welcome message
                let welcome = serde_json::to_string(&DevEvent::new(
                    EventType::Started,
                    Some(serde_json::json!({ "message": "tsx dev WebSocket connected" })),
                ))
                .unwrap_or_default();
                let _ = ws.send(tungstenite::Message::Text(welcome.into()));

                // Forward events until the channel is closed.
                for msg in rx {
                    if ws
                        .send(tungstenite::Message::Text(msg.into()))
                        .is_err()
                    {
                        break;
                    }
                }
            });
        }
    });

    (handle, clients, shutdown)
}

/// Broadcast a serialized event to all connected WebSocket clients, pruning dead senders.
fn broadcast(clients: &EventBroadcast, event: &DevEvent) {
    if let Ok(json) = serde_json::to_string(event) {
        let mut lock = clients.lock().unwrap();
        lock.retain(|tx| tx.send(json.clone()).is_ok());
    }
}

/// Run `tsx dev` with optional JSON event emission, watch mode, and WebSocket server.
pub fn dev(json_events: bool, watch: bool, ws_port: Option<u16>) -> CommandResult {
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err("dev", e.to_string()),
    };

    // Start WebSocket broadcast server if requested.
    let mut ws_handle: Option<std::thread::JoinHandle<()>> = None;
    let mut ws_shutdown: Option<Arc<AtomicBool>> = None;
    let ws_clients: Option<EventBroadcast> = ws_port.map(|port| {
        let (handle, clients, shutdown) = start_ws_server(port);
        ws_handle = Some(handle);
        ws_shutdown = Some(shutdown);
        clients
    });

    let emit_event = |event: &DevEvent| {
        if json_events {
            event.emit();
        }
        if let Some(ref clients) = ws_clients {
            broadcast(clients, event);
        }
    };

    emit_event(&DevEvent::new(
        EventType::Started,
        Some(serde_json::json!({
            "project_root": root.to_string_lossy(),
            "tsx_version": env!("CARGO_PKG_VERSION"),
            "json_events": json_events,
            "watch": watch,
            "ws_port": ws_port,
        })),
    ));

    // Spawn the watch thread if --watch is set.
    let watch_handle: Option<std::thread::JoinHandle<()>> = if watch {
        use notify::{recommended_watcher, EventKind, RecursiveMode, Watcher};
        use std::sync::mpsc;

        let (tx, rx) = mpsc::channel();
        let src_dir = root.join("src");
        let src_dir_clone = src_dir.clone();

        let watcher_handle = std::thread::spawn(move || {
            let Ok(mut watcher) = recommended_watcher(move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            }) else {
                return;
            };

            if watcher.watch(&src_dir_clone, RecursiveMode::Recursive).is_err() {
                return;
            }

            // Block until the channel closes (main thread exits).
            for fs_event in rx {
                let path_str = fs_event
                    .paths
                    .first()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let event_type = match fs_event.kind {
                    EventKind::Create(_) => EventType::FileAdded,
                    EventKind::Remove(_) => EventType::FileDeleted,
                    EventKind::Modify(_) => EventType::FileChanged,
                    _ => continue,
                };

                let dev_event = DevEvent::new(
                    event_type,
                    Some(serde_json::json!({ "path": path_str })),
                );
                dev_event.emit(); // always emit watch events to stdout
            }
        });

        Some(watcher_handle)
    } else {
        None
    };

    // Spawn the underlying dev server (Vite / TanStack Start).
    let mut child = match std::process::Command::new("npm")
        .args(["run", "dev"])
        .current_dir(&root)
        .stdout(if json_events {
            std::process::Stdio::piped()
        } else {
            std::process::Stdio::inherit()
        })
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            emit_event(&DevEvent::new(
                EventType::Error,
                Some(serde_json::json!({ "message": e.to_string() })),
            ));
            return CommandResult::err("dev", format!("Failed to start dev server: {}", e));
        }
    };

    if json_events {
        // Stream stdout from the child and translate known patterns into events.
        if let Some(stdout) = child.stdout.take() {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);

            for line in reader.lines().filter_map(|l| l.ok()) {
                let line_lower = line.to_lowercase();

                let event_type = if line_lower.contains("ready in")
                    || line_lower.contains("server running")
                {
                    Some(EventType::BuildCompleted)
                } else if line_lower.contains("page reload") || line_lower.contains("hmr update") {
                    Some(EventType::HotReload)
                } else if line_lower.contains("error") || line_lower.contains("failed") {
                    Some(EventType::Error)
                } else if line_lower.contains("build") {
                    Some(EventType::BuildStarted)
                } else {
                    None
                };

                if let Some(et) = event_type {
                    emit_event(&DevEvent::new(
                        et,
                        Some(serde_json::json!({ "message": line })),
                    ));
                }
            }
        }
    }

    let status = child.wait();

    emit_event(&DevEvent::new(
        EventType::Stopped,
        Some(serde_json::json!({
            "exit_code": status.map(|s| s.code()).unwrap_or(None)
        })),
    ));

    // Wait for watch thread to clean up.
    if let Some(handle) = watch_handle {
        drop(handle);
    }

    // Signal and join the WebSocket server thread.
    if let Some(shutdown) = ws_shutdown {
        shutdown.store(true, Ordering::Relaxed);
        // Unblock the accept loop with a dummy connection.
        if let Some(port) = ws_port {
            let _ = std::net::TcpStream::connect(format!("127.0.0.1:{}", port));
        }
    }
    if let Some(handle) = ws_handle {
        let _ = handle.join();
    }

    CommandResult::ok("dev", vec![])
}
