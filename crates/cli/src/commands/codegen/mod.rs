//! `tsx codegen` — code generation utilities.
//!
//! Currently supports:
//! - `rust-to-ts`: parse Rust struct/enum definitions and emit TypeScript interfaces + Zod schemas
//! - `openapi-to-zod`: convert an OpenAPI spec to Zod schemas (stub — emits instructions)
//! - `drizzle-to-zod`: run drizzle-zod across schema files (stub — emits instructions)

mod drizzle_to_zod;
mod openapi_to_zod;
mod rust_parser;
mod rust_to_ts;
mod types;

pub use drizzle_to_zod::codegen_drizzle_to_zod;
pub use openapi_to_zod::codegen_openapi_to_zod;
pub use rust_to_ts::codegen_rust_to_ts;
