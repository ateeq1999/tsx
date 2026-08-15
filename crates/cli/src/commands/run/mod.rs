//! `tsx run` — universal generator dispatcher: resolve by id or command name,
//! validate input against its schema, apply defaults, then execute.

mod list;
mod run;
mod types;
mod utils;

pub use list::run_list;
pub use run::run;
