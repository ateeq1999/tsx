//! `tsx pattern` — user-defined generator patterns (D1–D4).
//!
//! Patterns let users teach the CLI new generators without writing a full framework package.
//! They are stored at `.tsx/patterns/<id>/pack.json` alongside any `.forge` template files.
//!
//! ## Subcommands
//! - `tsx pattern new <id>` — scaffold a new pack with starter `pack.json` + `main.forge`
//! - `tsx pattern run <id>` — run a pack command (renders templates + injects markers)
//! - `tsx pattern install <source>` — install from local path or `github:user/repo#path@ref`
//! - `tsx pattern lint <id>` — validate pack templates and manifest
//! - `tsx pattern list` — list all local packs
//! - `tsx pattern show <id>` — show pack details
//! - `tsx pattern remove <id>` — remove a pack

mod add;
mod eject;
mod install;
mod lint;
mod list;
mod new;
mod publish;
mod record;
mod remove;
mod run;
mod search;
mod share;
mod show;
mod types;
mod update;
mod utils;

pub use add::pattern_add;
pub use eject::pattern_eject;
pub use install::pattern_install;
pub use lint::pattern_lint;
pub use list::{pattern_list, pattern_list_builtin};
pub use new::pattern_new;
pub use publish::pattern_publish;
pub use record::{pattern_record_start, pattern_record_stop};
pub use remove::pattern_remove;
pub use run::pattern_run;
pub use search::pattern_search;
pub use share::pattern_share;
pub use show::pattern_show;
pub use update::pattern_update;

pub use types::PatternDefinition;
