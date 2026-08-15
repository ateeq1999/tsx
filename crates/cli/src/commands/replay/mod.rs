//! `tsx replay` — record and replay generation sessions (H4).
//!
//! Subcommands:
//! - `tsx replay record --out <file>` — start recording a session
//! - `tsx replay record --stop`       — stop recording and write the session file
//! - `tsx replay run <file>`           — replay a recorded session
//! - `tsx replay list`                 — list recorded session files in .tsx/sessions/

mod list;
mod record;
mod run;
mod types;
mod utils;

pub use list::replay_list;
pub use record::{replay_record_start, replay_record_stop};
pub use run::replay_run;

pub use types::{ReplaySession, ReplayStep};
