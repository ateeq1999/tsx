use std::collections::HashMap;

/// Returns embedded (hardcoded) templates as a last-resort fallback.
///
/// Templates have been moved out of the binary into registry packages.
/// The engine now resolves templates from installed packages (PackageStore)
/// before falling back here. This function returns an empty map — kept
/// so the engine API is unchanged and tests that add templates inline still work.
pub fn get_embedded_templates() -> HashMap<&'static str, &'static str> {
    HashMap::new()
}
