//! Rust source model used by `rust_to_ts`'s parser and generator.

#[derive(Debug, Clone)]
pub(super) enum RustItem {
    Struct(RustStruct),
    Enum(RustEnum),
}

#[derive(Debug, Clone)]
pub(super) struct RustStruct {
    pub name: String,
    pub fields: Vec<RustField>,
}

#[derive(Debug, Clone)]
pub(super) struct RustField {
    pub name: String,
    pub rust_type: String,
    pub serde_rename: Option<String>,
    pub is_optional: bool, // Option<T> or #[serde(skip_serializing_if = "Option::is_none")]
    pub has_default: bool, // #[serde(default)]
    pub is_flatten: bool,  // #[serde(flatten)]
}

#[derive(Debug, Clone)]
pub(super) struct RustEnum {
    pub name: String,
    pub variants: Vec<RustVariant>,
    pub is_unit_only: bool,
}

#[derive(Debug, Clone)]
pub(super) struct RustVariant {
    pub name: String,
    pub serde_rename: Option<String>,
    pub payload: Option<String>, // for tuple/struct variants
}
