// --- Subdirectory modules ---
pub mod generate;
pub mod manage;
pub mod ops;
pub mod package;
pub mod query;

// --- Single-action commands, one folder each ---
pub mod docs;
pub mod fmt;
pub mod tui;
pub mod watch;

// --- Multi-action command groups, folderized (types.rs/utils.rs/one file per action) ---
pub mod batch;
pub mod codegen;
pub mod framework;
pub mod pattern;
pub mod pkg;
pub mod registry;
pub mod stack;

// --- Re-exports at flat paths (used by main.rs and batch.rs) ---
pub use generate::add_feature;
pub use generate::add_form;
pub use generate::add_page;
pub use generate::add_query;
pub use generate::add_schema;
pub use generate::add_seed;
pub use generate::add_server_fn;
pub use generate::add_table;

pub use manage::add_auth;
pub use manage::add_auth_guard;
pub use manage::add_migration;
pub use manage::auth;
pub use manage::create;
pub use manage::dev;
pub use manage::init;
pub use manage::plugin;
pub use manage::self_update;
pub use manage::upgrade;

pub use ops::analyze;
pub use ops::atoms;
pub use ops::audit;
pub use ops::build;
pub use ops::config;
pub use ops::context;
pub use ops::env;
pub use ops::lint_template;
pub use ops::migrate;
pub use ops::replay;
pub use ops::repl;
pub use ops::snapshot;
pub use ops::test_run;
pub use ops::generate as fw_generate;
pub use ops::inspect;
pub use ops::list;
pub use ops::plan;
pub use ops::publish;
pub use ops::run;
pub use ops::mcp;
pub use ops::subscribe;
pub use ops::template;

pub use query::ask;
pub use query::describe;
pub use query::explain;
pub use query::how;
pub use query::where_cmd;
