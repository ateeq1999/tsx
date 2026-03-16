// --- Subdirectory modules ---
pub mod generate;
pub mod manage;
pub mod ops;
pub mod query;

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
pub use manage::create;
pub use manage::dev;
pub use manage::framework_cmd;
pub use manage::init;
pub use manage::plugin;
pub use manage::upgrade;

pub use ops::batch;
pub use ops::generate as fw_generate;
pub use ops::inspect;
pub use ops::run;
pub use ops::list;
pub use ops::publish;
pub use ops::registry;
pub use ops::subscribe;

pub use query::ask;
pub use query::describe;
pub use query::explain;
pub use query::how;
pub use query::where_cmd;
