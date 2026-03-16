use std::fmt;

#[derive(Debug)]
pub enum ForgeError {
    TemplateNotFound(String),
    RenderError(String),
    LoadError(String),
    FrontmatterError(String),
}

impl fmt::Display for ForgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ForgeError::TemplateNotFound(n) => write!(f, "Template not found: {n}"),
            ForgeError::RenderError(e) => write!(f, "Render error: {e}"),
            ForgeError::LoadError(e) => write!(f, "Load error: {e}"),
            ForgeError::FrontmatterError(e) => write!(f, "Frontmatter parse error: {e}"),
        }
    }
}

impl std::error::Error for ForgeError {}

impl From<tera::Error> for ForgeError {
    fn from(e: tera::Error) -> Self {
        ForgeError::RenderError(e.to_string())
    }
}
