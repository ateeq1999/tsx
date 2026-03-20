//! Stack editor TUI view — view and edit .tsx/stack.json keys interactively.

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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

struct StackEditorApp {
    entries: Vec<(String, String)>,
    list_state: ListState,
    dirty: bool,
}

impl StackEditorApp {
    fn new(entries: Vec<(String, String)>) -> Self {
        let mut state = ListState::default();
        if !entries.is_empty() { state.select(Some(0)); }
        Self { entries, list_state: state, dirty: false }
    }

    fn scroll_down(&mut self) {
        if self.entries.is_empty() { return; }
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some((i + 1).min(self.entries.len() - 1)));
    }

    fn scroll_up(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(i.saturating_sub(1)));
    }
}

pub fn run_stack_editor() -> io::Result<()> {
    let entries = load_stack_entries();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = StackEditorApp::new(entries);
    let result = editor_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    result
}

fn editor_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut StackEditorApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.area();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(size);

            let dirty_hint = if app.dirty { " [modified]" } else { "" };
            let header = Paragraph::new(format!(".tsx/stack.json{}", dirty_hint))
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL).title("Stack Editor"));
            f.render_widget(header, layout[0]);

            let items: Vec<ListItem> = app.entries.iter().map(|(k, v)| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:<24}", k), Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(v.as_str(), Style::default().fg(Color::Green)),
                ]))
            }).collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Keys"))
                .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            f.render_stateful_widget(list, layout[1], &mut app.list_state);

            let footer = Paragraph::new("↑↓ navigate  ·  q quit  (editing not yet implemented — use tsx config set)")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, layout[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
            {
                return Ok(());
            }
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Up   | KeyCode::Char('k') => app.scroll_up(),
                _ => {}
            }
        }
    }
}

fn load_stack_entries() -> Vec<(String, String)> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let stack_path = cwd.join(".tsx").join("stack.json");

    if let Ok(content) = std::fs::read_to_string(&stack_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = val.as_object() {
                return obj
                    .iter()
                    .map(|(k, v)| {
                        let display = match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Number(n) => n.to_string(),
                            other => serde_json::to_string(other).unwrap_or_default(),
                        };
                        (k.clone(), display)
                    })
                    .collect();
            }
        }
    }

    // Fallback: show empty state
    vec![
        ("(no .tsx/stack.json found)".to_string(), "run `tsx init` to create one".to_string()),
    ]
}
