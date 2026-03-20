//! Doctor checklist TUI view — live project health display.

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus { Ok, Warn, Fail, Info }

#[derive(Debug, Clone)]
pub struct CheckItem {
    pub label: String,
    pub status: CheckStatus,
    pub detail: String,
}

impl CheckItem {
    pub fn ok(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { label: label.into(), status: CheckStatus::Ok, detail: detail.into() }
    }
    pub fn warn(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { label: label.into(), status: CheckStatus::Warn, detail: detail.into() }
    }
    pub fn fail(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { label: label.into(), status: CheckStatus::Fail, detail: detail.into() }
    }
    pub fn info(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { label: label.into(), status: CheckStatus::Info, detail: detail.into() }
    }
}

/// Run a stub set of doctor checks and display them in the TUI.
pub fn run_doctor_view() -> io::Result<()> {
    let checks = collect_checks();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = doctor_loop(&mut terminal, &checks);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    result
}

fn doctor_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    checks: &[CheckItem],
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.area();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(size);

            let items: Vec<ListItem> = checks.iter().map(|c| {
                let (icon, color) = match c.status {
                    CheckStatus::Ok   => ("✓", Color::Green),
                    CheckStatus::Warn => ("⚠", Color::Yellow),
                    CheckStatus::Fail => ("✗", Color::Red),
                    CheckStatus::Info => ("ℹ", Color::Cyan),
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::styled(&c.label, Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!("  {}", c.detail), Style::default().fg(Color::DarkGray)),
                ]))
            }).collect();

            let ok = checks.iter().filter(|c| c.status == CheckStatus::Ok).count();
            let warn = checks.iter().filter(|c| c.status == CheckStatus::Warn).count();
            let fail = checks.iter().filter(|c| c.status == CheckStatus::Fail).count();
            let title = format!("tsx doctor — {} ok  {} warn  {} fail", ok, warn, fail);

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(title));
            f.render_widget(list, layout[0]);

            let footer = Paragraph::new("q / Esc to quit")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, layout[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
            {
                return Ok(());
            }
        }
    }
}

fn collect_checks() -> Vec<CheckItem> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    vec![
        if cwd.join("package.json").exists() {
            CheckItem::ok("package.json", "found")
        } else {
            CheckItem::fail("package.json", "not found — run `npm init`")
        },
        if cwd.join(".tsx").exists() {
            CheckItem::ok(".tsx/", "configuration directory present")
        } else {
            CheckItem::warn(".tsx/", "not found — run `tsx init`")
        },
        if cwd.join("tsconfig.json").exists() {
            CheckItem::ok("tsconfig.json", "found")
        } else {
            CheckItem::warn("tsconfig.json", "not found")
        },
        if cwd.join(".env").exists() {
            CheckItem::ok(".env", "found")
        } else {
            CheckItem::info(".env", "not found (optional)")
        },
        if cwd.join("node_modules").exists() {
            CheckItem::ok("node_modules/", "dependencies installed")
        } else {
            CheckItem::fail("node_modules/", "missing — run `npm install`")
        },
        if which_node() {
            CheckItem::ok("node", format!("found in PATH"))
        } else {
            CheckItem::fail("node", "not found in PATH")
        },
        if which_npx() {
            CheckItem::ok("npx", "found in PATH")
        } else {
            CheckItem::fail("npx", "not found in PATH")
        },
    ]
}

fn which_node() -> bool {
    std::process::Command::new("node")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn which_npx() -> bool {
    std::process::Command::new("npx")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
