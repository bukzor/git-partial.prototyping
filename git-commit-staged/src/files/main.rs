use anyhow::{bail, Result};
use clap::{CommandFactory, FromArgMatches};

mod cli;
use cli::Args;
use git_commit_staged::commit::do_commit;
use git_commit_staged::exec::{
    check_no_staged_changes, commit_staged_index, discard_staged_index, print_dry_run,
    stage_paths_to_temp,
};
use git_commit_staged::index::write_temp_index_for_paths;
use git_commit_staged::lock::IndexLock;
use git_commit_staged::unglobbed_path::UnglobbedPath;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GIT_HASH"), ")");

fn main() -> Result<()> {
    let args = Args::from_arg_matches(&Args::command().version(VERSION).get_matches())?;

    // Expand directories to files
    let files = UnglobbedPath::from_paths(&args.paths);
    if files.is_empty() {
        bail!("no files found at specified paths");
    }

    // Acquire lock before reading any state (skip for dry-run)
    let _lock = if args.dry_run {
        None
    } else {
        Some(IndexLock::acquire()?)
    };

    // Bail if staging would destroy existing staged changes
    check_no_staged_changes(&files)?;

    // Stage working tree to temp index (same code path for dry-run and real)
    let stage_result = stage_paths_to_temp(&files)?;

    if stage_result.staged_entries.is_empty() {
        discard_staged_index(&stage_result)?;
        bail!("no changes to commit at specified paths");
    }

    if args.dry_run {
        print_dry_run(&stage_result.staged_entries);
        discard_staged_index(&stage_result)?;
        return Ok(());
    }

    // Commit path: rename temp → real index
    commit_staged_index(&stage_result)?;

    // Create temp index for commit (HEAD + staged entries)
    let commit_index_path = write_temp_index_for_paths(&stage_result.staged_entries)?;

    // Lock held throughout - do_commit expects caller to hold it
    let output = do_commit(&commit_index_path, &args.passthrough_args)?;
    println!("[commit-files {}]", &output.commit_sha[..7]);

    Ok(())
}
