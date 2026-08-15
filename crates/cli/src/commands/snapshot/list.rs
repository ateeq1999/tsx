use std::path::PathBuf;

use crate::json::response::ResponseEnvelope;

use super::types::SnapshotFixture;

pub fn snapshot_list(_verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let generators = SnapshotFixture::list_generators(&cwd);

    let mut all: Vec<serde_json::Value> = Vec::new();
    for gen in &generators {
        let fixtures = SnapshotFixture::list(&cwd, gen);
        for fix in &fixtures {
            all.push(serde_json::json!({
                "generator": gen,
                "fixture": fix,
                "input": SnapshotFixture::fixture_path(&cwd, gen, fix).to_string_lossy(),
                "output_dir": SnapshotFixture::output_dir(&cwd, gen, fix).to_string_lossy(),
            }));
        }
    }

    ResponseEnvelope::success(
        "snapshot list",
        serde_json::json!({
            "count": all.len(),
            "snapshots": all,
        }),
        0,
    )
}
