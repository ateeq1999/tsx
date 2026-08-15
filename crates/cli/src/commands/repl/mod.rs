//! `tsx repl` — interactive goal-driven REPL.
//!
//! The user types a natural-language goal; the REPL:
//! 1. Matches the goal against the command registry (same scoring as `tsx plan`)
//! 2. Shows the proposed commands
//! 3. Asks for confirmation (y/n)
//! 4. Executes each confirmed command via the batch dispatcher
//!
//! In agent mode (`--goal <goal>`), returns a JSON plan without prompting.
//! With `--execute`, runs the commands immediately.

use std::io::{BufRead, Write};

use crate::framework::command_registry::CommandRegistry;
use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn repl(goal: Option<String>, execute: bool, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    if let Some(g) = goal {
        return one_shot(g, execute, start);
    }

    if !atty_stdin() {
        return ResponseEnvelope::error(
            "repl",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                "No --goal provided and stdin is not a TTY. Use --goal <goal> for agent mode.",
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    run_interactive_loop(start)
}

// ---------------------------------------------------------------------------
// One-shot mode
// ---------------------------------------------------------------------------

fn one_shot(goal: String, execute: bool, start: std::time::Instant) -> ResponseEnvelope {
    let commands = plan_goal(&goal);

    if execute {
        let mut executed: Vec<serde_json::Value> = Vec::new();
        for cmd in &commands {
            let command = cmd["command"].as_str().unwrap_or("").to_string();
            let args = cmd["args"].clone();
            let result = crate::commands::batch::execute_command_pub(&command, &args, false, false);
            executed.push(serde_json::json!({
                "command": command,
                "status": if result.is_ok() { "ok" } else { "error" },
                "detail": result.err().map(|(_, e)| e).unwrap_or_default(),
            }));
        }
        let result = serde_json::json!({ "goal": goal, "executed": executed });
        return ResponseEnvelope::success("repl", result, start.elapsed().as_millis() as u64);
    }

    let result = serde_json::json!({
        "goal": goal,
        "proposed_commands": commands,
        "hint": "Run with --execute to apply, or use `tsx plan` to view without executing",
    });
    ResponseEnvelope::success("repl", result, start.elapsed().as_millis() as u64)
}

// ---------------------------------------------------------------------------
// Interactive loop
// ---------------------------------------------------------------------------

fn run_interactive_loop(start: std::time::Instant) -> ResponseEnvelope {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let _ = writeln!(out, "tsx repl — type a goal and press Enter ('exit' to quit).\n");

    let mut history: Vec<serde_json::Value> = Vec::new();

    for line in stdin.lock().lines() {
        let Ok(input) = line else { break };
        let input = input.trim().to_string();
        if input.is_empty() { continue; }
        if input == "exit" || input == "quit" { break; }

        let commands = plan_goal(&input);

        if commands.is_empty() {
            let _ = writeln!(out, "No matching commands found for: {}\n", input);
            history.push(serde_json::json!({ "goal": input, "status": "unresolved" }));
            continue;
        }

        let _ = writeln!(out, "\nProposed commands:");
        for (i, cmd) in commands.iter().enumerate() {
            let command = cmd["command"].as_str().unwrap_or("");
            let _ = writeln!(out, "  {}. {}", i + 1, command);
        }

        let _ = write!(out, "\nExecute? [y/n] ");
        let _ = out.flush();

        let mut answer = String::new();
        let _ = stdin.lock().read_line(&mut answer);

        if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
            let _ = writeln!(out, "Skipped.\n");
            history.push(serde_json::json!({ "goal": input, "status": "skipped" }));
            continue;
        }

        let mut executed: Vec<serde_json::Value> = Vec::new();
        for cmd in &commands {
            let command = cmd["command"].as_str().unwrap_or("").to_string();
            let args = cmd["args"].clone();
            let result = crate::commands::batch::execute_command_pub(&command, &args, false, false);
            let status = if result.is_ok() { "ok" } else { "error" };
            let _ = writeln!(out, "  {} {}", if status == "ok" { "✓" } else { "✗" }, command);
            executed.push(serde_json::json!({ "command": command, "status": status }));
        }
        let _ = writeln!(out);
        history.push(serde_json::json!({ "goal": input, "executed": executed }));
    }

    let result = serde_json::json!({ "session_history": history });
    ResponseEnvelope::success("repl", result, start.elapsed().as_millis() as u64)
}

// ---------------------------------------------------------------------------
// Goal planning (mirrors tsx plan scoring logic)
// ---------------------------------------------------------------------------

fn plan_goal(goal: &str) -> Vec<serde_json::Value> {
    let registry = CommandRegistry::load_all();
    let specs = registry.all();
    let goal_tokens = tokenise(goal);

    let mut scored: Vec<(usize, serde_json::Value)> = specs
        .iter()
        .filter_map(|spec| {
            let candidate = format!("{} {} {}", spec.id, spec.command, spec.description);
            let s = score(&goal_tokens, &tokenise(&candidate));
            if s == 0 { return None; }
            Some((s, serde_json::json!({
                "command": spec.command,
                "generator_id": spec.id,
                "framework": spec.framework,
                "description": spec.description,
                "score": s,
                "args": {},
            })))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(3).map(|(_, v)| v).collect()
}

fn tokenise(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 1)
        .map(|t| t.to_lowercase())
        .collect()
}

fn score(goal: &[String], candidate: &[String]) -> usize {
    goal.iter().filter(|t| candidate.contains(t)).count()
}

fn atty_stdin() -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        unsafe { libc::isatty(std::io::stdin().as_raw_fd()) != 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}
