use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::output::CommandResult;

/// A subscriber slot — each connected SSE client has a sender end.
type SseClients = Arc<Mutex<Vec<std::sync::mpsc::Sender<String>>>>;

/// Broadcast an SSE event to all connected clients, removing dead ones.
pub fn broadcast_sse(clients: &SseClients, event_type: &str, data: &str) {
    let msg = format!("event: {}\ndata: {}\n\n", event_type, data);
    let mut lock = clients.lock().unwrap();
    lock.retain(|tx| tx.send(msg.clone()).is_ok());
}

/// `tsx subscribe --port <PORT>`
///
/// Starts a minimal HTTP/1.1 SSE server on `127.0.0.1:<port>`.
/// External tools (IDE plugins, dashboards) connect to `GET /events` and
/// receive a `text/event-stream` response.  The server also exposes a
/// `POST /emit` endpoint that accepts `{ "event": "...", "data": "..." }`
/// JSON and broadcasts it to all connected clients.
pub fn subscribe(port: u16, verbose: bool) -> CommandResult {
    let start = Instant::now();
    let clients: SseClients = Arc::new(Mutex::new(Vec::new()));

    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            return CommandResult::err(
                "subscribe",
                format!("Cannot bind SSE server to {}: {}", addr, e),
            );
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    if verbose {
        eprintln!(
            "[tsx subscribe] SSE server started on http://{} in {}ms",
            addr, duration_ms
        );
        eprintln!("[tsx subscribe] GET  http://{}/events — connect to receive events", addr);
        eprintln!("[tsx subscribe] POST http://{}/emit   — broadcast an event", addr);
    } else {
        eprintln!("[tsx subscribe] SSE server on http://{}", addr);
    }

    // Broadcast a startup event to any already-connected clients (none yet).
    let startup = serde_json::json!({
        "message": "tsx subscribe SSE server started",
        "port": port,
        "tsx_version": env!("CARGO_PKG_VERSION"),
    });
    broadcast_sse(
        &clients,
        "started",
        &startup.to_string(),
    );

    for stream in listener.incoming().flatten() {
        let clients_ref = Arc::clone(&clients);
        std::thread::spawn(move || {
            handle_sse_connection(stream, clients_ref);
        });
    }

    CommandResult::ok("subscribe", vec![])
}

fn handle_sse_connection(mut stream: std::net::TcpStream, clients: SseClients) {
    use std::io::BufRead;

    let mut reader = std::io::BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });

    // Read the HTTP request line.
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }

    // Drain the rest of the headers.
    let mut content_length: usize = 0;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() {
            return;
        }
        let h = header.trim().to_lowercase();
        if h.is_empty() {
            break;
        }
        if h.starts_with("content-length:") {
            content_length = h
                .trim_start_matches("content-length:")
                .trim()
                .parse()
                .unwrap_or(0);
        }
    }

    let method_path: Vec<&str> = request_line.splitn(3, ' ').collect();
    if method_path.len() < 2 {
        return;
    }
    let method = method_path[0];
    let path = method_path[1];

    match (method, path) {
        ("GET", "/events") => {
            // SSE connection — keep alive and stream events.
            let response = "HTTP/1.1 200 OK\r\n\
                Content-Type: text/event-stream\r\n\
                Cache-Control: no-cache\r\n\
                Connection: keep-alive\r\n\
                Access-Control-Allow-Origin: *\r\n\
                \r\n";

            if stream.write_all(response.as_bytes()).is_err() {
                return;
            }

            // Send an initial comment to establish the connection.
            let _ = stream.write_all(b": connected\n\n");

            let (tx, rx) = std::sync::mpsc::channel::<String>();
            {
                let mut lock = clients.lock().unwrap();
                lock.push(tx);
            }

            // Forward events until the sender is dropped.
            for msg in rx {
                if stream.write_all(msg.as_bytes()).is_err() {
                    break;
                }
            }
        }

        ("POST", "/emit") => {
            // Read JSON body and broadcast as SSE event.
            let mut body = vec![0u8; content_length.min(65536)];
            let n = std::io::Read::read(&mut reader, &mut body).unwrap_or(0);
            body.truncate(n);

            let ok_response = "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n";
            let err_response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";

            if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(&body) {
                let event_type = parsed
                    .get("event")
                    .and_then(|v| v.as_str())
                    .unwrap_or("message");
                let data = parsed
                    .get("data")
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                broadcast_sse(&clients, event_type, &data);
                let _ = stream.write_all(ok_response.as_bytes());
            } else {
                let _ = stream.write_all(err_response.as_bytes());
            }
        }

        ("GET", "/health") => {
            let connected = clients.lock().map(|l| l.len()).unwrap_or(0);
            let body = serde_json::json!({
                "status": "ok",
                "connected_clients": connected,
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }

        _ => {
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        }
    }
}
