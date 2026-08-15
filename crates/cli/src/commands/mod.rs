// One module per CLI command, flat — no grouping umbrella. If it isn't a
// folder here, it's either not a command, or dead code that was removed.

pub mod add_auth;
pub mod add_auth_guard;
pub mod add_migration;
pub mod adb;
pub mod analyze;
pub mod atoms;
pub mod audit;
pub mod auth;
pub mod batch;
pub mod build;
pub mod codegen;
pub mod completions;
pub mod config;
pub mod context;
pub mod create;
pub mod dev;
pub mod docs;
pub mod doctor;
pub mod env;
pub mod flutter;
pub mod fmt;
pub mod framework;
pub mod generate;
pub mod init;
pub mod inspect;
pub mod lint_template;
pub mod list;
pub mod mcp;
pub mod migrate;
pub mod package;
pub mod path;
pub mod pattern;
pub mod pkg;
pub mod plan;
pub mod plugin;
pub mod port;
pub mod publish;
pub mod query;
pub mod registry;
pub mod repl;
pub mod replay;
pub mod run;
pub mod self_update;
pub mod snapshot;
pub mod stack;
pub mod subscribe;
pub mod template;
pub mod test_run;
pub mod tui;
pub mod upgrade;
pub mod watch;

// --- Flat re-exports so cross-module callers (batch/exec.rs, dispatch.rs) can
// reach individual generators without depending on the `generate` folder's
// internal layout. ---
pub use generate::add_feature;
pub use generate::add_form;
pub use generate::add_page;
pub use generate::add_query;
pub use generate::add_schema;
pub use generate::add_seed;
pub use generate::add_server_fn;
pub use generate::add_table;

pub use query::ask;
pub use query::describe;
pub use query::explain;
pub use query::how;
pub use query::where_cmd;
