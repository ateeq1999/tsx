use std::fmt;

/// Forge error codes.
///
/// | Code | Variant                |
/// |------|------------------------|
/// | F001 | `TemplateNotFound`     |
/// | F002 | `CircularDependency`   |
/// | F003 | `UnclosedBlock`        |
/// | F004 | `SchemaValidation`     |
/// | F005 | `RenderError`          |
/// | F006 | `UnknownVariable`      |
/// | F007 | `OutputConflict`       |
/// | F008 | `LintError`            |
#[derive(Debug)]
pub enum ForgeError {
    /// F001 — Template file could not be found.
    TemplateNotFound(String),
    /// F002 — Circular `@extends` inheritance chain detected.
    CircularDependency(String),
    /// F003 — A block directive (`@if`, `@for`, `@slot`, `@macro`) was never closed with `@end`.
    UnclosedBlock(String),
    /// F004 — `@schema` validation failed; inner `Vec` lists each field-level error.
    SchemaValidation(Vec<String>),
    /// F005 — Tera rendering failed.
    RenderError(String),
    /// F006 — Template references a variable that is not present in the context.
    UnknownVariable(String),
    /// F007 — Output file already exists and the policy forbids overwriting.
    OutputConflict(String),
    /// F008 — Template linting found one or more errors.
    LintError(String),
    /// Internal load / IO error (not assigned a user-facing error code).
    LoadError(String),
    /// Frontmatter parse error (not assigned a user-facing error code).
    FrontmatterError(String),
}

impl ForgeError {
    /// Machine-readable error code string (e.g. `"F004"`).
    pub fn code(&self) -> &'static str {
        match self {
            ForgeError::TemplateNotFound(_)  => "F001",
            ForgeError::CircularDependency(_) => "F002",
            ForgeError::UnclosedBlock(_)     => "F003",
            ForgeError::SchemaValidation(_)  => "F004",
            ForgeError::RenderError(_)       => "F005",
            ForgeError::UnknownVariable(_)   => "F006",
            ForgeError::OutputConflict(_)    => "F007",
            ForgeError::LintError(_)         => "F008",
            ForgeError::LoadError(_)         => "E001",
            ForgeError::FrontmatterError(_)  => "E002",
        }
    }
}

impl fmt::Display for ForgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ForgeError::TemplateNotFound(n)   => write!(f, "[F001] Template not found: {n}"),
            ForgeError::CircularDependency(p) => write!(f, "[F002] Circular @extends dependency: {p}"),
            ForgeError::UnclosedBlock(b)      => write!(f, "[F003] Unclosed block: {b}"),
            ForgeError::SchemaValidation(errs) => {
                write!(f, "[F004] Schema validation failed:\n  - {}", errs.join("\n  - "))
            }
            ForgeError::RenderError(e)        => write!(f, "[F005] Render error: {e}"),
            ForgeError::UnknownVariable(v)    => write!(f, "[F006] Unknown variable: {v}"),
            ForgeError::OutputConflict(p)     => write!(f, "[F007] Output conflict: {p}"),
            ForgeError::LintError(e)          => write!(f, "[F008] Lint error: {e}"),
            ForgeError::LoadError(e)          => write!(f, "[E001] Load error: {e}"),
            ForgeError::FrontmatterError(e)   => write!(f, "[E002] Frontmatter parse error: {e}"),
        }
    }
}

impl std::error::Error for ForgeError {}

impl From<tera::Error> for ForgeError {
    fn from(e: tera::Error) -> Self {
        ForgeError::RenderError(e.to_string())
    }
}
