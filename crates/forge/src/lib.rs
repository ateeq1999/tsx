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

pub mod collector;
pub mod context;
pub mod engine;
pub mod error;
pub mod filters;
pub mod metadata;
pub mod slots;
pub mod tier;

pub use context::ForgeContext;
pub use engine::Engine;
pub use error::ForgeError;
pub use metadata::{parse as parse_frontmatter, FrontMatter};
pub use tier::Tier;
