//! `tsx template` — manage forge template bundles.
//!
//! Subcommands:
//! - `tsx template list [--source <global|project|framework>]`
//! - `tsx template info <name>`
//! - `tsx template init <name> [--dest <dir>]`
//! - `tsx template install <source>`
//! - `tsx template uninstall <name>`
//! - `tsx template schema <name> <command>`
//! - `tsx template lint [<path>]`
//! - `tsx template config show`
//! - `tsx template config set <key> <value>`
//! - `tsx template config init`

mod config;
mod info;
mod init;
mod install;
mod lint;
mod list;
mod login;
mod logout;
mod publish;
mod schema;
mod uninstall;

pub use config::{template_config_init, template_config_set, template_config_show};
pub use info::template_info;
pub use init::template_init;
pub use install::template_install;
pub use lint::template_lint;
pub use list::template_list;
pub use login::template_login;
pub use logout::template_logout;
pub use publish::template_publish;
pub use schema::template_schema;
pub use uninstall::template_uninstall;
