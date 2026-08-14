//! Registry discovery and community sharing for tsx framework registries.
//!
//! `tsx registry search <query>`  — search npm for tsx-framework-* packages
//! `tsx registry install <pkg>`   — install a community registry into .tsx/frameworks/
//! `tsx registry list`            — list installed community registries

mod info;
mod install;
mod list;
mod search;
mod types;
mod update;
mod utils;
mod website;

pub use info::registry_info;
pub use install::registry_install;
pub use list::registry_list;
pub use search::registry_search;
pub use update::registry_update;
pub use website::registry_website;

pub use types::InstalledRegistry;
