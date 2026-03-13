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
        #[arg(long, value_parser = parse_json)]
        json: Option<String>,
    },
    /// Generate a Drizzle schema table definition
    #[command(name = "add:schema")]
    AddSchema {
        /// JSON payload
        #[arg(long, value_parser = parse_json)]
        json: Option<String>,
    },
    /// Generate a typed server function
    #[command(name = "add:server-fn")]
    AddServerFn {
        /// JSON payload
        #[arg(long, value_parser = parse_json)]
        json: Option<String>,
    },
    /// Generate a TanStack Query hook
    #[command(name = "add:query")]
    AddQuery {
        /// JSON payload
        #[arg(long, value_parser = parse_json)]
        json: Option<String>,
    },
    /// Generate a TanStack Form component
    #[command(name = "add:form")]
    AddForm {
        /// JSON payload
        #[arg(long, value_parser = parse_json)]
        json: Option<String>,
    },
    /// Generate a TanStack Table component
    #[command(name = "add:table")]
    AddTable {
        /// JSON payload
        #[arg(long, value_parser = parse_json)]
        json: Option<String>,
    },
    /// Add a new route page
    #[command(name = "add:page")]
    AddPage {
        /// JSON payload
        #[arg(long, value_parser = parse_json)]
        json: Option<String>,
    },
    /// Configure Better Auth
    #[command(name = "add:auth")]
    AddAuth {
        /// JSON payload
        #[arg(long, value_parser = parse_json)]
        json: Option<String>,
    },
    /// Wrap a route with a session guard
    #[command(name = "add:auth-guard")]
    AddAuthGuard {
        /// JSON payload
        #[arg(long, value_parser = parse_json)]
        json: Option<String>,
    },
    /// Run drizzle-kit generate + migrate
    #[command(name = "add:migration")]
    AddMigration,
    /// Generate a database seed file
    #[command(name = "add:seed")]
    AddSeed {
        /// JSON payload
        #[arg(long, value_parser = parse_json)]
        json: Option<String>,
    },
}

fn parse_json(s: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(s).map_err(|e| e.to_string())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Init { .. } => println!("not yet implemented"),
        Command::AddFeature { .. } => println!("not yet implemented"),
        Command::AddSchema { .. } => println!("not yet implemented"),
        Command::AddServerFn { .. } => println!("not yet implemented"),
        Command::AddQuery { .. } => println!("not yet implemented"),
        Command::AddForm { .. } => println!("not yet implemented"),
        Command::AddTable { .. } => println!("not yet implemented"),
        Command::AddPage { .. } => println!("not yet implemented"),
        Command::AddAuth { .. } => println!("not yet implemented"),
        Command::AddAuthGuard { .. } => println!("not yet implemented"),
        Command::AddMigration => println!("not yet implemented"),
        Command::AddSeed { .. } => println!("not yet implemented"),
    }
}
