use tsx_tui::{run, BrowserItem, TuiView};

/// Launch the ratatui terminal dashboard (registry browser, doctor, stack editor).
pub fn tui(view: String) {
    let tui_view = TuiView::from_str(&view);
    // Load items from all installed packages via PackageStore.
    let store = crate::packages::PackageStore::default();
    let mut items: Vec<BrowserItem> = store
        .list()
        .into_iter()
        .map(|p| BrowserItem::new(&p.id, &p.description))
        .collect();
    // Fallback: if no packages are installed, show placeholder guidance.
    if items.is_empty() {
        items.push(BrowserItem::new(
            "No packages installed",
            "Run `tsx registry install <pkg>` or `tsx package install <id>` to get started",
        ));
    }
    if let Err(e) = run(tui_view, items) {
        eprintln!("TUI error: {}", e);
        std::process::exit(1);
    }
}
