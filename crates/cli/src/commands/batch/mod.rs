//! `tsx batch` — execute (or plan) multiple commands in one invocation.

mod exec;
mod execute;
mod plan;
mod types;

pub use exec::execute_command_pub;
pub use execute::batch;
pub use plan::batch_plan;
