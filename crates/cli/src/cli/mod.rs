mod args;
mod dispatch;

pub use args::Cli;

/// Parse argv, resolve `--stdin`/`--file` payload input, and dispatch to a command handler.
pub fn run() {
    use clap::Parser;
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

    dispatch::dispatch(cli, json_input);
}
