use tsx_docs::{collect_topics, default_roots, run_docs_viewer};

/// Browse offline documentation from `.tsx/knowledge/` in a terminal UI, or list/filter it non-interactively.
pub fn docs(paths: Vec<String>, search: Option<String>, json: bool) {
    let mut roots = default_roots();
    for p in &paths {
        let pb = std::path::PathBuf::from(p);
        if pb.exists() {
            roots.push(pb);
        }
    }

    let mut topics = collect_topics(&roots);

    if let Some(q) = &search {
        let q = q.to_lowercase();
        topics.retain(|t| {
            t.title.to_lowercase().contains(&q)
                || t.category.to_lowercase().contains(&q)
                || t.summary.to_lowercase().contains(&q)
        });
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&topics).unwrap_or_default());
    } else if search.is_some() {
        // Non-interactive filtered output
        for t in &topics {
            println!("[{}] {} — {}", t.category, t.title, t.summary);
        }
    } else if let Err(e) = run_docs_viewer(topics) {
        eprintln!("docs viewer error: {}", e);
        std::process::exit(1);
    }
}
