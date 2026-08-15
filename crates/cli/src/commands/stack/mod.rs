//! `tsx stack` — manage the project stack profile (`.tsx/stack.json`).
//!
//! Not to be confused with `crate::stack`, the `StackProfile`/`UserStack` domain
//! model these handlers read and write.

mod add;
mod detect;
mod init;
mod remove;
mod show;

pub use add::stack_add;
pub use detect::stack_detect;
pub use init::stack_init;
pub use remove::stack_remove;
pub use show::stack_show;
