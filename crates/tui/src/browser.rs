//! Registry browser TUI — scrollable list + search input + detail pane.

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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single item shown in the registry browser list.
#[derive(Debug, Clone)]
pub struct BrowserItem {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: String,
    pub downloads: u64,
    pub starred: bool,
}

impl BrowserItem {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "latest".to_string(),
            description: description.into(),
            category: "general".to_string(),
            downloads: 0,
            starred: false,
        }
    }
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

struct BrowserApp {
    items: Vec<BrowserItem>,
    filtered: Vec<usize>,   // indices into `items`
    list_state: ListState,
    search: String,
    search_active: bool,
}

impl BrowserApp {
    fn new(items: Vec<BrowserItem>) -> Self {
        let filtered: Vec<usize> = (0..items.len()).collect();
        let mut list_state = ListState::default();
        if !filtered.is_empty() {
            list_state.select(Some(0));
        }
        Self { items, filtered, list_state, search: String::new(), search_active: false }
    }

    fn apply_filter(&mut self) {
        let q = self.search.to_lowercase();
        self.filtered = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.name.to_lowercase().contains(&q)
                    || item.description.to_lowercase().contains(&q)
                    || item.category.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect();

        let selected = self.list_state.selected().unwrap_or(0);
        if self.filtered.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(selected.min(self.filtered.len() - 1)));
        }
    }

    fn selected_item(&self) -> Option<&BrowserItem> {
        let idx = self.filtered.get(self.list_state.selected()?)?;
        self.items.get(*idx)
    }

    fn scroll_down(&mut self) {
        if self.filtered.is_empty() { return; }
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some((i + 1).min(self.filtered.len() - 1)));
    }

    fn scroll_up(&mut self) {
        if self.filtered.is_empty() { return; }
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(i.saturating_sub(1)));
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run_registry_browser(items: Vec<BrowserItem>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = BrowserApp::new(items);
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut BrowserApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            // Ctrl-C / q always quit
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(());
            }
            if !app.search_active && key.code == KeyCode::Char('q') {
                return Ok(());
            }
            if !app.search_active && key.code == KeyCode::Esc {
                return Ok(());
            }

            match key.code {
                KeyCode::Down | KeyCode::Char('j') if !app.search_active => app.scroll_down(),
                KeyCode::Up   | KeyCode::Char('k') if !app.search_active => app.scroll_up(),

                // Toggle search
                KeyCode::Char('/') if !app.search_active => {
                    app.search_active = true;
                }
                KeyCode::Esc if app.search_active => {
                    app.search_active = false;
                }
                KeyCode::Enter if app.search_active => {
                    app.search_active = false;
                }

                // Search input
                KeyCode::Char(c) if app.search_active => {
                    app.search.push(c);
                    app.apply_filter();
                }
                KeyCode::Backspace if app.search_active => {
                    app.search.pop();
                    app.apply_filter();
                }
                _ => {}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// UI drawing
// ---------------------------------------------------------------------------

fn ui(f: &mut ratatui::Frame, app: &mut BrowserApp) {
    let size = f.area();

    // Title + search bar at top
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(0),    // main content
            Constraint::Length(1), // footer / help
        ])
        .split(size);

    // --- Header ---
    let search_hint = if app.search_active { format!("Search: {}█", app.search) } else { format!("/ to search  ·  {} packages", app.filtered.len()) };
    let header = Paragraph::new(search_hint)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("tsx registry browser"));
    f.render_widget(header, vert[0]);

    // --- Main: list + detail pane ---
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(vert[1]);

    // List
    let list_items: Vec<ListItem> = app
        .filtered
        .iter()
        .map(|&i| {
            let item = &app.items[i];
            let star = if item.starred { "★ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(star, Style::default().fg(Color::Yellow)),
                Span::raw(&item.name),
                Span::styled(
                    format!("  v{}", item.version),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Packages"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, main[0], &mut app.list_state);

    // Detail pane
    let detail_text = if let Some(item) = app.selected_item() {
        vec![
            Line::from(vec![
                Span::styled("Name:        ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&item.name),
            ]),
            Line::from(vec![
                Span::styled("Version:     ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&item.version),
            ]),
            Line::from(vec![
                Span::styled("Category:    ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&item.category),
            ]),
            Line::from(vec![
                Span::styled("Downloads:   ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(item.downloads.to_string()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Description", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(item.description.as_str()),
        ]
    } else {
        vec![Line::from("No packages found. Press / to search.")]
    };

    let detail = Paragraph::new(detail_text)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });
    f.render_widget(detail, main[1]);

    // --- Footer ---
    let footer = Paragraph::new("↑↓/jk navigate  ·  / search  ·  q quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, vert[2]);
}
