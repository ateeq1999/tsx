//! LSP stdio transport — Content-Length framed JSON-RPC.

use std::io::{self, BufRead, Write};

/// Read one LSP message from `reader`.
/// Returns `None` on EOF or a fatal I/O error.
pub fn read_message(reader: &mut impl BufRead) -> Option<serde_json::Value> {
    // Parse headers
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => return None,
            _ => {}
        }
        let line = line.trim_end_matches(|c| c == '\r' || c == '\n');
        if line.is_empty() {
            break; // blank line separates headers from body
        }
        if let Some(val) = line.strip_prefix("Content-Length: ") {
            content_length = val.trim().parse().ok();
        }
    }

    let len = content_length?;
    let mut buf = vec![0u8; len];
    let mut filled = 0;
    // Use the underlying reader directly via read_exact-style fill
    loop {
        match reader.fill_buf() {
            Ok(slice) if slice.is_empty() => return None,
            Ok(slice) => {
                let take = (len - filled).min(slice.len());
                buf[filled..filled + take].copy_from_slice(&slice[..take]);
                filled += take;
                reader.consume(take);
                if filled == len {
                    break;
                }
            }
            Err(_) => return None,
        }
    }

    serde_json::from_slice(&buf).ok()
}

/// Write one LSP message to `writer` with proper Content-Length framing.
pub fn write_message(writer: &mut impl Write, msg: &serde_json::Value) -> io::Result<()> {
    let body = serde_json::to_string(msg)?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer.write_all(header.as_bytes())?;
    writer.write_all(body.as_bytes())?;
    writer.flush()
}
