use crate::output::CommandResult;

/// Validate the shell name and return the matching `clap_complete::Shell`.
/// Returns `Err` with a user-facing message if the name is unrecognised.
pub fn resolve_shell(shell: &str) -> Result<clap_complete::Shell, String> {
    match shell.to_ascii_lowercase().as_str() {
        "bash"                    => Ok(clap_complete::Shell::Bash),
        "zsh"                     => Ok(clap_complete::Shell::Zsh),
        "fish"                    => Ok(clap_complete::Shell::Fish),
        "powershell" | "ps" | "pwsh" => Ok(clap_complete::Shell::PowerShell),
        "elvish"                  => Ok(clap_complete::Shell::Elvish),
        other => Err(format!(
            "Unknown shell '{}'. Supported: bash, zsh, fish, powershell, elvish",
            other
        )),
    }
}

/// Dummy `CommandResult` helper used when the shell name is invalid.
pub fn unknown_shell_error(shell: &str) -> CommandResult {
    CommandResult::err(
        "completions",
        format!(
            "Unknown shell '{}'. Supported: bash, zsh, fish, powershell, elvish",
            shell
        ),
    )
}
