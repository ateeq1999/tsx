//! Helpers shared by two or more `pkg` actions (info, install, upgrade).

pub(super) const DEFAULT_REGISTRY_URL: &str = "https://tsx-tsnv.onrender.com";

/// Split `name@version` → `(name, Some(version))`, or `name` → `(name, None)`.
/// Handles scoped packages like `@scope/pkg@1.0.0`:
///   - If the string starts with `@`, the leading `@` belongs to the scope, so
///     we look for `@` after position 1.
pub(super) fn split_name_version(input: &str) -> (String, Option<String>) {
    let at_pos = if input.starts_with('@') {
        input[1..].find('@').map(|i| i + 1)
    } else {
        input.find('@')
    };
    match at_pos {
        Some(i) => (input[..i].to_string(), Some(input[i + 1..].to_string())),
        None    => (input.to_string(), None),
    }
}

pub(super) fn url_encode(s: &str) -> String {
    s.replace('@', "%40").replace('/', "%2F")
}
