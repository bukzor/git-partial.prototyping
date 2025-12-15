use anyhow::{Context, Result};
use clap::Parser;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

mod cli;
use cli::Args;
use git_commit_staged::prepare_staged_commit;

fn main() -> Result<()> {
    let args = Args::parse();

    let result = prepare_staged_commit(&args.paths, Path::new("."), args.dry_run)?;

    if args.dry_run {
        println!("Files to commit:");
        for (path, data) in &result.staged_entries {
            let status = if data.is_some() { "M" } else { "D" };
            println!("  {status} {path}");
        }
        return Ok(());
    }

    let temp_index_path = result
        .temp_index_path
        .expect("non-dry-run should have temp index");

    // Build git commit command with passthrough args
    let mut cmd = Command::new("git");
    cmd.arg("commit");
    cmd.args(&args.passthrough_args);
    cmd.env("GIT_INDEX_FILE", &temp_index_path);

    // exec() replaces this process with git commit
    // The temp index cleanup happens via the OS when the temp file is no longer referenced,
    // but we should clean it up explicitly on error. For now, exec() means we don't return.
    let err = cmd.exec();

    // exec() only returns on error
    // Clean up temp index before propagating error
    let _ = std::fs::remove_file(&temp_index_path);
    Err(err).context("failed to exec git commit")
}
