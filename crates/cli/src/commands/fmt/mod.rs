use tsx_fmt::{format_file, FmtConfig, QuoteStyle};
use walkdir::WalkDir;

/// Format `.forge` / `.jinja` template files (normalise indent, quotes, spacing).
pub fn fmt(paths: Vec<String>, check: bool, indent: usize, quotes: String) {
    let quote_style = if quotes == "single" { QuoteStyle::Single } else { QuoteStyle::Double };
    let config = FmtConfig { indent, quotes: quote_style, ..FmtConfig::default() };

    let roots: Vec<std::path::PathBuf> = if paths.is_empty() {
        vec![std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))]
    } else {
        paths.iter().map(std::path::PathBuf::from).collect()
    };

    let mut total = 0usize;
    let mut changed = 0usize;

    for root in &roots {
        let walker = WalkDir::new(root).into_iter().filter_map(|e| e.ok());
        for entry in walker {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "forge" | "jinja" | "jinja2" | "j2") {
                continue;
            }
            total += 1;
            match format_file(path, &config, check) {
                Ok(r) if r.changed => {
                    changed += 1;
                    if check {
                        eprintln!("  would reformat: {}", path.display());
                    } else {
                        println!("  formatted: {}", path.display());
                    }
                }
                Ok(_) => {}
                Err(e) => eprintln!("  error: {}: {}", path.display(), e),
            }
        }
    }

    println!("{} file(s) checked, {} reformatted.", total, changed);
    if check && changed > 0 {
        std::process::exit(1);
    }
}
