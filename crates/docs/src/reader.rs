//! ratatui-based documentation reader TUI.
//!
//! Layout:
//!   ┌─────────────────────────────────────────────────────────┐
//!   │  tsx docs — <search>                                    │
//!   ├──────────────┬──────────────────────────────────────────┤
//!   │ Topic list   │  Document content (scrollable)           │
//!   │  (filterable)│                                          │
//!   │              │                                          │
//!   ├──────────────┴──────────────────────────────────────────┤
//!   │  ↑↓ navigate · Tab switch pane · / search · q quit     │
//!   └─────────────────────────────────────────────────────────┘

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

use crate::markdown::render_markdown;
use crate::topic::DocTopic;

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(PartialEq, Eq)]
enum ActivePane { List, Reader }

struct DocsApp {
    topics: Vec<DocTopic>,
    filtered: Vec<usize>,
    list_state: ListState,
    search: String,
    search_active: bool,
    active_pane: ActivePane,
    scroll_offset: u16,
    current_content: Option<String>,
}

impl DocsApp {
    fn new(topics: Vec<DocTopic>) -> Self {
        let filtered: Vec<usize> = (0..topics.len()).collect();
        let mut list_state = ListState::default();
        if !filtered.is_empty() {
            list_state.select(Some(0));
        }
        let mut app = Self {
            topics,
            filtered,
            list_state,
            search: String::new(),
            search_active: false,
            active_pane: ActivePane::List,
            scroll_offset: 0,
            current_content: None,
        };
        app.load_selected();
        app
    }

    fn apply_filter(&mut self) {
        let q = self.search.to_lowercase();
        self.filtered = self
            .topics
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                t.title.to_lowercase().contains(&q)
                    || t.category.to_lowercase().contains(&q)
                    || t.summary.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect();

        let sel = self.list_state.selected().unwrap_or(0);
        if self.filtered.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(sel.min(self.filtered.len() - 1)));
        }
        self.load_selected();
    }

    fn selected_topic(&self) -> Option<&DocTopic> {
        let idx = self.filtered.get(self.list_state.selected()?)?;
        self.topics.get(*idx)
    }

    fn load_selected(&mut self) {
        self.current_content = self
            .selected_topic()
            .and_then(|t| std::fs::read_to_string(&t.path).ok());
        self.scroll_offset = 0;
    }

    fn scroll_list_down(&mut self) {
        if self.filtered.is_empty() { return; }
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some((i + 1).min(self.filtered.len() - 1)));
        self.load_selected();
    }

    fn scroll_list_up(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(i.saturating_sub(1)));
        self.load_selected();
    }

    fn scroll_reader_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(3);
    }

    fn scroll_reader_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(3);
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run_docs_viewer(topics: Vec<DocTopic>) -> io::Result<()> {
    if topics.is_empty() {
        eprintln!("No documentation topics found. Place .md files in .tsx/knowledge/");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = DocsApp::new(topics);
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut DocsApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            // Always-available: Ctrl-C
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(());
            }

            // Search mode
            if app.search_active {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => app.search_active = false,
                    KeyCode::Char(c) => { app.search.push(c); app.apply_filter(); }
                    KeyCode::Backspace => { app.search.pop(); app.apply_filter(); }
                    _ => {}
                }
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Tab => {
                    app.active_pane = match app.active_pane {
                        ActivePane::List => ActivePane::Reader,
                        ActivePane::Reader => ActivePane::List,
                    };
                }
                KeyCode::Char('/') => app.search_active = true,

                KeyCode::Down | KeyCode::Char('j') => match app.active_pane {
                    ActivePane::List => app.scroll_list_down(),
                    ActivePane::Reader => app.scroll_reader_down(),
                },
                KeyCode::Up | KeyCode::Char('k') => match app.active_pane {
                    ActivePane::List => app.scroll_list_up(),
                    ActivePane::Reader => app.scroll_reader_up(),
                },
                KeyCode::PageDown => app.scroll_reader_down(),
                KeyCode::PageUp => app.scroll_reader_up(),
                _ => {}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// UI rendering
// ---------------------------------------------------------------------------

fn ui(f: &mut ratatui::Frame, app: &mut DocsApp) {
    let size = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    // Header
    let header_text = if app.search_active {
        format!("Search: {}█", app.search)
    } else if !app.search.is_empty() {
        format!("tsx docs  [filter: {}]  ({} topics)", app.search, app.filtered.len())
    } else {
        format!("tsx docs  ({} topics)", app.filtered.len())
    };
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("tsx docs"));
    f.render_widget(header, outer[0]);

    // Main split: list | reader
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(outer[1]);

    // --- Topic list ---
    let list_border_style = if app.active_pane == ActivePane::List {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mut last_cat = String::new();
    let list_items: Vec<ListItem> = app
        .filtered
        .iter()
        .flat_map(|&i| {
            let topic = &app.topics[i];
            let mut items = Vec::new();
            if topic.category != last_cat {
                last_cat = topic.category.clone();
                items.push(ListItem::new(Line::from(Span::styled(
                    format!(" {} ", topic.category.to_uppercase()),
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
                ))));
            }
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(&topic.title, Style::default()),
                topic.token_estimate.map(|t| {
                    Span::styled(
                        format!(" ~{}t", t),
                        Style::default().fg(Color::DarkGray),
                    )
                }).unwrap_or_else(|| Span::raw("")),
            ])));
            items
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Topics")
                .border_style(list_border_style),
        )
        .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, main[0], &mut app.list_state);

    // --- Reader pane ---
    let reader_border_style = if app.active_pane == ActivePane::Reader {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let reader_title = app
        .selected_topic()
        .map(|t| t.title.clone())
        .unwrap_or_else(|| "No topic selected".to_string());

    let content_lines: Vec<Line<'static>> = match &app.current_content {
        Some(md) => render_markdown(md),
        None => vec![Line::from(Span::styled(
            "Select a topic from the list.",
            Style::default().fg(Color::DarkGray),
        ))],
    };

    let reader = Paragraph::new(content_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(reader_title)
                .border_style(reader_border_style),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));
    f.render_widget(reader, main[1]);

    // Footer
    let footer = Paragraph::new(
        "↑↓/jk navigate  ·  Tab switch pane  ·  / search  ·  PgUp/PgDn scroll  ·  q quit",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, outer[2]);
}