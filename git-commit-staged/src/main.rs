use anyhow::Result;
use clap::Parser;
use git2::Repository;
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

    if result.dry_run {
        println!("Dry run - not committing");
        println!();
        println!("Would commit with message:");
        println!("  {}", args.message);
        return Ok(());
    }

    if let Some(commit_oid) = result.commit_oid {
        println!("Created commit: {commit_oid}");
        let repo = Repository::discover(&args.directory)?;
        print_commit_oneline(&repo, commit_oid)?;
    }

    Ok(())
}

fn print_commit_oneline(repo: &Repository, oid: git2::Oid) -> Result<()> {
    use anyhow::Context;
    let commit = repo.find_commit(oid).context("failed to find commit")?;
    let short_id = commit
        .as_object()
        .short_id()
        .context("failed to get short id")?;
    let short_id_str = short_id.as_str().context("short id is not valid UTF-8")?;
    let summary = commit
        .summary()
        .context("commit message is not valid UTF-8")?;
    println!("{short_id_str} {summary}");
    Ok(())
}
