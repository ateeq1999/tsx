use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tsx", version, about = "TanStack Start code generation CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Overwrite existing files without prompting
    #[arg(long, global = true)]
    pub overwrite: bool,

    /// Print what would be written without creating files
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Enable verbose output with additional context
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Show a unified diff of what would change without writing files
    #[arg(long, global = true)]
    pub diff: bool,

    /// Read command payload from stdin as JSON
    #[arg(long, global = true)]
    pub stdin: bool,

    /// Read command payload from a file
    #[arg(long, global = true, value_name = "PATH")]
    pub file: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
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
    /// Browse offline documentation from .tsx/knowledge/ in a terminal UI
    Docs {
        /// Additional directories to scan for .md files
        #[arg(value_name = "PATH", num_args = 0..)]
        paths: Vec<String>,
        /// Filter topics by keyword (skips TUI, prints matching titles)
        #[arg(long, value_name = "QUERY")]
        search: Option<String>,
        /// Emit topic list as JSON instead of launching the TUI
        #[arg(long)]
        json: bool,
    },
    /// Format .forge / .jinja template files (normalise indent, quotes, spacing)
    Fmt {
        /// Paths to format — files or directories (default: current directory)
        #[arg(value_name = "PATH", num_args = 0..)]
        paths: Vec<String>,
        /// Check only — exit 1 if any file needs formatting, don't write
        #[arg(long)]
        check: bool,
        /// Spaces per indent level (default: 2)
        #[arg(long, default_value = "2")]
        indent: usize,
        /// Quote style: double or single (default: double)
        #[arg(long, default_value = "double")]
        quotes: String,
    },
    /// Watch files and re-run generators on change
    Watch {
        /// Root directories or files to watch (default: current directory)
        #[arg(value_name = "PATH", num_args = 0..)]
        paths: Vec<String>,
        /// File extensions to watch (default: ts tsx js rs forge jinja)
        #[arg(long, value_name = "EXT", num_args = 0.., value_delimiter = ',')]
        ext: Vec<String>,
        /// Debounce window in milliseconds (default: 300)
        #[arg(long, default_value = "300")]
        debounce: u64,
        /// Command to run on change (default: print changed files)
        #[arg(long, value_name = "CMD")]
        run: Option<String>,
        /// Emit structured JSON events
        #[arg(long)]
        json: bool,
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
    /// Start the Language Server (LSP) for .tsx/ config and .forge template files
    Lsp,
    /// Add directory to PATH (Windows: setx, Unix: export)
    Path {
        /// Directory to add (default: current directory)
        #[arg(value_name = "DIR")]
        directory: Option<String>,
        /// Persist to profile (default: true on Windows, depends on shell on Unix)
        #[arg(long)]
        permanent: bool,
        /// List current PATH entries
        #[arg(long)]
        list: bool,
    },
    /// Android Debug Bridge commands
    Adb {
        #[command(subcommand)]
        action: AdbCmd,
    },
    /// Flutter development commands
    Flutter {
        #[command(subcommand)]
        action: FlutterCmd,
    },
    /// Find and kill processes using a specific port
    Port {
        #[command(subcommand)]
        action: PortCmd,
    },
    /// Run any installed framework generator by id or command name
    Run {
        /// Generator id (e.g. `add-schema`) or command name (e.g. `add:schema`).
        /// Omit to list all available generators.
        #[arg(value_name = "ID")]
        id: Option<String>,
        /// Framework slug — auto-detected from package.json if omitted
        #[arg(long)]
        fw: Option<String>,
        /// Generator input as a JSON object
        #[arg(long)]
        json: Option<String>,
        /// List all available generators (optionally filtered by --fw)
        #[arg(long)]
        list: bool,
    },
    /// Start the MCP (Model Context Protocol) server over stdio
    Mcp,
    /// Manage installed template plugins
    Template {
        #[command(subcommand)]
        action: TemplateCmd,
    },
    /// Install and inspect packages from the tsx registry
    Package {
        #[command(subcommand)]
        action: PackageCmd,
    },
}

#[derive(Subcommand)]
pub enum Generate {
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
pub enum Add {
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
pub enum RegistryCmd {
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
pub enum Publish {
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
pub enum Plugin {
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
pub enum Upgrade {
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
pub enum FrameworkCmd {
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
pub enum StackCmd {
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
pub enum PkgCmd {
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
pub enum SnapshotCmd {
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
pub enum PatternCmd {
    /// Scaffold a new pack with starter pack.json and main.forge
    New {
        /// Pack id (becomes the folder name under .tsx/patterns/)
        #[arg(value_name = "ID")]
        id: String,
        /// Display name
        #[arg(long)]
        name: Option<String>,
        /// Human-readable description
        #[arg(long)]
        description: Option<String>,
        /// Target framework (e.g. tanstack-start)
        #[arg(long)]
        framework: Option<String>,
    },
    /// Run a pack command — render templates and inject markers
    Run {
        /// Pack id
        #[arg(value_name = "ID")]
        id: String,
        /// Named command from pack.json (uses default command if omitted)
        #[arg(long)]
        command: Option<String>,
        /// Template args as key=value pairs (e.g. --arg name=Todo --arg entity=todo)
        #[arg(long = "arg", value_name = "KEY=VALUE")]
        args: Vec<String>,
    },
    /// Install a pack from a local path or github:user/repo[#subpath][@ref]
    Install {
        /// Source: ./path/to/pack or github:user/repo#subpath@ref
        #[arg(value_name = "SOURCE")]
        source: String,
        /// Override the pack id instead of using the id from pack.json
        #[arg(long)]
        id: Option<String>,
    },
    /// Validate a pack's templates and manifest
    Lint {
        /// Pack id
        #[arg(value_name = "ID")]
        id: String,
    },
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
    /// List all local packs in .tsx/patterns/
    List {
        /// Show built-in packs embedded in the binary instead of local packs
        #[arg(long)]
        builtin: bool,
    },
    /// Show details of a specific pack
    Show {
        /// Pack id
        #[arg(value_name = "ID")]
        id: String,
    },
    /// Remove a pack
    Remove {
        /// Pack id
        #[arg(value_name = "ID")]
        id: String,
    },
    /// Publish a pack to the registry
    Publish {
        /// Pack id
        #[arg(value_name = "ID")]
        id: String,
        /// Registry URL (overrides .tsx/config.json)
        #[arg(long)]
        registry: Option<String>,
    },
    /// Search the registry for packs
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
        /// Filter by framework
        #[arg(long)]
        framework: Option<String>,
        /// Registry URL (overrides .tsx/config.json)
        #[arg(long)]
        registry: Option<String>,
    },
    /// Publish a pattern to the tsx registry (legacy)
    Share {
        /// Pattern id
        #[arg(long)]
        name: String,
        /// Version to publish
        #[arg(long)]
        version: Option<String>,
    },
    /// Remove generated files and reverse marker injections from a pack run
    Eject {
        /// Pack id
        #[arg(value_name = "ID")]
        id: String,
    },
    /// Update installed packs from their original source
    Update {
        /// Pack id to update (updates all installed packs if omitted)
        #[arg(value_name = "ID")]
        id: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CodegenCmd {
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
pub enum ConfigCmd {
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
pub enum EnvCmd {
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
pub enum AtomsCmd {
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
pub enum PackageCmd {
    /// Scaffold a new registry package directory
    New {
        /// Package id/slug (e.g. my-framework)
        #[arg(value_name = "ID")]
        id: String,
        /// Output directory (default: <id>)
        #[arg(long, value_name = "DIR")]
        out: Option<String>,
    },
    /// Validate manifest.json and template references
    Validate {
        /// Path to the package directory (default: current directory)
        #[arg(value_name = "DIR")]
        dir: Option<String>,
    },
    /// Create a .tgz tarball from the package directory
    Pack {
        /// Path to the package directory (default: current directory)
        #[arg(value_name = "DIR")]
        dir: Option<String>,
        /// Output tarball path (default: <id>-<version>.tgz)
        #[arg(long, value_name = "FILE")]
        out: Option<String>,
    },
    /// Publish the package to the tsx registry
    Publish {
        /// Path to the package directory (default: current directory)
        #[arg(value_name = "DIR")]
        dir: Option<String>,
        /// Registry URL (default: $TSX_REGISTRY_URL or https://registry.tsx.dev)
        #[arg(long)]
        registry: Option<String>,
        /// Bearer token (default: $TSX_TOKEN)
        #[arg(long)]
        token: Option<String>,
    },
    /// Install a package from the registry
    Install {
        /// Package id
        #[arg(value_name = "ID")]
        id: String,
        /// Registry URL (default: $TSX_REGISTRY_URL or https://registry.tsx.dev)
        #[arg(long)]
        registry: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ReplayCmd {
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

#[derive(Subcommand)]
pub enum TemplateCmd {
    /// List all installed templates with source labels
    List {
        /// Filter by source: global, project, or framework
        #[arg(long, value_name = "SOURCE")]
        source: Option<String>,
    },
    /// Show manifest details for a specific template
    Info {
        /// Template id (e.g. my-forms)
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Scaffold a new template bundle with manifest.json and README
    Init {
        /// Template id / name
        #[arg(value_name = "NAME")]
        name: String,
        /// Output directory (default: ./<name>)
        #[arg(long, value_name = "DIR")]
        dest: Option<String>,
    },
    /// Install a template bundle from a local directory into ~/.tsx/templates/
    Install {
        /// Local path to the template directory
        #[arg(value_name = "SOURCE")]
        source: String,
    },
    /// Remove an installed template
    Uninstall {
        /// Template id to remove
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Return the JSON Schema for a template command (for agent autocomplete)
    Schema {
        /// Template id
        #[arg(value_name = "NAME")]
        name: String,
        /// Command id within the template (e.g. form)
        #[arg(value_name = "COMMAND")]
        command: String,
    },
    /// Lint .forge / .jinja template files in a template bundle
    Lint {
        /// Path to a template file or directory (default: .tsx/templates/ or templates/)
        #[arg(value_name = "PATH")]
        path: Option<String>,
    },
    /// Manage forge template configuration
    Config {
        #[command(subcommand)]
        action: TemplateConfigCmd,
    },
    /// Log in to the forge template registry
    Login {
        /// API key from the registry dashboard
        #[arg(long)]
        token: String,
        /// Registry URL (default: https://registry.tsx.dev)
        #[arg(long)]
        registry: Option<String>,
    },
    /// Log out and remove stored registry credentials
    Logout,
    /// Publish a template bundle to the forge registry
    Publish {
        /// Template id to publish
        #[arg(long, value_name = "NAME")]
        name: String,
        /// Version to publish (e.g. 1.2.0)
        #[arg(long, value_name = "VERSION")]
        version: String,
        /// Path to the template directory (default: current directory)
        #[arg(long, value_name = "DIR")]
        path: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum TemplateConfigCmd {
    /// Print current global and project template config
    Show,
    /// Set a config value in ~/.tsx/config.json
    Set {
        /// Config key (e.g. registry_url, or a command like generate:schema)
        #[arg(value_name = "KEY")]
        key: String,
        /// Value to set
        #[arg(value_name = "VALUE")]
        value: String,
    },
    /// Scaffold ~/.tsx/config.json and .tsx/templates.config.json
    Init {
        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,
    },
}

#[derive(Subcommand)]
pub enum AdbCmd {
    /// Kill the ADB server
    Kill,
    /// Start the ADB server
    Start,
    /// Get ADB status and list devices
    Status,
    /// Reverse a port from device to host
    Reverse {
        /// Port to reverse
        #[arg(long, default_value = "3333")]
        port: u16,
    },
    /// Execute arbitrary adb command
    Exec {
        /// Arguments to pass to adb
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum FlutterCmd {
    /// Run Flutter app
    Run {
        /// Device ID to run on
        #[arg(long)]
        device: Option<String>,
        /// Build mode: debug, profile, release
        #[arg(long, default_value = "profile")]
        mode: String,
        /// Port to run on
        #[arg(long)]
        port: Option<u16>,
    },
    /// Build Flutter app
    Build {
        /// Build target: apk, appbundle, ios, etc.
        #[arg(long)]
        target: Option<String>,
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Clean build artifacts
    Clean,
    /// Get packages (flutter pub get)
    PubGet,
}

#[derive(Subcommand)]
pub enum PortCmd {
    /// Find processes using a specific port
    Find {
        /// Port number
        #[arg(long)]
        port: u16,
    },
    /// Kill all processes using a specific port
    Kill {
        /// Port number
        #[arg(long)]
        port: u16,
    },
}
