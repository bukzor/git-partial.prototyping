use anyhow::Result;
use clap::Parser;
use std::path::Path;

mod cli;
use cli::Args;
use git_commit_staged::exec::{check_no_staged_changes, exec_git_commit, print_dry_run, stage_paths};
use git_commit_staged::prepare_staged_commit;
use git_commit_staged::unglobbed_path::UnglobbedPath;

fn main() -> Result<()> {
    let args = Args::parse();

    // Expand directories to files
    let files = UnglobbedPath::from_paths(&args.paths);
    if files.is_empty() {
        anyhow::bail!("no files found at specified paths");
    }

    // Bail if staging would destroy existing staged changes
    check_no_staged_changes(&files)?;

    // Stage the files from working tree
    stage_paths(&files)?;

    let result = prepare_staged_commit(&files, Path::new("."), args.dry_run)?;

    if args.dry_run {
        print_dry_run(&result.staged_entries);
        return Ok(());
    }

    let temp_index_path = result
        .temp_index_path
        .expect("non-dry-run should have temp index");

    exec_git_commit(&temp_index_path, &args.passthrough_args)
}
