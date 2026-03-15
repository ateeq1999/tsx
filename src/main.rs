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

    /// Enable verbose output with additional context
    #[arg(long, global = true)]
    verbose: bool,

    /// Read command payload from stdin as JSON
    #[arg(long, global = true)]
    stdin: bool,

    /// Read command payload from a file
    #[arg(long, global = true, value_name = "PATH")]
    file: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a new TanStack Start project
    Init {
        /// Project name
        #[arg(long)]
        name: Option<String>,
    },
    /// Generate code from templates
    Generate {
        #[command(subcommand)]
        generator: Generate,
    },
    /// Add integrations to project
    Add {
        #[command(subcommand)]
        integration: Add,
    },
    /// List available templates, generators, components, or frameworks
    List {
        /// List kind: templates, generators, components, or frameworks
        #[arg(long)]
        kind: String,
    },
    /// Inspect current project state
    Inspect,
    /// Execute multiple commands in one invocation
    Batch {
        /// JSON payload with array of commands
        #[arg(long)]
        json: Option<String>,
    },
    /// Answer questions about a framework
    Ask {
        /// The question to ask
        #[arg(long)]
        question: String,
        /// Framework to query (optional)
        #[arg(long)]
        framework: Option<String>,
    },
    /// Find where things go in a framework
    Where {
        /// The thing to find (e.g., atom, route, schema)
        #[arg(long)]
        thing: String,
        /// Framework to query (optional)
        #[arg(long)]
        framework: Option<String>,
    },
    /// Get integration steps for a package
    How {
        /// The package/integration (e.g., @tanstack/react-router)
        #[arg(long)]
        integration: String,
        /// Framework to query (optional)
        #[arg(long)]
        framework: Option<String>,
    },
    /// Explain template decisions and conventions
    Explain {
        /// The topic to explain (e.g., atom, feature, schema)
        #[arg(long)]
        topic: String,
    },
}

#[derive(Subcommand)]
enum Generate {
    /// Scaffold a complete CRUD feature module
    Feature {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Generate a Drizzle schema table definition
    Schema {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Generate a typed server function
    ServerFn {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Generate a TanStack Query hook
    Query {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Generate a TanStack Form component
    Form {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Generate a TanStack Table component
    Table {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Add a new route page
    Page {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Generate a database seed file
    Seed {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
}

#[derive(Subcommand)]
enum Add {
    /// Configure Better Auth
    Auth {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Wrap a route with a session guard
    AuthGuard {
        /// JSON payload
        #[arg(long)]
        json: Option<String>,
    },
    /// Run drizzle-kit generate + migrate
    Migration,
}

fn main() {
    use std::io::{self, Read};

    let cli = Cli::parse();

    let json_input = if cli.stdin {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).ok();
        Some(buffer)
    } else if let Some(path) = &cli.file {
        Some(std::fs::read_to_string(path).unwrap_or_default())
    } else {
        None
    };

    if cli.dry_run {
        println!("Dry run mode - no files will be written");
    }

    match cli.command {
        Command::Init { name } => {
            use tsx::commands::init;
            let result = init::init(name);
            result.print();
        }
        Command::Generate { generator } => match generator {
            Generate::Feature { json } => {
                use tsx::commands::add_feature;
                use tsx::schemas::AddFeatureArgs;

                let args: AddFeatureArgs = serde_json::from_str(&json.unwrap()).unwrap();
                let result = add_feature::add_feature(args, cli.overwrite, cli.dry_run);
                result.print();
            }
            Generate::Schema { json } => {
                use tsx::commands::add_schema;
                use tsx::schemas::AddSchemaArgs;

                let args: AddSchemaArgs = serde_json::from_str(&json.unwrap()).unwrap();
                let result = add_schema::add_schema(args, cli.overwrite, cli.dry_run);
                result.print();
            }
            Generate::ServerFn { json } => {
                use tsx::commands::add_server_fn;
                use tsx::schemas::AddServerFnArgs;

                let args: AddServerFnArgs = serde_json::from_str(&json.unwrap()).unwrap();
                let result = add_server_fn::add_server_fn(args, cli.overwrite, cli.dry_run);
                result.print();
            }
            Generate::Query { json } => {
                use tsx::commands::add_query;
                use tsx::schemas::AddQueryArgs;

                let args: AddQueryArgs = serde_json::from_str(&json.unwrap()).unwrap();
                let result = add_query::add_query(args, cli.overwrite, cli.dry_run);
                result.print();
            }
            Generate::Form { json } => {
                use tsx::commands::add_form;
                use tsx::schemas::AddFormArgs;

                let args: AddFormArgs = serde_json::from_str(&json.unwrap()).unwrap();
                let result = add_form::add_form(args, cli.overwrite, cli.dry_run);
                result.print();
            }
            Generate::Table { json } => {
                use tsx::commands::add_table;
                use tsx::schemas::AddFormArgs;

                let args: AddFormArgs = serde_json::from_str(&json.unwrap()).unwrap();
                let result = add_table::add_table(args, cli.overwrite, cli.dry_run);
                result.print();
            }
            Generate::Page { json } => {
                use tsx::commands::add_page;
                use tsx::schemas::AddPageArgs;

                let args: AddPageArgs = serde_json::from_str(&json.unwrap()).unwrap();
                let result = add_page::add_page(args, cli.overwrite, cli.dry_run);
                result.print();
            }
            Generate::Seed { json } => {
                use tsx::commands::add_seed;
                use tsx::schemas::AddSeedArgs;

                let args: AddSeedArgs = serde_json::from_str(&json.unwrap()).unwrap();
                let result = add_seed::add_seed(args, cli.overwrite, cli.dry_run);
                result.print();
            }
        },
        Command::Add { integration } => match integration {
            Add::Auth { json } => {
                use tsx::commands::add_auth;
                use tsx::schemas::AddAuthArgs;

                let args: AddAuthArgs = serde_json::from_str(&json.unwrap()).unwrap();
                let result = add_auth::add_auth(args, cli.overwrite, cli.dry_run);
                result.print();
            }
            Add::AuthGuard { json } => {
                use tsx::commands::add_auth_guard;
                use tsx::schemas::AddAuthGuardArgs;

                let args: AddAuthGuardArgs = serde_json::from_str(&json.unwrap()).unwrap();
                let result = add_auth_guard::add_auth_guard(args, cli.overwrite, cli.dry_run);
                result.print();
            }
            Add::Migration => {
                use tsx::commands::add_migration;
                let result = add_migration::add_migration();
                result.print();
            }
        },
        Command::List { kind } => {
            use tsx::commands::list;
            let result = list::list(kind, cli.verbose);
            result.print();
        }
        Command::Inspect => {
            use tsx::commands::inspect;
            let result = inspect::inspect(cli.verbose);
            result.print();
        }
        Command::Batch { json } => {
            use tsx::commands::batch;
            use tsx::json::payload::BatchPayload;

            let payload: BatchPayload = serde_json::from_str(&json.unwrap()).unwrap();
            let result = batch::batch(payload, cli.overwrite, cli.dry_run, cli.verbose);
            result.print();
        }
        Command::Ask {
            question,
            framework,
        } => {
            use tsx::commands::ask;
            let result = ask::ask(question, framework, cli.verbose);
            result.print();
        }
        Command::Where { thing, framework } => {
            use tsx::commands::where_cmd;
            let result = where_cmd::where_cmd(thing, framework, cli.verbose);
            result.print();
        }
        Command::How {
            integration,
            framework,
        } => {
            use tsx::commands::how;
            let result = how::how(integration, framework, cli.verbose);
            result.print();
        }
        Command::Explain { topic } => {
            use tsx::commands::explain;
            let result = explain::explain(topic, cli.verbose);
            result.print();
        }
    }
}
