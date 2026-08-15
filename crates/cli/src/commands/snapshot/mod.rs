//! `tsx snapshot` — snapshot testing for generators (E3).
//!
//! Runs generators with fixture inputs, saves their output, and diffs on subsequent runs
//! to catch silent regressions in template changes.
//!
//! ## Directory layout
//!
//! ```text
//! .tsx/snapshots/
//!   <generator-id>/
//!     <fixture-name>.json        — input arguments (fixture)
//!     <fixture-name>.output/     — expected output files
//!       schema.ts
//!       index.ts
//!       ...
//! ```
//!
//! ## Commands
//! - `tsx snapshot update` — run all generators, save/overwrite snapshots
//! - `tsx snapshot diff`   — run generators, show diff vs saved snapshots
//! - `tsx snapshot accept` — alias for `update` (after intentional change)
//! - `tsx snapshot list`   — list all registered fixtures

mod accept;
mod add;
mod diff;
mod list;
mod runner;
mod types;
mod update;

pub use accept::snapshot_accept;
pub use add::snapshot_add;
pub use diff::snapshot_diff;
pub use list::snapshot_list;
pub use update::snapshot_update;

pub use types::SnapshotFixture;
