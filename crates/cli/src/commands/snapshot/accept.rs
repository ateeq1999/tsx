use crate::json::response::ResponseEnvelope;

use super::update::snapshot_update;

pub fn snapshot_accept(generator: Option<String>, verbose: bool) -> ResponseEnvelope {
    // Accept is the same as update
    snapshot_update(generator, verbose)
}
