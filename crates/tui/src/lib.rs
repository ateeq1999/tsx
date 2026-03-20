//! **tsx-tui** — ratatui-based terminal dashboard for the tsx CLI.
//!
//! Provides three interactive TUI views:
//! - **Registry browser** — searchable list of registry packages with detail pane
//! - **Doctor checklist** — live project health checklist
//! - **Stack editor** — interactive `.tsx/stack.json` editor
//!
//! ## Usage from tsx CLI
//! ```text
//! tsx tui                     # registry browser (default)
//! tsx tui --view doctor       # doctor checklist
//! tsx tui --view stack        # stack editor
//! ```

pub mod browser;
pub mod doctor;
pub mod stack_editor;

pub use browser::{run_registry_browser, BrowserItem};
pub use doctor::run_doctor_view;
pub use stack_editor::run_stack_editor;

/// Which TUI view to launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiView {
    Browser,
    Doctor,
    Stack,
}

impl TuiView {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "doctor" | "health" => TuiView::Doctor,
            "stack" => TuiView::Stack,
            _ => TuiView::Browser,
        }
    }
}

/// Launch the requested TUI view.
///
/// Returns `Ok(())` when the user quits (q / Esc / Ctrl-C).
pub fn run(view: TuiView, items: Vec<BrowserItem>) -> std::io::Result<()> {
    match view {
        TuiView::Browser => run_registry_browser(items),
        TuiView::Doctor => run_doctor_view(),
        TuiView::Stack => run_stack_editor(),
    }
}
