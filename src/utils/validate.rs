/// Input validation helpers shared across all generate commands.

/// Validate that `name` is a safe TypeScript/JavaScript identifier.
///
/// Rules:
/// - Non-empty
/// - Starts with a Unicode letter or `_`
/// - Contains only letters, digits, `_`, or `-` (kebab names are also allowed
///   since heck converts them before generation)
/// - Does not start or end with `-`
/// - Is not a reserved JS keyword
pub fn validate_identifier(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("name cannot be empty".to_string());
    }

    let first = name.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return Err(format!(
            "name '{}' must start with a letter or underscore",
            name
        ));
    }

    if name.ends_with('-') {
        return Err(format!("name '{}' cannot end with a hyphen", name));
    }

    let bad: Vec<char> = name
        .chars()
        .filter(|c| !c.is_alphanumeric() && *c != '_' && *c != '-')
        .collect();
    if !bad.is_empty() {
        return Err(format!(
            "name '{}' contains invalid characters: {:?}",
            name, bad
        ));
    }

    // Reserved JS/TS keywords that would produce broken output
    const RESERVED: &[&str] = &[
        "break", "case", "catch", "class", "const", "continue", "debugger",
        "default", "delete", "do", "else", "export", "extends", "false",
        "finally", "for", "function", "if", "import", "in", "instanceof",
        "let", "new", "null", "return", "static", "super", "switch", "this",
        "throw", "true", "try", "typeof", "var", "void", "while", "with",
        "yield", "enum", "implements", "interface", "package", "private",
        "protected", "public", "type", "namespace",
    ];

    if RESERVED.contains(&name.to_lowercase().as_str()) {
        return Err(format!(
            "name '{}' is a reserved JavaScript/TypeScript keyword",
            name
        ));
    }

    Ok(())
}

/// Validate that a route path is safe.
///
/// Rules:
/// - Non-empty
/// - Starts with `/`
/// - No `..` traversal segments
/// - No consecutive slashes (`//`)
pub fn validate_route_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("route path cannot be empty".to_string());
    }
    if !path.starts_with('/') {
        return Err(format!("route path '{}' must start with '/'", path));
    }
    if path.contains("..") {
        return Err(format!("route path '{}' contains path traversal '..'", path));
    }
    if path.contains("//") {
        return Err(format!("route path '{}' contains consecutive slashes", path));
    }
    Ok(())
}

/// Validate that a fields list is non-empty.
pub fn validate_fields_non_empty(fields: &[impl Sized]) -> Result<(), String> {
    if fields.is_empty() {
        Err("fields list cannot be empty".to_string())
    } else {
        Ok(())
    }
}

/// Validate that each field name is a safe identifier.
pub fn validate_field_names(fields: &[crate::schemas::FieldSchema]) -> Result<(), String> {
    for field in fields {
        validate_identifier(&field.name).map_err(|e| format!("field: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names() {
        assert!(validate_identifier("user").is_ok());
        assert!(validate_identifier("_private").is_ok());
        assert!(validate_identifier("my-resource").is_ok());
        assert!(validate_identifier("CamelCase").is_ok());
    }

    #[test]
    fn empty_name_rejected() {
        assert!(validate_identifier("").is_err());
    }

    #[test]
    fn starts_with_digit_rejected() {
        assert!(validate_identifier("1user").is_err());
    }

    #[test]
    fn trailing_hyphen_rejected() {
        assert!(validate_identifier("user-").is_err());
    }

    #[test]
    fn special_chars_rejected() {
        assert!(validate_identifier("user name").is_err());
        assert!(validate_identifier("user@name").is_err());
    }

    #[test]
    fn reserved_keyword_rejected() {
        assert!(validate_identifier("return").is_err());
        assert!(validate_identifier("class").is_err());
        assert!(validate_identifier("type").is_err());
    }

    #[test]
    fn valid_route_paths() {
        assert!(validate_route_path("/dashboard").is_ok());
        assert!(validate_route_path("/users/$id").is_ok());
    }

    #[test]
    fn route_path_no_slash_rejected() {
        assert!(validate_route_path("dashboard").is_err());
    }

    #[test]
    fn route_path_traversal_rejected() {
        assert!(validate_route_path("/../etc/passwd").is_err());
    }

    #[test]
    fn route_path_double_slash_rejected() {
        assert!(validate_route_path("//dashboard").is_err());
    }
}
