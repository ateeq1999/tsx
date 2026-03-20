use clap::{CommandFactory, Parser, Subcommand};

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

    /// Show a unified diff of what would change without writing files
    #[arg(long, global = true)]
    diff: bool,

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
        /// Comma-separated tsx packages to activate in the new project (e.g. tanstack-start,drizzle-pg,better-auth)
        #[arg(long)]
        stack: Option<String>,
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
        /// List kind: templates, generators, components, or frameworks.
        /// Omit for agent mode: returns all registry generators with full metadata.
        #[arg(long)]
        kind: Option<String>,
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
        /// Plan mode: resolve all commands and show what would be created without executing
        #[arg(long)]
        plan: bool,
    },
    /// Start an SSE event subscription server for external tool integration
    Subscribe {
        /// Port to listen on (default: 7331)
        #[arg(long, default_value = "7331")]
        port: u16,
    },
    /// Show framework overview or generator details (agent entry point)
    Describe {
        /// Framework slug (e.g., tanstack-start) or generator command-id (e.g., add:schema).
        /// Can be passed as a positional arg or via --framework.
        #[arg(value_name = "TARGET")]
        target: Option<String>,
        /// Framework slug (alternative to positional arg)
        #[arg(long)]
        framework: Option<String>,
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
    /// Log in to the tsx package registry with an API key
    Login {
        /// API key from registry-web /account/api-keys
        #[arg(long)]
        token: String,
        /// Registry URL (default: https://tsx-tsnv.onrender.com)
        #[arg(long)]
        registry: Option<String>,
    },
    /// Log out and remove stored registry credentials
    Logout,
    /// Show the currently logged-in user and registry
    Whoami,
    /// Install and inspect packages from the tsx registry
    Pkg {
        #[command(subcommand)]
        action: PkgCmd,
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
    /// Manage the project stack profile (.tsx/stack.json)
    Stack {
        #[command(subcommand)]
        action: StackCmd,
    },
    /// Translate natural-language goals into a concrete command sequence
    Plan {
        /// JSON array of goals, e.g. '[{"goal":"add a users schema"}]'
        #[arg(long)]
        json: Option<String>,
    },
    /// Print agent-ready context: active stack, available commands, and usage summary
    Context,
    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for: bash, zsh, fish, powershell, elvish
        #[arg(value_name = "SHELL")]
        shell: String,
    },
    /// Run diagnostic checks on the current project and environment
    Doctor,
    /// Lint .forge / .jinja template files for common errors
    LintTemplate {
        /// Path to a template file or directory (default: .tsx/templates/ or templates/)
        #[arg(value_name = "PATH")]
        path: Option<String>,
    },
    /// Snapshot testing for generators — save & diff outputs
    Snapshot {
        #[command(subcommand)]
        action: SnapshotCmd,
    },
    /// Manage user-defined generator patterns
    Pattern {
        #[command(subcommand)]
        action: PatternCmd,
    },
    /// Generate TypeScript interfaces and Zod schemas from Rust/OpenAPI/Drizzle sources
    Codegen {
        #[command(subcommand)]
        target: CodegenCmd,
    },
    /// Launch the ratatui terminal dashboard (registry browser, doctor, stack editor)
    Tui {
        /// Which view to open: browser (default), doctor, stack
        #[arg(long, value_name = "VIEW", default_value = "browser")]
        view: String,
    },
    /// Manage global tsx configuration (~/.tsx/config.json)
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
    },
    /// Validate or diff .env files
    Env {
        #[command(subcommand)]
        action: EnvCmd,
    },
    /// Run drizzle-kit database migrations
    Migrate {
        /// Only run `drizzle-kit generate` (skip apply)
        #[arg(long)]
        generate_only: bool,
        /// Only apply pending migrations (skip generate)
        #[arg(long)]
        apply_only: bool,
    },
    /// Detect and run the project's build command
    Build {
        /// Emit structured JSON events (agent mode)
        #[arg(long)]
        json_events: bool,
    },
    /// Run the project's test suite (vitest / jest / playwright)
    Test {
        /// Run only tests matching this pattern
        #[arg(long, value_name = "PATTERN")]
        filter: Option<String>,
        /// Watch mode — re-run on file changes
        #[arg(long)]
        watch: bool,
        /// Emit structured JSON test results
        #[arg(long)]
        json: bool,
    },
    /// Run npm audit and format vulnerabilities
    Audit {
        /// Minimum severity to report: critical, high, moderate, low
        #[arg(long, value_name = "LEVEL")]
        severity: Option<String>,
        /// Run `npm audit fix`
        #[arg(long)]
        fix: bool,
    },
    /// Interactive goal-driven REPL
    Repl {
        /// One-shot goal (agent mode — skips interactive loop)
        #[arg(long, value_name = "GOAL")]
        goal: Option<String>,
        /// Execute proposed commands without prompting
        #[arg(long)]
        execute: bool,
    },
    /// Queryable catalog of atoms and molecules for the active framework
    Atoms {
        #[command(subcommand)]
        action: AtomsCmd,
    },
    /// Scan project structure and report health/convention issues
    Analyze {
        /// Auto-apply safe fixes where possible
        #[arg(long)]
        fix: bool,
        /// Emit structured JSON suitable for CI pipelines
        #[arg(long)]
        report: bool,
    },
    /// Record and replay generation sessions
    Replay {
        #[command(subcommand)]
        action: ReplayCmd,
    },
    /// Run any installed framework generator by id or command name
    Run {
        /// Generator id (e.g. `add-schema`) or command name (e.g. `add:schema`).
        /// Omit to list all available generators.
        #[arg(value_name = "ID")]
        id: Option<String>,
        /// Framework slug — auto-detected from package.json when omitted
        #[arg(long)]
        fw: Option<String>,
        /// Generator input as a JSON object
        #[arg(long)]
        json: Option<String>,
        /// List all available generators (optionally filtered by --fw)
        #[arg(long)]
        list: bool,
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
    /// Check all installed packages for newer versions and reinstall if available
    Update,
    /// Show version, description, commands, and integration info for a package
    Info {
        /// npm package name (e.g. @tsx-pkg/drizzle-pg)
        #[arg(value_name = "PACKAGE")]
        package: String,
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
    /// Check for a newer tsx binary and self-update from GitHub Releases
    Cli {
        /// Only print the latest version without downloading
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
        /// Upload to a hosted registry instead of npm (e.g. https://registry.tsx.dev)
        #[arg(long)]
        registry: Option<String>,
        /// Bearer token for the hosted registry (or set TSX_REGISTRY_API_KEY)
        #[arg(long)]
        api_key: Option<String>,
    },
}

#[derive(Subcommand)]
enum StackCmd {
    /// Create or overwrite .tsx/stack.json (auto-detects from package.json when no flags given)
    Init {
        /// Override detected language (typescript, python, rust, go)
        #[arg(long)]
        lang: Option<String>,
        /// Comma-separated list of tsx packages to activate (e.g. tanstack-start,drizzle-pg)
        #[arg(long)]
        packages: Option<String>,
    },
    /// Print the current stack profile
    Show,
    /// Add a package to the active stack
    Add {
        /// Package name (e.g. better-auth, shadcn)
        #[arg(value_name = "PACKAGE")]
        package: String,
    },
    /// Remove a package from the active stack
    Remove {
        /// Package name (without version)
        #[arg(value_name = "PACKAGE")]
        package: String,
    },
    /// Detect the stack from project files and print suggestions
    Detect {
        /// Automatically install detected packages via `tsx registry install`
        #[arg(long)]
        install: bool,
    },
}

#[derive(Subcommand)]
enum PkgCmd {
    /// Install a package from the tsx registry into .tsx/packages/<name>/
    Install {
        /// Package name (e.g. auth-form or @scope/pkg)
        #[arg(value_name = "NAME")]
        name: String,
        /// Pin to a specific version (default: latest)
        #[arg(long)]
        version: Option<String>,
        /// Install into this directory instead of .tsx/packages/
        #[arg(long, value_name = "DIR")]
        target: Option<String>,
    },
    /// Show metadata, versions, and download stats for a registry package
    Info {
        /// Package name
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Upgrade an installed package to its latest version
    Upgrade {
        /// Package name (e.g. auth-form or @scope/pkg)
        #[arg(value_name = "NAME")]
        name: String,
        /// Install into this directory instead of .tsx/packages/
        #[arg(long, value_name = "DIR")]
        target: Option<String>,
    },
    /// Publish a package directory to the tsx registry
    Publish {
        /// Path to the package directory (default: current directory)
        #[arg(long, value_name = "DIR")]
        path: Option<String>,
        /// Override the package name from manifest.json
        #[arg(long)]
        name: Option<String>,
        /// Override the version from manifest.json
        #[arg(long)]
        version: Option<String>,
        /// Validate and show what would be published without uploading
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum SnapshotCmd {
    /// Run all generators with fixture inputs and save their output as snapshots
    Update {
        /// Only update snapshots for this generator id
        #[arg(long, value_name = "ID")]
        generator: Option<String>,
    },
    /// Re-run generators and diff against saved snapshots
    Diff {
        /// Only diff snapshots for this generator id
        #[arg(long, value_name = "ID")]
        generator: Option<String>,
    },
    /// Accept current output as the new baseline (alias for update)
    Accept {
        /// Only accept snapshots for this generator id
        #[arg(long, value_name = "ID")]
        generator: Option<String>,
    },
    /// List all registered snapshot fixtures
    List,
    /// Register a new fixture input for a generator
    Add {
        /// Generator id (e.g. add-schema)
        #[arg(long, value_name = "ID")]
        generator: String,
        /// Fixture name (e.g. users)
        #[arg(long, value_name = "NAME")]
        fixture: String,
        /// JSON input for the generator
        #[arg(long, value_name = "JSON")]
        input: Option<String>,
    },
}

#[derive(Subcommand)]
enum PatternCmd {
    /// Register a new generator pattern from a template file
    Add {
        /// Pattern id / name (e.g. "add-service")
        #[arg(long)]
        name: String,
        /// Human-readable description
        #[arg(long)]
        description: Option<String>,
        /// Path to the .forge template file
        #[arg(long, value_name = "FILE")]
        template: Option<String>,
        /// Argument spec: "name:string, entity:string, methods:string[]"
        #[arg(long, value_name = "SPEC")]
        args: Option<String>,
    },
    /// Start recording file changes as a reusable pattern
    Record {
        /// Pattern name (required when starting a recording)
        #[arg(long)]
        name: Option<String>,
        /// Stop the active recording session and save the pattern
        #[arg(long)]
        stop: bool,
    },
    /// List all local patterns in .tsx/patterns/
    List,
    /// Show details of a specific pattern
    Show {
        /// Pattern id
        #[arg(value_name = "ID")]
        id: String,
    },
    /// Remove a pattern
    Remove {
        /// Pattern id
        #[arg(value_name = "ID")]
        id: String,
    },
    /// Publish a pattern to the tsx registry
    Share {
        /// Pattern id
        #[arg(long)]
        name: String,
        /// Version to publish
        #[arg(long)]
        version: Option<String>,
    },
}

#[derive(Subcommand)]
enum CodegenCmd {
    /// Parse Rust structs/enums and emit TypeScript interfaces + Zod schemas
    RustToTs {
        /// Path to the Rust source file (default: crates/shared/src/lib.rs)
        #[arg(long, value_name = "FILE")]
        input: Option<String>,
        /// Output TypeScript file (default: generated/<stem>.ts)
        #[arg(long, value_name = "FILE")]
        out: Option<String>,
        /// Watch the input file and regenerate on change
        #[arg(long)]
        watch: bool,
    },
    /// Convert an OpenAPI spec to Zod schemas
    OpenapiToZod {
        /// URL or path to the OpenAPI spec
        #[arg(long, value_name = "SPEC")]
        spec: String,
        /// Output TypeScript file
        #[arg(long, value_name = "FILE")]
        out: Option<String>,
    },
    /// Auto-run drizzle-zod across all schema files
    DrizzleToZod,
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Get a single config value
    Get {
        #[arg(value_name = "KEY")]
        key: String,
    },
    /// Set a config value
    Set {
        #[arg(value_name = "KEY")]
        key: String,
        #[arg(value_name = "VALUE")]
        value: String,
    },
    /// List all config values
    List,
    /// Reset a key (or all keys) to defaults
    Reset {
        /// Key to reset (omit to reset everything)
        #[arg(value_name = "KEY")]
        key: Option<String>,
    },
}

#[derive(Subcommand)]
enum EnvCmd {
    /// Validate .env against .env.schema
    Check {
        /// Path to schema file (default: .env.schema)
        #[arg(long, value_name = "FILE")]
        schema: Option<String>,
        /// Path to .env file (default: .env)
        #[arg(long, value_name = "FILE")]
        env: Option<String>,
    },
    /// Show vars in .env.example missing from .env
    Diff {
        /// Path to example file (default: .env.example)
        #[arg(long, value_name = "FILE")]
        example: Option<String>,
        /// Path to .env file (default: .env)
        #[arg(long, value_name = "FILE")]
        env: Option<String>,
    },
}

#[derive(Subcommand)]
enum AtomsCmd {
    /// List available atoms and molecules, optionally filtered by category
    List {
        /// Filter by category (e.g. drizzle, form, zod, query)
        #[arg(long, value_name = "CATEGORY")]
        category: Option<String>,
    },
    /// Show the raw template source for an atom or molecule
    Preview {
        /// Atom id (e.g. drizzle/column, form/field_input)
        #[arg(value_name = "ID")]
        id: String,
    },
}

#[derive(Subcommand)]
enum ReplayCmd {
    /// Start recording a generation session
    Record {
        /// Path to write the session JSON file (default: .tsx/sessions/session-<ts>.json)
        #[arg(long, value_name = "FILE")]
        out: Option<String>,
        /// Stop the active recording and write the session file
        #[arg(long)]
        stop: bool,
    },
    /// Replay a previously recorded session file
    Run {
        /// Path to the session JSON file
        #[arg(value_name = "FILE")]
        file: String,
        /// Show what would be created without writing any files
        #[arg(long)]
        dry_run: bool,
    },
    /// List recorded session files in .tsx/sessions/
    List,
}

/// Parse a `--json` argument, printing a structured error and returning `None` on failure.
/// This replaces the old `serde_json::from_str(&json.unwrap()).unwrap()` pattern that panics.
fn parse_json_input<T: serde::de::DeserializeOwned>(
    json: Option<String>,
    json_input: Option<&str>,
    cmd_name: &str,
) -> Option<T> {
    use tsx::json::error::ErrorResponse;
    use tsx::json::response::ResponseEnvelope;

    let raw = match json.as_deref().or(json_input) {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => {
            let err =
                ErrorResponse::validation(&format!("'{}' requires --json <JSON>", cmd_name));
            ResponseEnvelope::error(cmd_name, err, 0).print();
            return None;
        }
    };
    match serde_json::from_str(&raw) {
        Ok(v) => Some(v),
        Err(e) => {
            let err = ErrorResponse::validation(&format!(
                "'{}' received invalid JSON — {}",
                cmd_name, e
            ));
            ResponseEnvelope::error(cmd_name, err, 0).print();
            None
        }
    }
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
        Command::Init { name, stack } => {
            use tsx::commands::init;
            let result = init::init(name, stack);
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
                let ji = json_input.as_deref();
                if let Some(args) = parse_json_input::<AddFeatureArgs>(json, ji, "generate feature") {
                    add_feature::add_feature(args, cli.overwrite, cli.dry_run, cli.diff).print();
                }
            }
            Generate::Schema { json } => {
                use tsx::commands::add_schema;
                use tsx::schemas::AddSchemaArgs;
                let ji = json_input.as_deref();
                if let Some(args) = parse_json_input::<AddSchemaArgs>(json, ji, "generate schema") {
                    add_schema::add_schema(args, cli.overwrite, cli.dry_run, cli.diff).print();
                }
            }
            Generate::ServerFn { json } => {
                use tsx::commands::add_server_fn;
                use tsx::schemas::AddServerFnArgs;
                let ji = json_input.as_deref();
                if let Some(args) = parse_json_input::<AddServerFnArgs>(json, ji, "generate server-fn") {
                    add_server_fn::add_server_fn(args, cli.overwrite, cli.dry_run, cli.diff).print();
                }
            }
            Generate::Query { json } => {
                use tsx::commands::add_query;
                use tsx::schemas::AddQueryArgs;
                let ji = json_input.as_deref();
                if let Some(args) = parse_json_input::<AddQueryArgs>(json, ji, "generate query") {
                    add_query::add_query(args, cli.overwrite, cli.dry_run, cli.diff).print();
                }
            }
            Generate::Form { json } => {
                use tsx::commands::add_form;
                use tsx::schemas::AddFormArgs;
                let ji = json_input.as_deref();
                if let Some(args) = parse_json_input::<AddFormArgs>(json, ji, "generate form") {
                    add_form::add_form(args, cli.overwrite, cli.dry_run, cli.diff).print();
                }
            }
            Generate::Table { json } => {
                use tsx::commands::add_table;
                use tsx::schemas::AddTableArgs;
                let ji = json_input.as_deref();
                if let Some(args) = parse_json_input::<AddTableArgs>(json, ji, "generate table") {
                    add_table::add_table(args, cli.overwrite, cli.dry_run, cli.diff).print();
                }
            }
            Generate::Page { json } => {
                use tsx::commands::add_page;
                use tsx::schemas::AddPageArgs;
                let ji = json_input.as_deref();
                if let Some(args) = parse_json_input::<AddPageArgs>(json, ji, "generate page") {
                    add_page::add_page(args, cli.overwrite, cli.dry_run, cli.diff).print();
                }
            }
            Generate::Seed { json } => {
                use tsx::commands::add_seed;
                use tsx::schemas::AddSeedArgs;
                let ji = json_input.as_deref();
                if let Some(args) = parse_json_input::<AddSeedArgs>(json, ji, "generate seed") {
                    add_seed::add_seed(args, cli.overwrite, cli.dry_run, cli.diff).print();
                }
            }
            Generate::Fw { id, fw, json } => {
                // Delegate to the universal `run` dispatcher.
                use tsx::commands::run;
                let merged_json = json.or_else(|| json_input.clone());
                run::run(id, fw, merged_json, cli.overwrite, cli.dry_run, cli.verbose).print();
            }
        },
        Command::Add { integration } => match integration {
            Add::Auth { json } => {
                use tsx::commands::add_auth;
                use tsx::schemas::AddAuthArgs;
                let ji = json_input.as_deref();
                if let Some(args) = parse_json_input::<AddAuthArgs>(json, ji, "add auth") {
                    add_auth::add_auth(args, cli.overwrite, cli.dry_run, cli.diff).print();
                }
            }
            Add::AuthGuard { json } => {
                use tsx::commands::add_auth_guard;
                use tsx::schemas::AddAuthGuardArgs;
                let ji = json_input.as_deref();
                if let Some(args) = parse_json_input::<AddAuthGuardArgs>(json, ji, "add auth-guard") {
                    add_auth_guard::add_auth_guard(args, cli.overwrite, cli.dry_run, cli.diff).print();
                }
            }
            Add::Migration => {
                use tsx::commands::add_migration;
                add_migration::add_migration().print();
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
            FrameworkCmd::Publish { path, dry_run, registry, api_key } => {
                use tsx::commands::framework_cmd;
                let result = framework_cmd::framework_publish(path, dry_run, registry, api_key, cli.verbose);
                result.print();
            }
        },
        Command::Inspect => {
            use tsx::commands::inspect;
            let result = inspect::inspect(cli.verbose);
            result.print();
        }
        Command::Batch { json, stream, plan } => {
            use tsx::commands::batch;
            use tsx::json::payload::BatchPayload;
            let ji = json_input.as_deref();
            if let Some(payload) = parse_json_input::<BatchPayload>(json, ji, "batch") {
                if plan || cli.dry_run {
                    batch::batch_plan(payload, cli.verbose).print();
                } else {
                    batch::batch(payload, cli.overwrite, false, cli.verbose, stream).print();
                }
            }
        }
        Command::Subscribe { port } => {
            use tsx::commands::subscribe;
            let result = subscribe::subscribe(port, cli.verbose);
            result.print();
        }
        Command::Describe { target, framework, section } => {
            use tsx::commands::query::describe;
            // Resolve: positional arg takes precedence over --framework flag
            let resolved = target.or(framework);
            let result = describe::describe(resolved, section, cli.verbose);
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
            Upgrade::Cli { check } => {
                use tsx::commands::self_update;
                let result = self_update::self_update(check);
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
            RegistryCmd::Update => {
                use tsx::commands::registry;
                let result = registry::registry_update(cli.verbose);
                result.print();
            }
            RegistryCmd::Info { package } => {
                use tsx::commands::registry;
                let result = registry::registry_info(package, cli.verbose);
                result.print();
            }
        },
        Command::Login { token, registry } => {
            use tsx::commands::auth;
            auth::login(token, registry).print();
        }
        Command::Logout => {
            use tsx::commands::auth;
            auth::logout().print();
        }
        Command::Whoami => {
            use tsx::commands::auth;
            auth::whoami().print();
        }
        Command::Pkg { action } => match action {
            PkgCmd::Install { name, version, target } => {
                use tsx::commands::pkg;
                pkg::pkg_install(name, version, target).print();
            }
            PkgCmd::Info { name } => {
                use tsx::commands::pkg;
                pkg::pkg_info(name).print();
            }
            PkgCmd::Upgrade { name, target } => {
                use tsx::commands::pkg;
                pkg::pkg_upgrade(name, target).print();
            }
            PkgCmd::Publish { path, name, version, dry_run } => {
                use tsx::commands::pkg;
                pkg::pkg_publish(path, name, version, dry_run).print();
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
        Command::Stack { action } => match action {
            StackCmd::Init { lang, packages } => {
                use tsx::commands::stack;
                stack::stack_init(lang, packages, cli.dry_run, cli.verbose).print();
            }
            StackCmd::Show => {
                use tsx::commands::stack;
                stack::stack_show(cli.verbose).print();
            }
            StackCmd::Add { package } => {
                use tsx::commands::stack;
                stack::stack_add(package, cli.verbose).print();
            }
            StackCmd::Remove { package } => {
                use tsx::commands::stack;
                stack::stack_remove(package, cli.verbose).print();
            }
            StackCmd::Detect { install } => {
                use tsx::commands::stack;
                stack::stack_detect(install, cli.verbose).print();
            }
        },
        Command::Plan { json } => {
            use tsx::commands::plan;
            use tsx::commands::plan::PlanGoal;
            let ji = json_input.as_deref();
            if let Some(goals) = parse_json_input::<Vec<PlanGoal>>(json, ji, "plan") {
                plan::plan(goals, cli.verbose).print();
            }
        }
        Command::Context => {
            use tsx::commands::context;
            context::context(cli.verbose).print();
        }
        Command::Completions { shell } => {
            use clap_complete::generate;
            use std::io::Write;
            use tsx::commands::manage::completions;

            match completions::resolve_shell(&shell) {
                Err(_) => completions::unknown_shell_error(&shell).print(),
                Ok(sh) => {
                    let mut cmd = Cli::command();
                    let mut buf = Vec::<u8>::new();
                    generate(sh, &mut cmd, "tsx", &mut buf);
                    std::io::stdout().write_all(&buf).ok();
                }
            }
        }
        Command::Doctor => {
            use tsx::commands::manage::doctor;
            doctor::doctor().print();
        }
        Command::LintTemplate { path } => {
            use tsx::commands::lint_template;
            lint_template::lint_template(path, cli.verbose).print();
        }
        Command::Snapshot { action } => match action {
            SnapshotCmd::Update { generator } => {
                use tsx::commands::snapshot;
                snapshot::snapshot_update(generator, cli.verbose).print();
            }
            SnapshotCmd::Diff { generator } => {
                use tsx::commands::snapshot;
                snapshot::snapshot_diff(generator, cli.verbose).print();
            }
            SnapshotCmd::Accept { generator } => {
                use tsx::commands::snapshot;
                snapshot::snapshot_accept(generator, cli.verbose).print();
            }
            SnapshotCmd::List => {
                use tsx::commands::snapshot;
                snapshot::snapshot_list(cli.verbose).print();
            }
            SnapshotCmd::Add { generator, fixture, input } => {
                use tsx::commands::snapshot;
                snapshot::snapshot_add(generator, fixture, input, cli.verbose).print();
            }
        },
        Command::Pattern { action } => match action {
            PatternCmd::Add { name, description, template, args } => {
                use tsx::commands::pattern;
                pattern::pattern_add(name, description, template, args, cli.verbose).print();
            }
            PatternCmd::Record { name, stop } => {
                use tsx::commands::pattern;
                if stop {
                    pattern::pattern_record_stop(cli.verbose).print();
                } else {
                    match name {
                        Some(n) => pattern::pattern_record_start(n, cli.verbose).print(),
                        None => {
                            use tsx::json::error::{ErrorCode, ErrorResponse};
                            use tsx::json::response::ResponseEnvelope;
                            ResponseEnvelope::error(
                                "pattern record",
                                ErrorResponse::new(ErrorCode::ValidationError, "--name is required when starting a recording"),
                                0,
                            ).print();
                        }
                    }
                }
            }
            PatternCmd::List => {
                use tsx::commands::pattern;
                pattern::pattern_list(cli.verbose).print();
            }
            PatternCmd::Show { id } => {
                use tsx::commands::pattern;
                pattern::pattern_show(id, cli.verbose).print();
            }
            PatternCmd::Remove { id } => {
                use tsx::commands::pattern;
                pattern::pattern_remove(id, cli.verbose).print();
            }
            PatternCmd::Share { name, version } => {
                use tsx::commands::pattern;
                pattern::pattern_share(name, version, cli.verbose).print();
            }
        },
        Command::Codegen { target } => match target {
            CodegenCmd::RustToTs { input, out, watch } => {
                use tsx::commands::codegen;
                codegen::codegen_rust_to_ts(input, out, watch, cli.verbose).print();
            }
            CodegenCmd::OpenapiToZod { spec, out } => {
                use tsx::commands::codegen;
                codegen::codegen_openapi_to_zod(spec, out, cli.verbose).print();
            }
            CodegenCmd::DrizzleToZod => {
                use tsx::commands::codegen;
                codegen::codegen_drizzle_to_zod(cli.verbose).print();
            }
        },
        Command::Run { id, fw, json, list } => {
            use tsx::commands::run;
            if list || id.is_none() {
                run::run_list(fw, cli.verbose).print();
            } else {
                let merged_json = json.or_else(|| json_input.clone());
                run::run(
                    id.unwrap(),
                    fw,
                    merged_json,
                    cli.overwrite,
                    cli.dry_run,
                    cli.verbose,
                )
                .print();
            }
        }
        Command::Tui { view } => {
            use tsx_tui::{run, TuiView, BrowserItem};
            let tui_view = TuiView::from_str(&view);
            // Provide a minimal set of items if no registry data is available
            let items = vec![
                BrowserItem::new("tanstack-start", "Full-stack React framework with SSR and file-based routing"),
                BrowserItem::new("drizzle-pg", "PostgreSQL schema and query generator for Drizzle ORM"),
                BrowserItem::new("better-auth", "Authentication integration for TanStack Start"),
                BrowserItem::new("shadcn", "shadcn/ui component generator"),
            ];
            if let Err(e) = run(tui_view, items) {
                eprintln!("TUI error: {}", e);
                std::process::exit(1);
            }
        }
        Command::Config { action } => {
            use tsx::commands::config;
            match action {
                ConfigCmd::Get { key } => config::config_get(key).print(),
                ConfigCmd::Set { key, value } => config::config_set(key, value).print(),
                ConfigCmd::List => config::config_list().print(),
                ConfigCmd::Reset { key } => config::config_reset(key).print(),
            }
        }
        Command::Env { action } => {
            use tsx::commands::env;
            match action {
                EnvCmd::Check { schema, env: env_path } => env::env_check(schema, env_path).print(),
                EnvCmd::Diff { example, env: env_path } => env::env_diff(example, env_path).print(),
            }
        }
        Command::Migrate { generate_only, apply_only } => {
            use tsx::commands::migrate;
            migrate::migrate(generate_only, apply_only, cli.dry_run, cli.verbose).print();
        }
        Command::Build { json_events } => {
            use tsx::commands::build;
            build::build(json_events, cli.verbose).print();
        }
        Command::Test { filter, watch, json } => {
            use tsx::commands::test_run;
            test_run::test_run(filter, watch, json, cli.verbose).print();
        }
        Command::Audit { severity, fix } => {
            use tsx::commands::audit;
            audit::audit(severity, fix, cli.verbose).print();
        }
        Command::Repl { goal, execute } => {
            use tsx::commands::repl;
            repl::repl(goal, execute, cli.verbose).print();
        }
        Command::Atoms { action } => {
            use tsx::commands::atoms;
            match action {
                AtomsCmd::List { category } => atoms::atoms_list(category, cli.verbose).print(),
                AtomsCmd::Preview { id } => atoms::atoms_preview(id, cli.verbose).print(),
            }
        }
        Command::Analyze { fix, report } => {
            use tsx::commands::analyze;
            analyze::analyze(fix, report, cli.verbose).print();
        }
        Command::Replay { action } => {
            use tsx::commands::replay;
            match action {
                ReplayCmd::Record { out, stop } => {
                    if stop {
                        replay::replay_record_stop(cli.verbose).print();
                    } else {
                        replay::replay_record_start(out, cli.verbose).print();
                    }
                }
                ReplayCmd::Run { file, dry_run } => {
                    replay::replay_run(file, dry_run, cli.verbose).print();
                }
                ReplayCmd::List => replay::replay_list(cli.verbose).print(),
            }
        }
    }
}
