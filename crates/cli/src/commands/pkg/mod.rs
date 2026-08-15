//! `tsx pkg` — install and publish packages to/from the tsx registry.

mod info;
mod install;
mod publish;
mod types;
mod upgrade;
mod utils;

pub use info::pkg_info;
pub use install::pkg_install;
pub use publish::pkg_publish;
pub use upgrade::pkg_upgrade;
