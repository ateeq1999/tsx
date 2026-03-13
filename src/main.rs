use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tsx", version, about = "TanStack Start code generation CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Overwrite existing files without prompting
    #[arg(long, global = true)]
    overwrite: bool,

    /// Print what would be written without creating files
    #[arg(long, global = true)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Bootstrap a new TanStack Start project
    Init {
        /// Project name
        #[arg(long)]
        name: Option<String>,
    },
    /// Scaffold a complete CRUD feature module
    #[command(name = "add:feature")]
    AddFeature {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Generate a Drizzle schema table definition
    #[command(name = "add:schema")]
    AddSchema {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Generate a typed server function
    #[command(name = "add:server-fn")]
    AddServerFn {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Generate a TanStack Query hook
    #[command(name = "add:query")]
    AddQuery {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Generate a TanStack Form component
    #[command(name = "add:form")]
    AddForm {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Generate a TanStack Table component
    #[command(name = "add:table")]
    AddTable {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Add a new route page
    #[command(name = "add:page")]
    AddPage {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Configure Better Auth
    #[command(name = "add:auth")]
    AddAuth {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Wrap a route with a session guard
    #[command(name = "add:auth-guard")]
    AddAuthGuard {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Run drizzle-kit generate + migrate
    #[command(name = "add:migration")]
    AddMigration,
    /// Generate a database seed file
    #[command(name = "add:seed")]
    AddSeed {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    if cli.dry_run {
        println!("Dry run mode - no files will be written");
    }

    match cli.command {
        Command::Init { name } => {
            use tsx::commands::init;
            let result = init::init(name);
            result.print();
        }
        Command::AddFeature { json } => {
            use tsx::commands::add_feature;
            use tsx::schemas::AddFeatureArgs;

            let args: AddFeatureArgs = serde_json::from_str(&json.unwrap()).unwrap();
            let result = add_feature::add_feature(args, cli.overwrite, cli.dry_run);
            result.print();
        }
        Command::AddSchema { json } => {
            use tsx::commands::add_schema;
            use tsx::schemas::AddSchemaArgs;

            let args: AddSchemaArgs = serde_json::from_str(&json.unwrap()).unwrap();
            let result = add_schema::add_schema(args, cli.overwrite, cli.dry_run);
            result.print();
        }
        Command::AddServerFn { json } => {
            use tsx::commands::add_server_fn;
            use tsx::schemas::AddServerFnArgs;

            let args: AddServerFnArgs = serde_json::from_str(&json.unwrap()).unwrap();
            let result = add_server_fn::add_server_fn(args, cli.overwrite, cli.dry_run);
            result.print();
        }
        Command::AddQuery { json } => {
            use tsx::commands::add_query;
            use tsx::schemas::AddQueryArgs;

            let args: AddQueryArgs = serde_json::from_str(&json.unwrap()).unwrap();
            let result = add_query::add_query(args, cli.overwrite, cli.dry_run);
            result.print();
        }
        Command::AddForm { json } => {
            use tsx::commands::add_form;
            use tsx::schemas::AddFormArgs;

            let args: AddFormArgs = serde_json::from_str(&json.unwrap()).unwrap();
            let result = add_form::add_form(args, cli.overwrite, cli.dry_run);
            result.print();
        }
        Command::AddTable { json } => {
            use tsx::commands::add_table;
            use tsx::schemas::AddFormArgs;

            let args: AddFormArgs = serde_json::from_str(&json.unwrap()).unwrap();
            let result = add_table::add_table(args, cli.overwrite, cli.dry_run);
            result.print();
        }
        Command::AddPage { json } => {
            use tsx::commands::add_page;
            use tsx::schemas::AddPageArgs;

            let args: AddPageArgs = serde_json::from_str(&json.unwrap()).unwrap();
            let result = add_page::add_page(args, cli.overwrite, cli.dry_run);
            result.print();
        }
        Command::AddAuth { json } => {
            use tsx::commands::add_auth;
            use tsx::schemas::AddAuthArgs;

            let args: AddAuthArgs = serde_json::from_str(&json.unwrap()).unwrap();
            let result = add_auth::add_auth(args, cli.overwrite, cli.dry_run);
            result.print();
        }
        Command::AddAuthGuard { json } => {
            use tsx::commands::add_auth_guard;
            use tsx::schemas::AddAuthGuardArgs;

            let args: AddAuthGuardArgs = serde_json::from_str(&json.unwrap()).unwrap();
            let result = add_auth_guard::add_auth_guard(args, cli.overwrite, cli.dry_run);
            result.print();
        }
        Command::AddMigration => {
            use tsx::commands::add_migration;
            let result = add_migration::add_migration();
            result.print();
        }
        Command::AddSeed { json } => {
            use tsx::commands::add_seed;
            use tsx::schemas::AddSeedArgs;

            let args: AddSeedArgs = serde_json::from_str(&json.unwrap()).unwrap();
            let result = add_seed::add_seed(args, cli.overwrite, cli.dry_run);
            result.print();
        }
    }
}
