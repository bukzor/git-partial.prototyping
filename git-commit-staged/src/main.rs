use anyhow::{Context, Result};
use clap::Parser;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use git_commit_staged::prepare_staged_commit;

#[derive(Parser, Debug)]
#[command(name = "git-commit-staged")]
#[command(about = "Commit staged changes at specific paths only")]
#[command(
    long_about = "Unlike `git commit -- paths`, this commits from the index, not the working copy.\n\n\
                  Arguments after -- are passed through to git commit,\n\
                  enabling -m, --amend, --fixup, -C, GPG signing, hooks, etc.\n\n\
                  Examples:\n\
                  \x20 git commit-staged src/ -- -m \"Add feature\"\n\
                  \x20 git commit-staged src/ tests/ -- --amend\n\
                  \x20 git commit-staged . -- --fixup HEAD~1"
)]
struct Args {
    /// Paths to commit (only staged changes at these paths)
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Show what would be committed without committing
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Arguments to pass through to git commit
    #[arg(last = true)]
    #[allow(clippy::struct_field_names)]
    passthrough_args: Vec<String>,
}

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
