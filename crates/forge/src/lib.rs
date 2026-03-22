//! **forge** — 4-tier code generation engine for the tsx Framework Protocol.
//!
//! ## Overview
//!
//! `forge` is built on [Tera](https://keats.github.io/tera/) and extends it with:
//!
//! - **4-tier template hierarchy** — `Atom → Molecule → Layout → Feature`
//!   (classified automatically from template paths)
//! - **Import hoisting** — templates call `{{ "import x from 'x'" | collect_import }}`
//!   and `{{ render_imports() }}` at the top of the file to emit a deduplicated import block.
//! - **Token-budget metadata** — knowledge `.md` files carry `token_estimate` in frontmatter
//!   so agents can request exactly as much context as they need.
//! - **Framework package loading** — load templates from disk, embedded bytes, or npm packages.
//!
//! ## Quick Start
//!
//! ```rust
//! use forge::{Engine, ForgeContext};
//!
//! let mut engine = Engine::new();
//! engine.add_raw("hello.jinja", "Hello {{ name | pascal_case }}!").unwrap();
//!
//! let ctx = ForgeContext::new().insert("name", "world");
//! let out = engine.render("hello.jinja", &ctx).unwrap();
//! assert_eq!(out, "Hello World!");
//! ```

pub mod ast;
pub mod collector;
pub mod compose;
pub mod context;
pub mod engine;
pub mod error;
pub mod filters;
pub mod manifest;
pub mod metadata;
pub mod plan;
pub mod preprocessor;
pub mod provide;
pub mod slots;
pub mod tier;
pub mod validate;

pub use ast::{
    ForgeFile, StyleConfig, QuoteStyle, Render,
    BodyNode, TableNode, TableKind, ColumnNode, ImportNode, RawNode,
    pg_table, sqlite_table, uuid_pk, text_col, int_col, bool_col,
    timestamp_col, real_col, raw, to_snake_case, to_pascal_case,
};
pub use compose::{ExtendsGraph, check_extends_cycle, extract_extends_path};
pub use context::ForgeContext;
pub use engine::Engine;
pub use error::ForgeError;
pub use manifest::{
    GeneratedFile, ManifestDependencies, MultiOutput, OutputPath,
    TemplateManifest, interpolate_path, load_manifest, render_multi,
};
pub use metadata::{parse as parse_frontmatter, FrontMatter};
pub use plan::{GeneratorPlan, OverwritePolicy, PlanResult, PlanError};
pub use preprocessor::preprocess;
pub use tier::Tier;
pub use validate::{
    ValidationResult, extract_schema, validate_input, validate_template_input,
};
