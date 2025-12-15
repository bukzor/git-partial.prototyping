use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use git_commit_staged::git_commit_staged;

#[derive(Parser, Debug)]
#[command(name = "git-commit-staged")]
#[command(about = "Commit staged changes at specific paths only")]
#[command(
    long_about = "Unlike `git commit -- paths`, this commits from the index, not the working copy."
)]
struct Args {
    /// Paths to commit (only staged changes at these paths)
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Commit message
    #[arg(short, long, required = true)]
    message: String,

    /// Run as if git was started in <path>
    #[arg(short = 'C', long = "directory", default_value = ".")]
    directory: PathBuf,

    /// Show what would be committed without committing
    #[arg(short = 'n', long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let result = git_commit_staged(&args.paths, &args.message, &args.directory, args.dry_run)?;

    println!("Files to commit:");
    for (path, data) in &result.staged_entries {
        let status = if data.is_some() { "M" } else { "D" };
        println!("  {status} {path}");
    }
    println!();

    let Some(commit_oid) = result.commit_oid else {
        // Dry run
        println!("Would commit with message:");
        println!("  {}", args.message);
        return Ok(());
    };

    // Print short hash + first line of message (like git commit does)
    println!(
        "[{:.7}] {}",
        commit_oid,
        args.message.lines().next().unwrap_or("")
    );

    Ok(())
}
