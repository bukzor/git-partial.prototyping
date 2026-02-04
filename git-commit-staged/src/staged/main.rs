use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};
use std::path::Path;

mod cli;
use cli::Args;
use git_commit_staged::commit::do_commit;
use git_commit_staged::exec::print_dry_run;
use git_commit_staged::lock::IndexLock;
use git_commit_staged::prepare_staged_commit;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GIT_HASH"), ")");

fn main() -> Result<()> {
    let args = Args::from_arg_matches(&Args::command().version(VERSION).get_matches())?;

    // Acquire lock before reading any state (skip for dry-run)
    let _lock = if args.dry_run {
        None
    } else {
        Some(IndexLock::acquire()?)
    };

    let result = prepare_staged_commit(&args.paths, Path::new("."), args.dry_run)?;

    if args.dry_run {
        print_dry_run(&result.staged_entries);
        return Ok(());
    }

    let temp_index_path = result
        .temp_index_path
        .expect("non-dry-run should have temp index");

    // Lock held throughout - do_commit expects caller to hold it
    let output = do_commit(&temp_index_path, &args.passthrough_args)?;
    println!("[commit-staged {}]", &output.commit_sha[..7]);

    Ok(())
}
