//! **tsx-codegen** — standalone code generation library (Section B).
//!
//! Provides four transformation pipelines:
//!
//! | Source | Target | Entry point |
//! |---|---|---|
//! | Rust structs/enums | Zod schemas + TypeScript types | `rust_to_zod::convert` |
//! | Rust structs/enums | TypeScript interfaces only | `rust_to_ts::convert` |
//! | OpenAPI 3.x JSON | Zod schemas | `openapi_to_zod::convert` |
//! | Drizzle schema file | Zod insert/select schemas | `drizzle_to_zod::convert` |

pub mod drizzle_to_zod;
pub mod openapi_to_zod;
pub mod rust_to_ts;
pub mod rust_to_zod;
pub mod types;

pub use types::{CodegenInput, CodegenOutput, CodegenError};
