//! Helpers shared by two or more `registry` actions (search, install, update, info).

pub(super) fn iso_now() -> String {
    unix_secs_to_iso(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    )
}

/// Convert a Unix timestamp (seconds) to an ISO-8601 UTC string with correct
/// leap-year and month-length handling.
fn unix_secs_to_iso(total_secs: u64) -> String {
    let sec = (total_secs % 60) as u32;
    let min = ((total_secs / 60) % 60) as u32;
    let hour = ((total_secs / 3600) % 24) as u32;

    let mut days = total_secs / 86400;
    let mut year = 1970u32;
    loop {
        let in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < in_year { break; }
        days -= in_year;
        year += 1;
    }
    let month_days: [u64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u32;
    for &md in &month_days {
        if days < md { break; }
        days -= md;
        month += 1;
    }
    let day = days as u32 + 1;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hour, min, sec)
}

fn is_leap_year(y: u32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

/// Return the configured registry base URL.
/// Checks `TSX_REGISTRY_URL` env var; falls back to `None` (→ use npm/unpkg).
pub(super) fn registry_url() -> Option<String> {
    std::env::var("TSX_REGISTRY_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim_end_matches('/').to_string())
}

/// Minimal percent-encoding for URL query parameters.
pub(super) fn urlencoding(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                vec![c]
            }
            ' ' => vec!['+'],
            c => {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf);
                bytes
                    .bytes()
                    .flat_map(|b| {
                        let hex: Vec<char> =
                            format!("%{:02X}", b).chars().collect();
                        hex
                    })
                    .collect()
            }
        })
        .collect()
}
