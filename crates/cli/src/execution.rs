/// Execution flags shared by generator commands that need more than one of
/// overwrite/dry_run/diff — grouped here so they're threaded as a single
/// borrowed value instead of a run of same-typed positional `bool`s that are
/// easy to transpose by accident at a call site.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExecutionContext {
    pub overwrite: bool,
    pub dry_run: bool,
    pub diff: bool,
    pub verbose: bool,
}
