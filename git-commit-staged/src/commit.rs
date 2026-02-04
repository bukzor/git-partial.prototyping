//! Commit with index lock held.
//!
//! Spawns git commit as subprocess while holding the main index lock,
//! preserving full git commit compatibility (--amend, --fixup, -C, etc.).

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Result of a successful commit operation.
pub struct CommitOutput {
    /// The new commit's SHA (short form from git output).
    pub commit_sha: String,
}

/// Commit from a temporary index file while caller holds index.lock.
///
/// Spawns `git commit` as a subprocess with `GIT_INDEX_FILE` set to the temp index.
/// Git will lock the temp index (not the main one), so no conflict with our lock.
///
/// **Caller must hold index.lock** for the entire operation.
///
/// # Errors
/// - Git commit fails (hooks reject, no message, etc.)
pub fn do_commit(
    temp_index_path: &Path,
    passthrough_args: &[String],
) -> Result<CommitOutput> {
    let output = Command::new("git")
        .arg("commit")
        .args(passthrough_args)
        .env("GIT_INDEX_FILE", temp_index_path)
        .output()
        .context("failed to run git commit")?;

    // Clean up temp index regardless of outcome
    let _ = std::fs::remove_file(temp_index_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git commit failed: {stderr}");
    }

    // Parse commit SHA from output (e.g., "[main abc1234] message")
    let stdout = String::from_utf8_lossy(&output.stdout);
    let commit_sha = parse_commit_sha(&stdout).unwrap_or_else(|| "unknown".to_string());

    Ok(CommitOutput { commit_sha })
}

/// Extract short SHA from git commit output like "[main abc1234] message"
fn parse_commit_sha(output: &str) -> Option<String> {
    let line = output.lines().next()?;
    let start = line.find(' ')? + 1;
    let end = line[start..].find(']').map(|i| start + i)?;
    Some(line[start..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sha_from_commit_output() {
        assert_eq!(
            parse_commit_sha("[main abc1234] Add feature"),
            Some("abc1234".to_string())
        );
        assert_eq!(
            parse_commit_sha("[detached HEAD def5678] Fix bug"),
            Some("HEAD def5678".to_string())
        );
    }
}
