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
    /// Start the development server
    Dev {
        /// Emit structured JSON events to stdout instead of raw terminal output
        #[arg(long)]
        json_events: bool,
        /// Watch for template and source changes and regenerate automatically
        #[arg(long)]
        watch: bool,
        /// Start WebSocket server for real-time IDE events on this port
        #[arg(long, value_name = "PORT")]
        ws_port: Option<u16>,
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
    /// Scaffold a project from a framework starter recipe
    Create {
        /// Framework slug to create from (e.g., tanstack-start)
        #[arg(long)]
        from: String,
        /// Starter ID to use (default: basic)
        #[arg(long)]
        starter: Option<String>,
    },
    /// Manage framework packages (author tools)
    Framework {
        #[command(subcommand)]
        action: FrameworkCmd,
    },
    /// Inspect current project state
    Inspect,
    /// Execute multiple commands in one invocation
    Batch {
        /// JSON payload with array of commands
        #[arg(long)]
        json: Option<String>,
        /// Stream each result as newline-delimited JSON as it completes
        #[arg(long)]
        stream: bool,
    },
    /// Start an SSE event subscription server for external tool integration
    Subscribe {
        /// Port to listen on (default: 7331)
        #[arg(long, default_value = "7331")]
        port: u16,
    },
    /// Show framework overview and available knowledge cost map (agent entry point)
    Describe {
        /// Framework slug to describe (e.g., tanstack-start)
        #[arg(long)]
        framework: String,
        /// Return a specific knowledge section (overview, concepts, patterns, faq, decisions)
        #[arg(long)]
        section: Option<String>,
    },
    /// Answer questions about a framework
    Ask {
        /// The question to ask
        #[arg(long)]
        question: String,
        /// Framework to query (optional)
        #[arg(long)]
        framework: Option<String>,
        /// Response depth: brief (~50 tokens), default (~150 tokens), full (~400 tokens)
        #[arg(long, default_value = "default")]
        depth: String,
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
    /// Check or pin atom template versions
    Upgrade {
        #[command(subcommand)]
        target: Upgrade,
    },
    /// Manage installed template plugins
    Plugin {
        #[command(subcommand)]
        action: Plugin,
    },
    /// Publish or validate a framework registry
    Publish {
        #[command(subcommand)]
        action: Publish,
    },
    /// Discover and manage community framework registries
    Registry {
        #[command(subcommand)]
        action: RegistryCmd,
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
    /// Run a framework-defined generator by ID
    Fw {
        /// Generator ID (e.g., add-schema, add-page)
        #[arg(long)]
        id: String,
        /// Framework slug (auto-detected from package.json if omitted)
        #[arg(long)]
        fw: Option<String>,
        /// Generator arguments as JSON
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

#[derive(Subcommand)]
enum RegistryCmd {
    /// Search npm for community tsx-framework-* packages
    Search {
        /// Search query (leave empty to list all tsx-framework packages)
        #[arg(long, default_value = "")]
        query: String,
    },
    /// Install a community registry from an npm package
    Install {
        /// npm package name
        #[arg(long)]
        package: String,
    },
    /// List community registries installed in this project
    List,
    /// Generate a static HTML registry catalog website
    Website {
        /// Output directory for the generated site (default: registry-site/)
        #[arg(long, default_value = "registry-site")]
        output: String,
    },
}

#[derive(Subcommand)]
enum Publish {
    /// Validate and publish a registry.json file (print to stdout or write to --output)
    Registry {
        /// Path to the registry.json file to publish
        #[arg(long, value_name = "PATH")]
        registry: String,
        /// Write the published package to this file instead of stdout
        #[arg(long, value_name = "PATH")]
        output: Option<String>,
    },
    /// List registries installed in .tsx/frameworks/
    List,
}

#[derive(Subcommand)]
enum Plugin {
    /// List installed plugins
    List,
    /// Install a plugin from a local directory or npm
    Install {
        /// Local path or npm package name
        #[arg(long)]
        source: String,
    },
    /// Remove an installed plugin
    Remove {
        /// npm package name of the plugin
        #[arg(long)]
        package: String,
    },
}

#[derive(Subcommand)]
enum Upgrade {
    /// Check atom versions and pin to current (default: pin)
    Atoms {
        /// Only report version status without writing to package.json
        #[arg(long)]
        check: bool,
    },
}

#[derive(Subcommand)]
enum FrameworkCmd {
    /// Scaffold a new framework package directory
    Init {
        /// Framework name/slug
        #[arg(long)]
        name: String,
    },
    /// Validate a framework package directory
    Validate {
        /// Path to the framework package (default: current directory)
        #[arg(long)]
        path: Option<String>,
    },
    /// Render a framework template with test data
    Preview {
        /// Path to the template file
        #[arg(long)]
        template: String,
        /// JSON context data for rendering
        #[arg(long)]
        data: Option<String>,
    },
    /// Install a framework package from a local directory
    Add {
        /// Local path to the framework package directory
        #[arg(long)]
        source: String,
    },
    /// List installed framework packages
    List,
    /// Publish a framework package to npm as @tsx-pkg/<id>
    Publish {
        /// Path to the framework package (default: current directory)
        #[arg(long)]
        path: Option<String>,
        /// Validate and show what would be published without running npm publish
        #[arg(long)]
        dry_run: bool,
    },
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
        Command::Dev {
            json_events,
            watch,
            ws_port,
        } => {
            use tsx::commands::dev;
            let result = dev::dev(json_events, watch, ws_port);
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
                use tsx::schemas::AddTableArgs;

                let args: AddTableArgs = serde_json::from_str(&json.unwrap()).unwrap();
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
            Generate::Fw { id, fw, json } => {
                use tsx::commands::fw_generate;
                let result = fw_generate::generate(id, fw, json, cli.overwrite, cli.dry_run, cli.verbose);
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
        Command::Create { from, starter } => {
            use tsx::commands::create;
            let result = create::create(from, starter, cli.dry_run, cli.verbose);
            result.print();
        }
        Command::Framework { action } => match action {
            FrameworkCmd::Init { name } => {
                use tsx::commands::framework_cmd;
                let result = framework_cmd::framework_init(name, cli.verbose);
                result.print();
            }
            FrameworkCmd::Validate { path } => {
                use tsx::commands::framework_cmd;
                let result = framework_cmd::framework_validate(path, cli.verbose);
                result.print();
            }
            FrameworkCmd::Preview { template, data } => {
                use tsx::commands::framework_cmd;
                let result = framework_cmd::framework_preview(template, data, cli.verbose);
                result.print();
            }
            FrameworkCmd::Add { source } => {
                use tsx::commands::framework_cmd;
                let result = framework_cmd::framework_add(source, cli.verbose);
                result.print();
            }
            FrameworkCmd::List => {
                use tsx::commands::framework_cmd;
                let result = framework_cmd::framework_list(cli.verbose);
                result.print();
            }
            FrameworkCmd::Publish { path, dry_run } => {
                use tsx::commands::framework_cmd;
                let result = framework_cmd::framework_publish(path, dry_run, cli.verbose);
                result.print();
            }
        },
        Command::Inspect => {
            use tsx::commands::inspect;
            let result = inspect::inspect(cli.verbose);
            result.print();
        }
        Command::Batch { json, stream } => {
            use tsx::commands::batch;
            use tsx::json::payload::BatchPayload;

            let payload: BatchPayload = serde_json::from_str(&json.unwrap()).unwrap();
            let result = batch::batch(payload, cli.overwrite, cli.dry_run, cli.verbose, stream);
            result.print();
        }
        Command::Subscribe { port } => {
            use tsx::commands::subscribe;
            let result = subscribe::subscribe(port, cli.verbose);
            result.print();
        }
        Command::Describe { framework, section } => {
            use tsx::commands::query::describe;
            let result = describe::describe(framework, section, cli.verbose);
            result.print();
        }
        Command::Ask {
            question,
            framework,
            depth,
        } => {
            use tsx::commands::query::ask;
            // Auto-detect framework from package.json when not specified
            let resolved_framework = framework.or_else(|| {
                let root = std::env::current_dir().ok()?;
                tsx::framework::detect::detect_framework(&root)
            });
            let result = ask::ask(question, resolved_framework, depth, cli.verbose);
            result.print();
        }
        Command::Where { thing, framework } => {
            use tsx::commands::query::where_cmd;
            let result = where_cmd::where_cmd(thing, framework, cli.verbose);
            result.print();
        }
        Command::How {
            integration,
            framework,
        } => {
            use tsx::commands::query::how;
            let result = how::how(integration, framework, cli.verbose);
            result.print();
        }
        Command::Explain { topic } => {
            use tsx::commands::query::explain;
            let result = explain::explain(topic, cli.verbose);
            result.print();
        }
        Command::Upgrade { target } => match target {
            Upgrade::Atoms { check } => {
                use tsx::commands::upgrade;
                let result = upgrade::upgrade(check, cli.verbose);
                result.print();
            }
        },
        Command::Publish { action } => match action {
            Publish::Registry { registry, output } => {
                use tsx::commands::publish;
                let result = publish::publish(registry, output, cli.verbose);
                result.print();
            }
            Publish::List => {
                use tsx::commands::publish;
                let result = publish::publish_list(cli.verbose);
                result.print();
            }
        },
        Command::Registry { action } => match action {
            RegistryCmd::Search { query } => {
                use tsx::commands::registry;
                let result = registry::registry_search(query, cli.verbose);
                result.print();
            }
            RegistryCmd::Install { package } => {
                use tsx::commands::registry;
                let result = registry::registry_install(package, cli.verbose);
                result.print();
            }
            RegistryCmd::List => {
                use tsx::commands::registry;
                let result = registry::registry_list(cli.verbose);
                result.print();
            }
            RegistryCmd::Website { output } => {
                use tsx::commands::registry;
                let result = registry::registry_website(output, cli.verbose);
                result.print();
            }
        },
        Command::Plugin { action } => match action {
            Plugin::List => {
                use tsx::commands::plugin;
                let result = plugin::plugin_list(cli.verbose);
                result.print();
            }
            Plugin::Install { source } => {
                use tsx::commands::plugin;
                let result = plugin::plugin_install(source, cli.verbose);
                result.print();
            }
            Plugin::Remove { package } => {
                use tsx::commands::plugin;
                let result = plugin::plugin_remove(package, cli.verbose);
                result.print();
            }
        },
    }
}
