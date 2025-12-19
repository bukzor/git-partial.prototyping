use anyhow::Result;
use clap::Parser;
use std::path::Path;

mod cli;
use cli::Args;
use git_commit_staged::exec::{exec_git_commit, print_dry_run, stage_paths};
use git_commit_staged::prepare_staged_commit;

fn main() -> Result<()> {
    let args = Args::parse();

    // Stage the paths from working tree first
    stage_paths(&args.paths)?;

    let result = prepare_staged_commit(&args.paths, Path::new("."), args.dry_run)?;

    if args.dry_run {
        print_dry_run(&result.staged_entries);
        return Ok(());
    }

    let temp_index_path = result
        .temp_index_path
        .expect("non-dry-run should have temp index");

    exec_git_commit(&temp_index_path, &args.passthrough_args)
}
