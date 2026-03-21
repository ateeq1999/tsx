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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn roundtrip(msg: &serde_json::Value) -> Option<serde_json::Value> {
        let mut buf = Vec::new();
        write_message(&mut buf, msg).unwrap();
        let mut reader = BufReader::new(buf.as_slice());
        read_message(&mut reader)
    }

    #[test]
    fn write_emits_content_length_header() {
        let msg = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "test"});
        let mut buf = Vec::new();
        write_message(&mut buf, &msg).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.starts_with("Content-Length: "));
        assert!(s.contains("\r\n\r\n"));
    }

    #[test]
    fn roundtrip_simple_message() {
        let msg = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"});
        let result = roundtrip(&msg).unwrap();
        assert_eq!(result["method"], "initialize");
        assert_eq!(result["id"], 1);
    }

    #[test]
    fn roundtrip_preserves_nested_params() {
        let msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "textDocument/completion",
            "params": { "textDocument": { "uri": "file:///test.forge" } }
        });
        let result = roundtrip(&msg).unwrap();
        assert_eq!(result["params"]["textDocument"]["uri"], "file:///test.forge");
    }

    #[test]
    fn roundtrip_unicode_content() {
        let msg = serde_json::json!({"text": "こんにちは 🦀"});
        let result = roundtrip(&msg).unwrap();
        assert_eq!(result["text"], "こんにちは 🦀");
    }

    #[test]
    fn read_on_eof_returns_none() {
        let empty: &[u8] = &[];
        let mut reader = BufReader::new(empty);
        assert!(read_message(&mut reader).is_none());
    }

    #[test]
    fn write_then_read_multiple_messages() {
        let mut buf = Vec::new();
        let msgs = [
            serde_json::json!({"id": 1, "method": "a"}),
            serde_json::json!({"id": 2, "method": "b"}),
            serde_json::json!({"id": 3, "method": "c"}),
        ];
        for m in &msgs {
            write_message(&mut buf, m).unwrap();
        }
        let mut reader = BufReader::new(buf.as_slice());
        for expected_id in 1..=3i64 {
            let got = read_message(&mut reader).unwrap();
            assert_eq!(got["id"], expected_id);
        }
    }
}
